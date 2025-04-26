use crate::{
    config::env::Env,
    dto::auth::{CurrentUserResponse, LoginRequest, LoginResponse},
    errors::AppError,
    models::AdminUser,
    util::{create_jwt, verify_password},
};
use actix_web::web;
use chrono::{TimeZone, Utc};
use sqlx::SqlitePool;
use validator::Validate;

pub async fn login(
    pool: web::Data<SqlitePool>,
    config: web::Data<Env>,
    req: web::Json<LoginRequest>,
) -> Result<LoginResponse, AppError> {
    req.validate()?;

    let user = sqlx::query_as!(
        AdminUser,
        r#"SELECT
            id as "id!",
            username as "username!",
            password_hash as "password_hash!",
            user_type_id,
            is_active as "is_active!",
            last_login_at,
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM admin_user
        WHERE username = ?"#,
        req.username
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::unauthorized("Invalid username or password"))?;

    if !user.is_active {
        return Err(AppError::unauthorized("User account is inactive"));
    }

    if !verify_password(&req.password, &user.password_hash).await? {
        return Err(AppError::unauthorized("Invalid username or password"));
    }

    let token = create_jwt(user.id, user.user_type_id, &user.username, &config)?;

    let _ = sqlx::query!(
        "UPDATE admin_user SET last_login_at = CURRENT_TIMESTAMP WHERE id = ?",
        user.id
    )
    .execute(pool.get_ref())
    .await;

    Ok(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
    })
}

pub async fn get_current_user(
    pool: web::Data<SqlitePool>,
    current_user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
) -> Result<CurrentUserResponse, AppError> {
    let user_type_info = sqlx::query!(
        "SELECT id, name, description, created_at, updated_at FROM user_type WHERE id = ?",
        current_user.user_type_id
    )
    .fetch_optional(pool.get_ref())
    .await?
    .map(|record| crate::dto::user_type::UserTypeResponse {
        id: record.id,
        name: record.name,
        description: record.description.unwrap_or_default(),
        created_at: Utc.from_utc_datetime(&record.created_at),
        updated_at: Utc.from_utc_datetime(&record.updated_at),
    });

    Ok(CurrentUserResponse {
        id: current_user.id,
        username: current_user.username,
        user_type_id: current_user.user_type_id,
        user_type: user_type_info,
        permissions: current_user.permissions.iter().cloned().collect(),
    })
}
