use axum::{routing::any, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod worker;
use crate::worker::worker_task;
mod api;
mod enums;
mod structs;
use crate::api::count;
use crate::api::post;
use crate::api::race;
mod websocket;
use crate::structs::state::AppState;
use crate::websocket::ws_handler;
mod db;
use crate::db::init_db;

#[tokio::main]
async fn main() {
    let (command_sender, command_receiver) = mpsc::channel(32);
    let (tx, _) = broadcast::channel(100);
    let db = init_db().await.unwrap();

    let app_state = AppState {
        command_sender,
        tx,
        db,
    };

    let db_pool = app_state.db.clone();
    tokio::spawn(worker_task(command_receiver, db_pool));

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
        .route("/increment", get(count::increment_handler))
        .route("/get_count", get(count::get_count_handler))
        .route("/start_race", get(race::start_race))
        .route("/stop_race", get(race::stop_race))
        .route("/posts", get(post::get_posts).post(post::create_post))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    let mut server = axum_server::bind_rustls(addr, config);

    // IMPORTANT: This is required to advertise our support for HTTP/2 websockets to the client.
    // If you use axum::serve, it is enabled by default.
    server.http_builder().http2().enable_connect_protocol();

    server.serve(app.into_make_service()).await.unwrap();
}

async fn ping_handler() -> &'static str {
    "pong"
}
