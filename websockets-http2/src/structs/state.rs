use crate::enums::command::Command;
use sqlx::sqlite::SqlitePool;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AppState {
    pub command_sender: mpsc::Sender<Command>,
    pub tx: broadcast::Sender<String>,
    pub db: SqlitePool,
}
