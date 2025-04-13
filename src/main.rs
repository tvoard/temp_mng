use actix_cors::Cors;
use actix_web::{http, middleware::{Condition, Logger}, web, App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use sqlx::SqlitePool;
use std::env;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod config;
mod db;
mod dtos;
mod errors;
mod handlers;
mod middleware;
mod models;
mod utils;

use crate::errors::AppError;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
// AppError 임포트

// --- API 문서 설정 (Utoipa) ---
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health_check,
        handlers::auth::login,
        handlers::auth::get_current_user,
        handlers::user::create_user,
        handlers::user::get_users,
        // 다른 핸들러 함수들 추가...
    ),
    components(
        schemas(
            // DTO 스키마들 추가
            dtos::auth::LoginRequest, dtos::auth::LoginResponse, dtos::auth::CurrentUserResponse,
            dtos::user::CreateUserRequest, dtos::user::UpdateUserRequest, dtos::user::UserResponse,
            dtos::user_type::UserTypeResponse, // 예시
            dtos::permission::PermissionResponse, // 예시
            dtos::menu::MenuResponse, // 예시
            // 에러 응답 스키마 (AppError의 ErrorResponse 구조 반영 필요 - 현재는 AppError 직접 사용)
            errors::AppError,
            models::UserType, models::AdminUser, models::Permission, models::MenuItem // 모델도 추가 가능
        ),
        security_schemes(
            // JWT 보안 스키마 정의
            (name = "bearer_auth", scheme = bearer, bearer_format = "JWT", type = http)
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication operations"),
        (name = "User Management", description = "Admin user management"),
        (name = "User Type Management", description = "User role/type management"),
        (name = "Permission Management", description = "Permission definition management"),
        (name = "Menu Management", description = "Menu item management"),
        (name = "Health", description = "Server health check")
    ),
    servers(
        (url = "/api/v1", description = "Development server")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // 로깅/트레이싱 초기화 (RUST_LOG 또는 TRACING_LEVEL 환경 변수 사용)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")); // 기본 레벨 info
    FmtSubscriber::builder().with_env_filter(filter).init();

    let cfg = config::Config::from_env()?;
    let pool = db::create_pool(&cfg.database_url).await?;

    // 데이터베이스 마이그레이션 실행
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    tracing::info!("Database migrations completed.");

    tracing::info!("Starting server at http://{}", cfg.server_addr);

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        // CORS 설정 (개발 중에는 관대하게, 프로덕션에서는 엄격하게)
        let cors = Cors::default()
            .allow_any_origin() // 개발용, 프로덕션에서는 특정 도메인 지정: .allowed_origin("http://your-frontend.com")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials() // 필요시 true
            .max_age(3600);

        App::new()
            // 상태 공유: 데이터베이스 풀, 설정값
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cfg.clone()))
            // 미들웨어 등록 (순서 중요)
            .wrap(cors) // CORS 먼저
            .wrap(Condition::new(true, Logger::default())) // 로깅 (tracing으로 대체 가능)
            // !!! 인증 미들웨어는 API 범위 내에서 적용 !!!
            // API 문서 서빙 (Swagger UI)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            // API 라우팅 설정 (v1 네임스페이스)
            .service(
                web::scope("/api/v1")
                    .wrap(middleware::auth::Authentication)
                    .service(handlers::health::health_check) // 인증 불필요 (위치 조정 또는 미들웨어에서 경로 예외처리)
                    .configure(handlers::auth::configure_routes)
                    .configure(handlers::user::configure_routes)
                    .configure(handlers::user_type::configure_routes) // 추가
                    .configure(handlers::permission::configure_routes) // 추가
                    .configure(handlers::menu::configure_routes), // 추가
            )
    })
    .bind(cfg.server_addr.clone())? // bind 전에 clone
    .run()
    .await?;

    Ok(())
}
