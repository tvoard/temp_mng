use anyhow::Context;
use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, SqlitePool};
use std::{error::Error, path::Path, result::Result};

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    tracing::info!("Connecting to database at `{}`", database_url);

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn migrate_db(pool: &SqlitePool, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    tracing::info!("Running database migrations from `{}`", migration_dir);

    Migrator::new(Path::new(&migration_dir))
        .await
        .context("Failed to initialize database migrator")?
        .run(pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)
}
