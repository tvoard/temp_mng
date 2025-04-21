use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpServer};
use anyhow::{Context, Result};
use dotenv::dotenv;
use sqlx::migrate::Migrator;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod db;
mod dto;
mod errors;
mod handler;
mod middleware;
mod models;
mod util;

// --- API 문서 설정 (Utoipa) ---
#[derive(OpenApi)]
#[openapi(
    paths(
        handler::health::health_check,
        handler::auth::login,
        handler::auth::get_current_user,
        handler::user::create_user,
        handler::user::get_users,
        handler::user::get_user_by_id,
        handler::user_type::create_user_type,
        handler::user_type::get_user_types,
        handler::user_type::get_user_type_by_id,
        handler::user_type::update_user_type,
        handler::user_type::delete_user_type,
        handler::user_type::add_permission_to_user_type,
        handler::user_type::remove_permission_from_user_type,
        handler::permission::create_permission,
        handler::permission::get_permissions,
        handler::permission::get_permission_by_id,
        handler::menu::create_menu,
        handler::menu::get_menus,
    ),
    components(
        schemas(
            dto::auth::LoginRequest,
            dto::auth::LoginResponse,
            dto::auth::CurrentUserResponse,
            dto::user::CreateUserRequest,
            dto::user::UpdateUserRequest,
            dto::user::UserResponse,
            dto::user_type::UserTypeResponse,
            dto::permission::PermissionResponse,
            errors::ErrorResponse,
            models::UserType,
            models::AdminUser,
            models::Permission,
            models::MenuItem
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

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing::info!("DATABASE_URL: {:?}", env::var("DATABASE_URL"));
    tracing::info!("SERVER_ADDR: {:?}", env::var("SERVER_ADDR"));

    // 로깅/트레이싱 초기화 (RUST_LOG 또는 TRACING_LEVEL 환경 변수 사용)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")); // 기본 레벨 info
    FmtSubscriber::builder().with_env_filter(filter).init();

    let cfg = Arc::new(config::Config::from_env()?);
    let pool = db::create_pool(&cfg.database_url)
        .await
        .context("Failed to connect to the database")?;

    // 데이터베이스 마이그레이션 실행
    tracing::info!("Running database migrations...");
    let migration_dir = env::var("MIGRATION_DIR").unwrap_or_else(|_| "./db".to_string());
    let migrator = Migrator::new(Path::new(&migration_dir))
        .await
        .context("Failed to initialize database migrator")?;
    if let Err(e) = migrator.run(&pool).await {
        tracing::error!("Failed to run database migrations: {:?}", e);
        return Err(e.into());
    }
    tracing::info!("Database migrations completed.");

    let cloned_pool = pool.clone();
    let cloned_cfg = Arc::clone(&cfg);

    let server_addr = Arc::new(cfg.server_addr.clone());
    tracing::info!("Starting server at http://{}", server_addr);

    let openapi = ApiDoc::openapi();
    // handler::health::initialize_server_start_time();

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
            .app_data(web::Data::new(cloned_pool.clone()))
            .app_data(web::Data::new(cloned_cfg.clone()))
            // 미들웨어 등록 (순서 중요)
            .wrap(cors) // CORS 먼저
            .wrap(Logger::default())
            // API 문서 서빙 (Swagger UI)
            .service(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi.clone()))
            // API 라우팅 설정 (v1 네임스페이스)
            .service(
                web::scope("/api/v1")
                    .wrap(middleware::auth::Authentication)
                    .service(handler::health::health_check) // 인증 불필요 (위치 조정 또는 미들웨어에서 경로 예외처리)
                    .service(handler::auth::configure_routes())
                    .service(handler::user::configure_routes())
                    .service(handler::user_type::configure_routes())
                    .service(handler::permission::configure_routes())
                    .service(handler::menu::configure_routes()),
            )
    })
    .bind(server_addr.as_str())?
    .run()
    .await?;

    Ok(())
}
