use crate::structs::post::CreatePost;
use crate::structs::post::Post;
use crate::structs::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};

pub async fn create_post(
    State(state): State<AppState>,
    Json(payload): Json<CreatePost>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as::<_, Post>(
        "INSERT INTO posts (title, content) VALUES (?, ?) RETURNING id, title, content",
    )
    .bind(payload.title)
    .bind(payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(post))
}

pub async fn get_posts(State(state): State<AppState>) -> Result<Json<Vec<Post>>, StatusCode> {
    let posts = sqlx::query_as::<_, Post>("SELECT id, title, content FROM posts")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch posts: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(posts))
}
