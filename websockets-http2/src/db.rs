use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::ConnectOptions;
use sqlx::SqlitePool;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_options: SqliteConnectOptions = SqliteConnectOptions::new()
        .filename("test.db")
        .create_if_missing(true)
        .disable_statement_logging()
        .to_owned();

    let pool = SqlitePoolOptions::new().connect_with(db_options).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                content NOT NULL
            );",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
