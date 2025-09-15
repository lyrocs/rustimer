use crate::enums::command::Command;
use sqlx::sqlite::SqlitePool;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task;

use crate::node;
use crate::structs::post::CreatePost;
use crate::structs::post::Post;

pub async fn worker_task(mut command_receiver: mpsc::Receiver<Command>, db_pool: SqlitePool) {
    let mut counter: u32 = 0;
    let mut is_listening = false;
    let port_name = "/dev/cu.usbserial-11230";
    let baud_rate = 115200;
    let port = Arc::new(Mutex::new(node::open_port(port_name, baud_rate).unwrap()));

    let mut last_peak = 0;
    let mut race_start_time = std::time::Instant::now();
    let mut last_peak_time = std::time::Instant::now();
    let threshold = 3;
    loop {
        tokio::select! {
                Some(command) = command_receiver.recv() => {
                match command {
                    Command::Increment => {
                        counter += 1;
                        println!("Worker: Counter incremented to {}", counter);

                        let new_post = CreatePost {
                            title: format!("Post {}", counter),
                            content: format!("This is the content of post {}", counter),
                        };

                        let post = sqlx::query_as::<_, Post>(
                            "INSERT INTO posts (title, content) VALUES (?, ?) RETURNING id, title, content",
                        )
                        .bind(new_post.title)
                        .bind(new_post.content)
                        .fetch_one(&db_pool)
                        .await;

                        match post {
                            Ok(p) => println!("Worker: Created new post with id {}", p.id),
                            Err(e) => eprintln!("Worker: Failed to create post: {}", e),
                        }
                    }
                    Command::GetCount { respond_to } => {
                        println!(
                            "Worker: Sending current count ({}) back to handler",
                            counter
                        );
                        let _ = respond_to.send(counter);
                    }
                    Command::StartRace { time } => {
                        is_listening = true;
                        race_start_time = time;
                    }
                    Command::StopRace => {
                        is_listening = false;
                    }
                }
            },
            Ok(peak) = {
                let port_clone = Arc::clone(&port);
                task::spawn_blocking(move || {
                    let mut port_guard = port_clone.lock().unwrap();
                    node::read_peak(&mut *port_guard)
                })
            }, if is_listening => {
                // let peak = peak_result.unwrap();
                if last_peak == 0 {
                    last_peak = peak;
                    last_peak_time = std::time::Instant::now();
                }
                if peak < last_peak.saturating_sub(threshold) || peak > last_peak.saturating_add(threshold)
                {
                    let duration = last_peak_time.elapsed().as_secs_f64();
                    let start_time = race_start_time.elapsed().as_nanos();
                    // let now = Utc::now();
                    println!(
                        "Peak: {} during {} seconds, {} nanoseconds",
                        last_peak, duration, start_time
                    );
                    last_peak = peak;
                    last_peak_time = std::time::Instant::now();
                }
                if last_peak_time.elapsed().as_secs_f64() > 1.0 {
                    let duration = last_peak_time.elapsed().as_secs_f64();
                    let start_time = race_start_time.elapsed().as_nanos();
                    println!(
                        "Peak: {} during {} seconds, {} nanoseconds",
                        last_peak, duration, start_time
                    );
                    last_peak_time = std::time::Instant::now();
                }

            }
        }
    }
}
