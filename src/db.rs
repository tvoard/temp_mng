use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(10) // 풀 크기 조정 가능
        .connect(database_url)
        .await?;
    Ok(pool)
}
