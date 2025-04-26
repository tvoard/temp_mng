use std::env;

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use config::{db, env::Env};
use dotenv::from_filename;

mod config;
mod dto;
mod errors;
mod handlers;
mod middleware;
mod models;
mod services;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 환경 파일 로드
    let env_file = match env::var("PROFILE").as_deref() {
        Ok("prod") => ".env.prod",
        Ok("dev") => ".env.dev",
        _ => ".env", // 기본값
    };
    from_filename(env_file).ok();

    // 2. 로깅 초기화
    tracing_subscriber::fmt::init();

    // 3. 환경 변수 로드
    let env = Env::from_env()?;

    // 4. 데이터베이스 연결
    let pool = db::create_pool(&env.database_url).await?;

    // 5. 데이터베이스 초기화
    let _ = db::migrate_db(&pool, &env.migration_dir).await;

    // 6. 서버 시작 시간 초기화
    services::health::initialize_server_start_time();

    // 7. HTTP 서버 실행
    let server_addr = env.server_addr.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(env.clone()))
            .app_data(web::Data::new(pool.clone()))
            .configure(handlers::configure)
    })
    .bind(&server_addr)?
    .run()
    .await?;

    Ok(())
}
