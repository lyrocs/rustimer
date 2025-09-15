use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct Node {
    pub id: i32,
    pub peak: u32,
    pub time: Vec<u8>,
    pub duration: f64,
    pub race_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateNode {
    pub peak: u32,
    pub time: Vec<u8>,
    pub duration: f64,
    pub race_id: i32,
}

#[derive(Debug, Serialize)]
pub struct NodeJson {
    pub id: i32,
    pub peak: u32,
    pub time: u128,
    pub duration: f64,
    pub race_id: i32,
}
