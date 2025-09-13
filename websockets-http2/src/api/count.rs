use crate::enums::command::Command;
use crate::structs::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use tokio::sync::oneshot;

pub async fn increment_handler(State(state): State<AppState>) -> StatusCode {
    state.command_sender.send(Command::Increment).await.unwrap();
    StatusCode::OK
}

pub async fn get_count_handler(State(state): State<AppState>) -> (StatusCode, Json<u32>) {
    let (response_sender, response_receiver) = oneshot::channel();

    let command = Command::GetCount {
        respond_to: response_sender,
    };

    state.command_sender.send(command).await.unwrap();

    match response_receiver.await {
        Ok(count) => (StatusCode::OK, Json(count)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(0)),
    }
}
