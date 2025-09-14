use crate::enums::command::Command;
use serialport::{self, SerialPort};
use sqlx::sqlite::SqlitePool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;

use crate::structs::post::CreatePost;
use crate::structs::post::Post;

fn open_port(
    port_name: &str,
    baud_rate: u32,
) -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
    let mut port = serialport::new(port_name, baud_rate)
        .dtr_on_open(true)
        .timeout(Duration::from_millis(5000)) // Increased to 5 seconds
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;

    let mut discard_buffer = [0u8; 256];
    while let Ok(bytes_read) = port.read(&mut discard_buffer) {
        if bytes_read == 0 {
            break;
        }
        println!("Cleared {} bytes from input buffer", bytes_read);
    }

    println!("Arduino should be ready now");
    Ok(port)
}

fn send_and_read(
    port: &mut Box<dyn SerialPort>,
    data_to_send: &[u8],
    debug: bool,
) -> Result<[u8; 256], Box<dyn std::error::Error>> {
    let mut response_buffer: [u8; 256] = [0u8; 256];
    let mut total_bytes_read = 0;
    port.write(&data_to_send)?;
    port.flush()?; // Ensure data is actually sent

    match port.read(&mut response_buffer[total_bytes_read..]) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                total_bytes_read += bytes_read;

                if debug {
                    println!("Received {} bytes in this attempt", bytes_read);

                    print!("Hex: ");
                    for i in 0..total_bytes_read {
                        print!("0x{:02X} ", response_buffer[i]);
                    }
                    println!();

                    print!("ASCII: ");
                    for i in 0..total_bytes_read {
                        let byte = response_buffer[i];
                        if byte >= 32 && byte <= 126 {
                            print!("{}", byte as char);
                        } else {
                            print!(".");
                        }
                    }
                    println!();
                }
                // Check if there might be more data coming
                // std::thread::sleep(Duration::from_millis(100));
                return Ok(response_buffer);
            } else {
                println!("No data in this attempt");
                return Err("No data received".into());
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            println!("Timeout");
            return Err("Read timeout".into());
        }
        Err(e) => {
            println!("Error reading response: {}", e);
            return Err(e.into());
        }
    }
}

fn read_peak(port: &mut Box<dyn SerialPort>) -> u32 {
    let data_to_send = [0x0D]; // 0E
    let response_buffer = send_and_read(port, &data_to_send, false).unwrap();

    let _lap_id = response_buffer[0];
    // ms_val is a 16 bit value
    let _ms_val = (response_buffer[1] as u16 * 256 + response_buffer[2] as u16) as u32;
    let rssi = response_buffer[3];

    // println!("lap_id: {}, ms_val: {}, rssi: {}", lap_id, ms_val, rssi);
    rssi as u32
}

pub async fn worker_task(mut command_receiver: mpsc::Receiver<Command>, db_pool: SqlitePool) {
    let mut counter: u32 = 0;
    let mut is_listening = false;
    let port_name = "/dev/cu.usbserial-11230";
    let baud_rate = 115200;
    let port = Arc::new(Mutex::new(open_port(port_name, baud_rate).unwrap()));

    let mut last_peak = 0;
    let mut last_peak_time = std::time::Instant::now();
    let threshold = 3;
    loop {
        tokio::select! {
                Some(command) = command_receiver.recv() => {
                match command {
                    Command::Increment => {
                        if !is_listening {
                            is_listening = true;
                        } else {
                            is_listening = false;
                        }
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
                }
            },
            Ok(peak) = {
                let port_clone = Arc::clone(&port);
                task::spawn_blocking(move || {
                    let mut port_guard = port_clone.lock().unwrap();
                    read_peak(&mut *port_guard)
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
                    // let now = Utc::now();
                    println!(
                        "Peak: {} during {} seconds",
                        last_peak, duration
                    );
                    last_peak = peak;
                    last_peak_time = std::time::Instant::now();
                }
                if last_peak_time.elapsed().as_secs_f64() > 1.0 {
                    let duration = last_peak_time.elapsed().as_secs_f64();
                    println!(
                        "Peak: {} during {} seconds",
                        last_peak, duration
                    );
                    last_peak_time = std::time::Instant::now();
                }

            }
        }
    }
}
