use crate::{
    dto::{
        common::ListQueryParams,
        permission::{CreatePermissionRequest, PermissionResponse},
    },
    errors::AppError,
    models::Permission,
};
use actix_web::web;
use sqlx::SqlitePool;
use validator::Validate;

pub async fn create_permission(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    req: web::Json<CreatePermissionRequest>,
) -> Result<i64, AppError> {
    req.validate()?;
    let inserted_id = sqlx::query!(
        "INSERT INTO permission (code, description) VALUES (?, ?) RETURNING id",
        req.code,
        req.description
    )
    .fetch_one(pool.get_ref())
    .await?
    .id;

    Ok(inserted_id)
}

pub async fn get_permissions(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    query: web::Query<ListQueryParams>,
) -> Result<Vec<PermissionResponse>, AppError> {
    let limit = query.limit.unwrap_or(10);
    let page = query.page.unwrap_or(1);
    if limit <= 0 || page <= 0 {
        return Err(AppError::bad_request("Invalid pagination parameters"));
    }
    let offset = (page - 1) * limit;

    let permissions = sqlx::query_as!(
        Permission,
        "SELECT * FROM permission ORDER BY code LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(permissions.into_iter().map(PermissionResponse::from).collect())
}

pub async fn get_permission_by_id(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<PermissionResponse, AppError> {
    let id = path.into_inner();
    let permission = sqlx::query_as!(Permission, "SELECT * FROM permission WHERE id = ?", id)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|_| AppError::not_found("Permission not found"))?;

    Ok(PermissionResponse::from(permission))
}
