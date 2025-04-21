use crate::{
    dto::common::ListQueryParams, dto::permission::CreatePermissionRequest,
    dto::permission::PermissionResponse, errors::AppError, middleware::auth::AuthenticatedUser,
    models::Permission,
};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use sqlx::SqlitePool;
use utoipa;
use validator::Validate;

/// Create a Permission
#[utoipa::path(tag = "Permission Management")]
#[post("")]
async fn create_permission(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
    req: web::Json<CreatePermissionRequest>,
) -> Result<impl Responder, AppError> {
    req.validate()?;
    let inserted_id = sqlx::query!(
        "INSERT INTO permission (code, description) VALUES (?, ?) RETURNING id",
        req.code,
        req.description
    )
    .fetch_one(pool.get_ref())
    .await?
    .id;

    Ok(HttpResponse::Created().json(inserted_id))
}

/// Get a list of Permissions
#[utoipa::path(params(ListQueryParams), tag = "Permission Management")]
#[get("")]
async fn get_permissions(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
    query: web::Query<ListQueryParams>,
) -> Result<impl Responder, AppError> {
    let limit = query.limit.unwrap_or(10);
    let page = query.page.unwrap_or(1);
    if limit <= 0 || page <= 0 {
        return Err(AppError::BadRequest(
            "Invalid pagination parameters".to_string(),
        ));
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

    let response: Vec<PermissionResponse> = permissions.into_iter().map(|p| p.into()).collect();
    Ok(HttpResponse::Ok().json(response))
}

/// Get Permission by ID
#[utoipa::path(tag = "Permission Management")]
#[get("/{id}")]
async fn get_permission_by_id(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
    id: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let id = id.into_inner();
    let permission = sqlx::query_as!(Permission, "SELECT * FROM permission WHERE id = ?", id)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|_| AppError::NotFound("Permission not found".to_string()))?;
    Ok(HttpResponse::Created().json(PermissionResponse::from(permission)))
}

// Update Permission (구현 필요)
// Delete Permission (구현 필요, user_type_permission 연관관계 고려)

pub fn configure_routes() -> Scope {
    web::scope("/permission")
        .service(create_permission)
        .service(get_permissions)
        .service(get_permission_by_id)
    // .service(update_permission)
    // .service(delete_permission)
}
