use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use tokio::sync::mpsc::UnboundedSender;

use crate::config::Config;
use crate::serial::{Command, MUTE, VOL_DOWN, VOL_UP, ZONE_OFF};

const PAGE: &str = include_str!("page.html");
const MANIFEST: &str = include_str!("manifest.webmanifest");
const ICON: &[u8] = include_bytes!("icon.png");

#[derive(Clone)]
pub struct App {
    inner: Arc<Inner>,
}

struct Inner {
    page: String,
    zones: HashMap<String, u8>,
    inputs: HashMap<String, u8>,
    tx: UnboundedSender<Command>,
}

pub fn app(config: &Config, debug: bool, tx: UnboundedSender<Command>) -> App {
    let zones = config
        .zones
        .iter()
        .map(|zone| (zone.name.clone(), zone.number))
        .collect();
    let inputs = config
        .inputs
        .iter()
        .map(|input| (input.name.clone(), input.number))
        .collect();
    App {
        inner: Arc::new(Inner {
            page: render(config, debug),
            zones,
            inputs,
            tx,
        }),
    }
}

pub fn router(app: App) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/manifest.webmanifest", get(manifest))
        .route("/icon.png", get(icon))
        .route("/off", post(all_off))
        .route("/zones/{zone}/inputs/{input}", post(select_input))
        .route("/zones/{zone}/volume/up", post(volume_up))
        .route("/zones/{zone}/volume/down", post(volume_down))
        .route("/zones/{zone}/mute", post(mute))
        .route("/zones/{zone}/off", post(zone_off))
        .with_state(app)
}

fn render(config: &Config, debug: bool) -> String {
    let mut tabs = String::new();
    for zone in &config.zones {
        let name = escape(&zone.name);
        tabs.push_str(&format!(
            "<button type=\"button\" class=\"tab\" data-zone=\"{name}\"><span class=\"name\">{name}</span><span class=\"floor\">{}</span></button>\n",
            zone.floor,
        ));
    }
    let mut inputs = String::new();
    for input in &config.inputs {
        let name = escape(&input.name);
        inputs.push_str(&format!(
            "<button type=\"button\" class=\"input\" data-input=\"{name}\">{name}</button>\n"
        ));
    }
    let debug_line = if debug {
        format!(
            "<p class=\"debug\">{} · Debug · serial off</p>",
            escape(&crate::version::label())
        )
    } else {
        format!(
            "<p class=\"debug\">{}</p>",
            escape(&crate::version::label())
        )
    };
    PAGE.replace("<!--TABS-->", &tabs)
        .replace("<!--INPUTS-->", &inputs)
        .replace("<!--DEBUG-->", &debug_line)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

async fn manifest() -> impl IntoResponse {
    (
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/manifest+json",
            ),
            (axum::http::header::CACHE_CONTROL, "no-store"),
        ],
        MANIFEST,
    )
}

async fn icon() -> impl IntoResponse {
    (
        [
            (axum::http::header::CONTENT_TYPE, "image/png"),
            (axum::http::header::CACHE_CONTROL, "no-store"),
        ],
        ICON,
    )
}

async fn index(State(app): State<App>) -> impl IntoResponse {
    (
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Html(app.inner.page.clone()),
    )
}

async fn all_off(State(app): State<App>) -> StatusCode {
    accept(&app, Command::AllOff)
}

async fn select_input(
    State(app): State<App>,
    Path((zone, input)): Path<(String, String)>,
) -> StatusCode {
    let Some(zone) = app.inner.zones.get(&zone).copied() else {
        return StatusCode::NOT_FOUND;
    };
    let Some(code) = app.inner.inputs.get(&input).copied() else {
        return StatusCode::NOT_FOUND;
    };
    accept(&app, Command::Zone { zone, code })
}

async fn volume_up(State(app): State<App>, Path(zone): Path<String>) -> StatusCode {
    zone_command(&app, &zone, VOL_UP)
}

async fn volume_down(State(app): State<App>, Path(zone): Path<String>) -> StatusCode {
    zone_command(&app, &zone, VOL_DOWN)
}

async fn mute(State(app): State<App>, Path(zone): Path<String>) -> StatusCode {
    zone_command(&app, &zone, MUTE)
}

async fn zone_off(State(app): State<App>, Path(zone): Path<String>) -> StatusCode {
    zone_command(&app, &zone, ZONE_OFF)
}

fn zone_command(app: &App, name: &str, code: u8) -> StatusCode {
    let Some(zone) = app.inner.zones.get(name).copied() else {
        return StatusCode::NOT_FOUND;
    };
    accept(app, Command::Zone { zone, code })
}

fn accept(app: &App, command: Command) -> StatusCode {
    match app.inner.tx.send(command) {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::path::Path;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::mpsc;

    use super::*;
    use crate::config::Config;
    use crate::serial;

    fn sample() -> Config {
        Config::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("niles.toml")).unwrap()
    }

    #[test]
    fn page_lists_rooms_in_tab_order() {
        let page = render(&sample(), true);
        let garage = page.find("Garage").unwrap();
        let master = page.find("Master").unwrap();
        let patio = page.find("Patio").unwrap();
        let study = page.find("Study").unwrap();
        let balcony = page.find("Balcony").unwrap();
        let game = page.find("Game").unwrap();
        assert!(garage < master && master < patio && patio < study);
        assert!(study < balcony && balcony < game);
        assert!(page.contains("viewport-fit=cover"));
        assert!(page.contains("mobile-web-app-capable"));
        assert!(page.contains("apple-mobile-web-app-status-bar-style"));
        assert!(page.contains("apple-mobile-web-app-title"));
        assert!(page.contains("rel=\"manifest\""));
        assert!(page.contains("apple-touch-icon"));
        assert!(page.contains("prefers-color-scheme"));
        assert!(page.contains("100dvh"));
        assert!(page.contains("touch-action: manipulation"));
        let label = crate::version::label();
        assert!(page.contains(&label));
        assert!(page.contains("Debug · serial off"));
        assert!(render(&sample(), false).contains(&label));
        let inputs_at = page.find("class=\"inputs\"").unwrap();
        let volume_at = page.find("id=\"vol-down\"").unwrap();
        let all_off_at = page.find("id=\"all-off\"").unwrap();
        assert!(inputs_at < volume_at && volume_at < all_off_at);
        assert!(page.contains("All off"));
        assert!(!page.contains("syc"));
        assert!(!render(&sample(), false).contains("Debug · serial off"));
    }

    #[tokio::test]
    async fn posts_return_204_for_known_names_and_404_otherwise() {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(serial::run(rx, None));
        let app = app(&sample(), true, tx);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router(app)).await.unwrap();
        });

        let (status, body) = exchange(addr, "GET", "/").await;
        assert_eq!(status, 200);
        assert!(body.contains("AirPlay"));

        let (status, body) = exchange(addr, "GET", "/manifest.webmanifest").await;
        assert_eq!(status, 200);
        assert!(body.contains("application/manifest+json"));
        assert!(body.contains("\"display\": \"standalone\""));
        let (status, body) = exchange(addr, "GET", "/icon.png").await;
        assert_eq!(status, 200);
        assert!(body.contains("image/png"));

        assert_eq!(
            exchange(addr, "POST", "/zones/Master/inputs/AirPlay")
                .await
                .0,
            204
        );
        assert_eq!(
            exchange(addr, "POST", "/zones/Garage/volume/up").await.0,
            204
        );
        assert_eq!(
            exchange(addr, "POST", "/zones/Game/volume/down").await.0,
            204
        );
        assert_eq!(exchange(addr, "POST", "/zones/Study/mute").await.0, 204);
        assert_eq!(exchange(addr, "POST", "/zones/Patio/off").await.0, 204);
        assert_eq!(exchange(addr, "POST", "/off").await.0, 204);
        assert_eq!(exchange(addr, "POST", "/zones/Attic/off").await.0, 404);
        assert_eq!(
            exchange(addr, "POST", "/zones/Master/inputs/Radio").await.0,
            404
        );
    }

    async fn exchange(addr: SocketAddr, method: &str, path: &str) -> (u16, String) {
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let text = String::from_utf8_lossy(&buf).into_owned();
        let status = text.split_whitespace().nth(1).unwrap().parse().unwrap();
        (status, text)
    }
}
