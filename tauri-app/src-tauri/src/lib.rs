use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest;

#[derive(Deserialize, Serialize)]
struct Rssi {
    id: i32,
    peak: i32,
    time: f64,
    duration: f64,
    race_id: i32,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn greet(name: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let res = client
        .get("https://localhost:3000/ping")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{:?}", res);
    // let res = res.json().await;

    Ok(format!("Hello, {}! You've been greeted from Rust!", res))
}

#[tauri::command]
async fn get_rssi() -> Result<Vec<Rssi>, String> {
    println!("get_rssi");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let res = client
        .get("https://localhost:3000/debug")
        .send()
        .await
        .unwrap()
        .json::<Vec<Rssi>>()
        .await
        .unwrap();

    println!("{}", res.len());

    Ok(res)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_rssi])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
