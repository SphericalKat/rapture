mod sfu;
mod signalling;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use once_cell::sync::Lazy;
use sfu::SFU;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static SFU: Lazy<Mutex<SFU>> = Lazy::new(|| Mutex::new(SFU::new()));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rapture=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // build our application with some routes
    let app = Router::new()
        // top since it matches all routes
        .route("/ws/:session_id", get(ws_handler))
        // logging so we can see whats going on
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default()));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn ws_handler(Path(session_id): Path<String>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, session_id))
}

async fn handle_socket(mut socket: WebSocket, session_id: String) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => signalling::parse_message(&mut socket, t, &session_id).await,
                Message::Close(_) => {
                    tracing::debug!("client disconnected");
                    return;
                }
                _ => {}
            }
        } else {
            tracing::debug!("client disconnected");
            return;
        }
    }
}
