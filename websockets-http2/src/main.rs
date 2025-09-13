use axum::{
    extract::{
        ws::{self, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    http::Version,
    response::Json,
    routing::any,
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::broadcast;
use tokio::sync::{mpsc, oneshot};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
enum Command {
    Increment,
    GetCount { respond_to: oneshot::Sender<u32> },
}

#[derive(Clone)]
struct AppState {
    command_sender: mpsc::Sender<Command>,
    tx: broadcast::Sender<String>,
}

async fn worker_task(mut command_receiver: mpsc::Receiver<Command>) {
    let mut counter: u32 = 0;

    while let Some(command) = command_receiver.recv().await {
        match command {
            Command::Increment => {
                counter += 1;
                println!("Worker: Counter incremented to {}", counter);
            }
            Command::GetCount { respond_to } => {
                println!(
                    "Worker: Sending current count ({}) back to handler",
                    counter
                );
                let _ = respond_to.send(counter);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (command_sender, command_receiver) = mpsc::channel(32);

    tokio::spawn(worker_task(command_receiver));

    let (tx, _) = broadcast::channel(100);

    let app_state = AppState {
        command_sender,
        tx: tx.clone(),
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("self_signed_certs")
            .join("cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("self_signed_certs")
            .join("key.pem"),
    )
    .await
    .unwrap();

    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", any(ws_handler))
        .route("/ping", any(ping_handler))
        .route("/increment", get(increment_handler))
        .route("/get_count", get(get_count_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    let mut server = axum_server::bind_rustls(addr, config);

    // IMPORTANT: This is required to advertise our support for HTTP/2 websockets to the client.
    // If you use axum::serve, it is enabled by default.
    server.http_builder().http2().enable_connect_protocol();

    server.serve(app.into_make_service()).await.unwrap();
}

async fn increment_handler(State(state): State<AppState>) -> StatusCode {
    state.command_sender.send(Command::Increment).await.unwrap();
    StatusCode::OK
}

async fn get_count_handler(State(state): State<AppState>) -> (StatusCode, Json<u32>) {
    let (response_sender, response_receiver) = oneshot::channel();

    let command = Command::GetCount {
        respond_to: response_sender,
    };
    let command = Command::GetCount {
        respond_to: response_sender,
    };

    state.command_sender.send(command).await.unwrap();

    match response_receiver.await {
        Ok(count) => (StatusCode::OK, Json(count)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(0)),
    }
}

async fn ping_handler() -> &'static str {
    "pong"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    version: Version,
    State(state): State<AppState>,
) -> axum::response::Response {
    let sender = state.tx;
    tracing::debug!("accepted a WebSocket using {version:?}");
    let mut receiver = sender.subscribe();
    ws.on_upgrade(|mut ws| async move {
        loop {
            tokio::select! {
                res = ws.recv() => {
                    match res {
                        Some(Ok(ws::Message::Text(s))) => {
                            let _ = sender.send(s.to_string());
                        }
                        Some(Ok(_)) => {}
                        Some(Err(e)) => tracing::debug!("client disconnected abruptly: {e}"),
                        None => break,
                    }
                }
                res = receiver.recv() => {
                    match res {
                        Ok(msg) => if let Err(e) = ws.send(ws::Message::Text(msg.into())).await {
                            tracing::debug!("client disconnected abruptly: {e}");
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
    })
}
