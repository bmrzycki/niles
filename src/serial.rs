use std::collections::VecDeque;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_serial::{DataBits, FlowControl, Parity, SerialPortBuilderExt, SerialStream, StopBits};

pub const VOL_UP: u8 = 12;
pub const VOL_DOWN: u8 = 13;
pub const MUTE: u8 = 11;
pub const ZONE_OFF: u8 = 10;

const REPLY_TIMEOUT: Duration = Duration::from_millis(800);

/// A command the HTTP allowlist is willing to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Zone { zone: u8, code: u8 },
    AllOff,
}

impl Command {
    /// Bytes written to the receiver, including the trailing carriage return.
    pub fn wire(&self) -> String {
        match self {
            Command::Zone { zone, code } => format!("zsc,{zone},{code:02}\r"),
            Command::AllOff => "znt,10,h\r".to_string(),
        }
    }

    pub fn log_text(&self) -> String {
        self.wire().trim_end_matches(['\r', '\n']).to_string()
    }
}

/// Keep at most one waiting volume step of the same zone and direction.
/// The command already in flight is not in `queue`.
pub fn push_coalesced(queue: &mut VecDeque<Command>, command: Command) {
    if let Command::Zone { zone, code } = command {
        if code == VOL_UP || code == VOL_DOWN {
            let waiting = queue.iter().any(|queued| {
                matches!(
                    queued,
                    Command::Zone { zone: queued_zone, code: queued_code }
                        if *queued_zone == zone && *queued_code == code
                )
            });
            if waiting {
                tracing::debug!(zone, code, "volume step already queued");
                return;
            }
        }
    }
    queue.push_back(command);
}

pub fn open(path: &str) -> std::io::Result<SerialStream> {
    tokio_serial::new(path, 38_400)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(REPLY_TIMEOUT)
        .open_native_async()
        .map_err(std::io::Error::other)
}

pub async fn run(mut rx: UnboundedReceiver<Command>, mut port: Option<SerialStream>) {
    let mut queue = VecDeque::new();
    loop {
        let command = if let Some(command) = queue.pop_front() {
            command
        } else {
            match rx.recv().await {
                Some(command) => command,
                None => break,
            }
        };
        while let Ok(extra) = rx.try_recv() {
            push_coalesced(&mut queue, extra);
        }
        if let Some(port) = port.as_mut() {
            write_and_drain(port, &command).await;
        } else {
            tracing::info!(command = %command.log_text(), "debug");
        }
    }
}

async fn write_and_drain(port: &mut SerialStream, command: &Command) {
    let wire = command.wire();
    let shown = command.log_text();
    if let Err(err) = port.write_all(wire.as_bytes()).await {
        tracing::warn!(%err, command = %shown, "serial write failed");
        return;
    }
    if let Err(err) = port.flush().await {
        tracing::warn!(%err, command = %shown, "serial flush failed");
        return;
    }
    tracing::info!(command = %shown, "sent");

    let mut reply = Vec::new();
    let read = tokio::time::timeout(REPLY_TIMEOUT, read_line(port, &mut reply)).await;
    match read {
        Ok(Ok(())) => {
            let text = String::from_utf8_lossy(&reply);
            if text.contains("FAIL") {
                tracing::warn!(reply = %text, command = %shown, "receiver rejected command");
            } else {
                tracing::debug!(reply = %text, command = %shown, "reply");
            }
        }
        Ok(Err(err)) => tracing::warn!(%err, command = %shown, "serial read failed"),
        Err(_) => tracing::warn!(command = %shown, "no reply"),
    }
}

async fn read_line(port: &mut SerialStream, buf: &mut Vec<u8>) -> std::io::Result<()> {
    let mut byte = [0_u8; 1];
    loop {
        port.read_exact(&mut byte).await?;
        if byte[0] == b'\r' {
            return Ok(());
        }
        if byte[0] != b'\n' {
            buf.push(byte[0]);
        }
        if buf.len() > 512 {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_formats_are_the_allowlist() {
        let up = Command::Zone {
            zone: 2,
            code: VOL_UP,
        };
        assert_eq!(up.wire(), "zsc,2,12\r");
        assert_eq!(Command::Zone { zone: 1, code: 5 }.wire(), "zsc,1,05\r");
        assert_eq!(
            Command::Zone {
                zone: 6,
                code: ZONE_OFF
            }
            .wire(),
            "zsc,6,10\r"
        );
        assert_eq!(
            Command::Zone {
                zone: 4,
                code: MUTE
            }
            .wire(),
            "zsc,4,11\r"
        );
        assert_eq!(
            Command::Zone {
                zone: 3,
                code: VOL_DOWN
            }
            .wire(),
            "zsc,3,13\r"
        );
        assert_eq!(Command::AllOff.wire(), "znt,10,h\r");
        let joined = format!(
            "{}{}{}",
            up.wire(),
            Command::AllOff.wire(),
            Command::Zone { zone: 1, code: 3 }.wire()
        );
        assert!(!joined.contains("syc"));
    }

    #[test]
    fn volume_hold_keeps_one_extra_step() {
        let up = Command::Zone {
            zone: 1,
            code: VOL_UP,
        };
        let down = Command::Zone {
            zone: 1,
            code: VOL_DOWN,
        };
        let other = Command::Zone {
            zone: 2,
            code: VOL_UP,
        };
        let mut queue = VecDeque::new();
        push_coalesced(&mut queue, up.clone());
        push_coalesced(&mut queue, up.clone());
        push_coalesced(&mut queue, up);
        assert_eq!(queue.len(), 1);

        push_coalesced(&mut queue, down);
        push_coalesced(&mut queue, other);
        push_coalesced(
            &mut queue,
            Command::Zone {
                zone: 1,
                code: MUTE,
            },
        );
        assert_eq!(queue.len(), 4);
    }
}
