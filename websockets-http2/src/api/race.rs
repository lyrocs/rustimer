use crate::enums::command::Command;
use crate::structs::state::AppState;
use axum::{extract::State, http::StatusCode, Json};

pub async fn start_race(State(state): State<AppState>) -> StatusCode {
    // create race in db
    let bytes = std::time::Instant::now().elapsed().as_nanos().to_be_bytes();
    let race = sqlx::query_as::<_, crate::structs::race::Race>(
        "INSERT INTO race (start_time) VALUES (?) RETURNING id, start_time, end_time",
    )
    .bind(&bytes[..])
    .fetch_one(&state.db)
    .await;

    state
        .command_sender
        .send(Command::StartRace {
            time: std::time::Instant::now(),
            race_id: race.unwrap().id,
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

pub async fn debug(State(state): State<AppState>) -> Json<Vec<crate::structs::node::NodeJson>> {
    // fetch all nodes data
    let nodes = sqlx::query_as::<_, crate::structs::node::Node>("SELECT * FROM node")
        .fetch_all(&state.db)
        .await;

    Json(
        nodes
            .unwrap()
            .into_iter()
            .map(|node| crate::structs::node::NodeJson {
                id: node.id,
                peak: node.peak,
                time: u128::from_be_bytes(node.time.try_into().unwrap()),
                duration: node.duration,
                race_id: node.race_id,
            })
            .collect(),
    )
}
