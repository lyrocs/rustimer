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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS race (
                id INTEGER PRIMARY KEY,
                start_time TEXT NOT NULL,
                end_time TEXT NULL
            );",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS node (
                id INTEGER PRIMARY KEY,
                peak INTEGER NOT NULL,
                time BLOB NOT NULL,
                duration INTEGER NOT NULL,
                race_id INTEGER NOT NULL,
                FOREIGN KEY (race_id) REFERENCES race (id)
            );",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
