use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct Race {
    pub id: i32,
    pub start_time: Vec<u8>,
    pub end_time: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRace {
    pub start_time: Vec<u8>,
}
