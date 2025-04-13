// src/handlers/permission.rs
use crate::{
    dtos::{
        common::ListQueryParams,
        permission::{CreatePermissionRequest, PermissionResponse, UpdatePermissionRequest},
    },
    errors::AppError,
    middleware::auth::{AuthenticatedUser, RequirePermission},
    models::Permission,
};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, Scope};
use sqlx::SqlitePool;
use utoipa;
use validator::Validate;

// --- Permission CRUD ---

/// Create a Permission
#[utoipa::path(tag = "Permission Management")]
#[post("")]
async fn create_permission(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("permission:create") 적용
    req: web::Json<CreatePermissionRequest>,
) -> Result<impl Responder, AppError> {
    req.validate()?;
    // ... 생성 로직 (user_type.rs 참고) ...
    let result = sqlx::query!(/* ... */).fetch_one(pool.get_ref()).await;
    // ... 에러 처리 및 성공 응답 ...
    Ok(HttpResponse::Created().json(/* ... PermissionResponse ... */))
}

/// Get list of Permissions
#[utoipa::path( /* ... */ params(ListQueryParams), tag = "Permission Management")]
#[get("")]
async fn get_permissions(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("permission:read") 적용
    query: web::Query<ListQueryParams>,
) -> Result<impl Responder, AppError> {
    // ... 목록 조회 로직 (user_type.rs 참고) ...
    let permissions = sqlx::query_as!(/* ... */).fetch_all(pool.get_ref()).await?;
    let response: Vec<PermissionResponse> = permissions.into_iter().map(|p| p.into()).collect();
    Ok(HttpResponse::Ok().json(response))
}

// Get Permission by ID (구현 필요)
// Update Permission (구현 필요)
// Delete Permission (구현 필요, user_type_permission 연관관계 고려)

pub fn configure_routes() -> Scope {
    web::scope("/permissions")
        .service(create_permission)
        .service(get_permissions)
    // .service(get_permission_by_id)
    // .service(update_permission)
    // .service(delete_permission)
}
