use crate::enums::command::Command;
use crate::structs::state::AppState;
use axum::{extract::State, http::StatusCode};

pub async fn start_race(State(state): State<AppState>) -> StatusCode {
    state
        .command_sender
        .send(Command::StartRace {
            time: std::time::Instant::now(),
        })
        .await
        .unwrap();
    StatusCode::OK
}

pub async fn stop_race(State(state): State<AppState>) -> StatusCode {
    let command = Command::StopRace;

    state.command_sender.send(command).await.unwrap();

    StatusCode::OK
}
