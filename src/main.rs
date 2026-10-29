mod config;
mod http;
mod serial;
mod version;

use std::path::PathBuf;
use std::process::ExitCode;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> ExitCode {
    let (debug, path) = match parse_args() {
        Ok(Args::Help) => {
            println!("{}", help());
            return ExitCode::SUCCESS;
        }
        Ok(Args::Run { debug, path }) => (debug, path),
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    tracing::info!(version = %version::label(), "niles");

    let config = match config::Config::load(&path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("niles: {err}");
            return ExitCode::from(1);
        }
    };

    let port = if debug {
        tracing::info!("debug: serial port left closed");
        None
    } else {
        match serial::open(&config.serial) {
            Ok(port) => {
                tracing::info!(path = %config.serial, "serial port open");
                Some(port)
            }
            Err(err) => {
                eprintln!("niles: open {}: {err}", config.serial);
                return ExitCode::from(1);
            }
        }
    };

    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(serial::run(rx, port));

    let app = http::app(&config, debug, tx);
    let listener = match tokio::net::TcpListener::bind(config.listen).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("niles: listen {}: {err}", config.listen);
            return ExitCode::from(1);
        }
    };
    let debug_mode = debug;
    tracing::info!(address = %config.listen, debug_mode, "listening");
    if let Err(err) = axum::serve(listener, http::router(app)).await {
        eprintln!("niles: {err}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

enum Args {
    Help,
    Run { debug: bool, path: PathBuf },
}

fn parse_args() -> Result<Args, String> {
    let mut debug = false;
    let mut path = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--debug" => debug = true,
            "-h" | "--help" => return Ok(Args::Help),
            other if other.starts_with('-') => {
                return Err(format!("niles: unknown argument {other}\n\n{}", help()));
            }
            other => {
                if path.is_some() {
                    return Err(format!("niles: unexpected argument {other}\n\n{}", help()));
                }
                path = Some(PathBuf::from(other));
            }
        }
    }
    Ok(Args::Run {
        debug,
        path: path.unwrap_or_else(|| PathBuf::from("niles.toml")),
    })
}

fn help() -> String {
    "\
niles [--debug] [config]

  --debug   Do not open the serial port. Known buttons still return 204.
  config    TOML file to read. Default: niles.toml"
        .to_string()
}
