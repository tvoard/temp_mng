use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Env {
    pub database_url: String,
    pub migration_dir: String,
    pub server_addr: String,
    pub jwt_secret: String,
    pub jwt_expires_in_seconds: i64,
}

impl Env {
    pub fn from_env() -> Result<Self> {
        tracing::info!("Loading environment variables...");

        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            migration_dir: env::var("MIGRATION_DIR").unwrap_or_else(|_| "./db".to_string()),
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            jwt_secret: env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
            jwt_expires_in_seconds: env::var("JWT_EXPIRES_IN_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse::<i64>()
                .context("JWT_EXPIRES_IN_SECONDS must be a valid number")?,
        })
    }
}
