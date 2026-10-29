# ZR-6 web control

A one-page controller for the Niles ZR-6, opened from Safari on an iPhone, iPad, or Mac. The usual visit is: pick a room, pick an input, set the volume, and later turn it off.

The wire protocol is in [protocol.md](protocol.md). This server uses only the commands listed below.

## Out of scope

The server's command allowlist is the safety boundary. Anything absent from the allowlist is never written to the serial port, including a URL a browser did not offer.

Left out on purpose:

- Tuner and direct tune. Inputs are RCA sources only.
- Bass, treble, and balance.
- Transport keys: play, pause, stop, skip, seek, and the rest. Sources are AirPlay or Bluetooth devices.
- Status polling and a volume slider. The receiver has no absolute volume command.
- Factory reset and any other system command. Those commands must not appear in this program.

## Screen

One screen, `100dvh`, with the home-indicator inset respected. Nothing on the page scrolls.

Zone tabs are a single row across the top. The room name is the first line and the floor is a smaller second line. Tabs are ordered by floor, then A–Z within the floor. Zone number is not a sort key. It is only the address sent to the receiver.

```text
Garage   Master   Patio   Study   Balcony   Game
  1        1        1       1        2        2
```

Under the tabs, three compact input buttons in one row: Bluetooth, TV, AirPlay. A source is chosen once, so these stay small.

Volume is pressed much more often, so Vol −, Vol +, and Mute fill the middle of the screen. Those three are the large targets.

The bottom bar is the same on every tab, in thumb reach:

```text
Off    All off
```

Off turns off the zone whose tab is open. All off turns off every zone on the chassis and does not require changing tabs. All off is visually distinct so a miss does not shut the house down.

The phone remembers the open tab and highlights the last input tapped on that tab. The highlight is local. It can be wrong after someone uses a keypad. Off clears the highlight for the open room. All off clears it for every room, so the inputs look unselected, as they do before any input has been tapped. The server does not poll.

Volume − and Volume + repeat while held. The server keeps at most one extra step queued behind the command in flight.

## Appearance

Both color schemes, selected with `prefers-color-scheme`. Tap targets are at least 44 pt. `touch-action: manipulation` so a double tap does not zoom.

Safari home-screen meta tags:

- `viewport` with `width=device-width` and `viewport-fit=cover`
- `mobile-web-app-capable`
- `apple-mobile-web-app-status-bar-style`
- `apple-mobile-web-app-title`
- a web app manifest with `display` set to `standalone`, and an Apple touch icon, so Safari can save a Home Screen or Dock icon

## Config

TOML. Zone and input keys are the protocol numbers. Each zone has a room name and a floor number, `0` or greater. The small line under the room is that number. Tabs sort by floor, then by name.

```toml
listen = "0.0.0.0:8080"
serial = "/dev/ttyUSB0"

[zone.1]
name = "Master"
floor = 1

[zone.2]
name = "Study"
floor = 1

[zone.3]
name = "Garage"
floor = 1

[zone.4]
name = "Game"
floor = 2

[zone.5]
name = "Balcony"
floor = 2

[zone.6]
name = "Patio"
floor = 1

[input]
3 = "Bluetooth"
4 = "TV"
5 = "AirPlay"
```

Reject a zone number outside 1–18 and an input number outside 2–6. Input 1 is the tuner. A zone without `name` or `floor` is a config error.

## HTTP

The page is one HTML document. CSS and the few lines of script are inline. The binary serves that document. There is no JSON API.

Empty `POST`s:

| Action | Path |
| --- | --- |
| Select input | `/zones/{name}/inputs/{name}` |
| Volume up | `/zones/{name}/volume/up` |
| Volume down | `/zones/{name}/volume/down` |
| Mute | `/zones/{name}/mute` |
| Zone off | `/zones/{name}/off` |
| All off | `/off` |

`{name}` is the room name, so `/zones/Master/inputs/AirPlay`. An unknown name is a 404. A successful command is `204`.

`--debug` does not open the serial port. Allowlisted buttons still return `204`, so the page can be tried with the receiver unplugged. An unknown name is still `404`.

The page is reachable by anyone on the LAN. That is accepted for a Pi at home.

## Serial

One task owns the port. Requests are queued behind it. Each command is written, the reply line is read and discarded, and the next command runs. A `FAIL` line is logged. The button still returns `204`, because the UI has no status to correct.

Settings are 38400 8N1, no flow control. Each command ends in a carriage return.

| Button | Bytes |
| --- | --- |
| Input 2–6 | `zsc,<zone>,<code>` |
| Volume up | `zsc,<zone>,12` |
| Volume down | `zsc,<zone>,13` |
| Mute | `zsc,<zone>,11` |
| Zone off | `zsc,<zone>,10` |
| All off | `znt,10,h` |

`zsc` carries the zone number, so the server does not send `znc,4`. Codes are the decimal forms from [protocol.md](protocol.md). If volume up stops playback instead of raising it, the chassis wanted the hex spelling and these four constants are the change.

Mute is a single key. The button toggles. The page does not know the current mute state.

Selecting an input is how a zone turns on. There is no separate power button.

Use a USB-to-RS-232 adapter. The Pi GPIO UART is 3.3 V, and this jack is RS-232. The process user needs read-write access to the device node. Cable wiring is in [protocol.md](protocol.md).

## Program

Rust. One binary, built and run on the Pi. The USB-to-RS-232 adapter is plugged into that machine, so the program is not built anywhere else.

| Crate | Role |
| --- | --- |
| tokio | Runtime and the serial queue |
| axum | HTTP |
| tokio-serial | Port, with the `libudev` feature off. Open the configured path. |
| serde, toml | Config |
| tracing, tracing-subscriber | Logs to stdout |
| thiserror | Error types |

`serde_json`, `nom`, and `rust-embed` are not used. The page is included in the binary as text.

Alpine Linux 3.24 or newer, aarch64, on the Raspberry Pi that has the adapter. Alpine 3.24's `rust` package is 1.96.1, which is new enough for this crate. The binary links against that system's musl.

On the git checkout, run `./version.sh` before copying the tree to the Pi. It writes `src/version.rs` from `git describe --tags --always`. The label ends in `-dirty` when `git status` shows uncommitted or untracked changes. That file is not committed. The binary logs the label, and the page shows it under the room tabs.

```sh
./version.sh
```

On the Pi:

```sh
apk add cargo
cargo build --release
./target/release/niles
```

`cargo` pulls in `rustc` and the system linker. The first release build compiles the dependencies and can need `CARGO_BUILD_JOBS=1` on a Pi with little free RAM.

```sh
./target/release/niles --debug
```

The process listens on the configured address. Without `--debug` it opens the serial device and exits if that open fails.
