// src/handlers/auth.rs
use crate::{
    config::Config,
    dtos::auth::{CurrentUserResponse, LoginRequest, LoginResponse},
    errors::AppError,
    middleware::auth::AuthenticatedUser, // 현재 사용자 정보 Extractor
    models::AdminUser,
    utils::{create_jwt, verify_password},
};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use sqlx::SqlitePool;
use utoipa;
use validator::Validate;

/// User Login
///
/// Authenticates a user and returns a JWT access token.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
#[post("/login")]
async fn login(
    pool: web::Data<SqlitePool>,
    config: web::Data<Config>,
    req: web::Json<LoginRequest>,
) -> Result<impl Responder, AppError> {
    req.validate()?; // 입력값 유효성 검사

    let user = sqlx::query_as!(
        AdminUser,
        "SELECT * FROM admin_user WHERE username = ?",
        req.username
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::unauthorized("Invalid username or password"))?; // 사용자가 없어도 동일 메시지

    if !user.is_active {
        return Err(AppError::unauthorized("User account is inactive"));
    }

    // 비밀번호 검증
    if !verify_password(&req.password, &user.password_hash).await? {
        return Err(AppError::unauthorized("Invalid username or password"));
    }

    // 로그인 성공, JWT 생성
    let token = create_jwt(user.id, user.user_type_id, &user.username, &config)?;

    // 마지막 로그인 시간 업데이트 (오류 무시 가능)
    let _ = sqlx::query!(
        "UPDATE admin_user SET last_login_at = CURRENT_TIMESTAMP WHERE id = ?",
        user.id
    )
    .execute(pool.get_ref())
    .await; // 실패해도 로그인은 성공 처리

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
    }))
}

/// Get Current User Info
///
/// Returns information about the currently authenticated user based on the provided JWT.
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "Current user information", body = CurrentUserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
#[get("/me")]
async fn get_current_user(
    pool: web::Data<SqlitePool>,     // DB 풀 추가 (UserType 정보 조회용)
    current_user: AuthenticatedUser, // 미들웨어에서 주입된 사용자 정보
) -> Result<impl Responder, AppError> {
    // 사용자 종류 정보 조회 (선택적)
    let user_type_info = sqlx::query_as!(
        UserTypeResponse, // UserTypeResponse DTO 사용
        "SELECT id, name, description, created_at, updated_at FROM user_type WHERE id = ?",
        current_user.user_type_id
    )
    .fetch_optional(pool.get_ref())
    .await?;

    let response = CurrentUserResponse {
        id: current_user.id,
        username: current_user.username,
        user_type_id: current_user.user_type_id,
        user_type: user_type_info,
        permissions: current_user.permissions.iter().cloned().collect(), // HashSet -> Vec
    };
    Ok(HttpResponse::Ok().json(response))
}

pub fn configure_routes() -> Scope {
    web::scope("/auth").service(login).service(get_current_user)
}
