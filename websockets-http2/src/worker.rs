use crate::enums::command::Command;
use sqlx::sqlite::SqlitePool;
use tokio::sync::mpsc;

use crate::structs::post::CreatePost;
use crate::structs::post::Post;

pub async fn worker_task(mut command_receiver: mpsc::Receiver<Command>, db_pool: SqlitePool) {
    let mut counter: u32 = 0;

    while let Some(command) = command_receiver.recv().await {
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
        }
    }
}
