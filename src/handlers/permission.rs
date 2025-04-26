use crate::{
    dto::{common::ListQueryParams, permission::CreatePermissionRequest},
    errors::AppError,
    middleware::auth::authenticated_user::AuthenticatedUser,
    services::permission,
};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};

pub fn route() -> Scope {
    web::scope("/permission")
        .service(post_permission)
        .service(get_permission)
        .service(get_permission_by_id)
}

#[post("")]
async fn post_permission(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    req: web::Json<CreatePermissionRequest>,
) -> Result<impl Responder, AppError> {
    let response = permission::create_permission(pool, user, req).await?;
    Ok(HttpResponse::Created().json(response))
}

#[get("")]
async fn get_permission(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    query: web::Query<ListQueryParams>,
) -> Result<impl Responder, AppError> {
    let response = permission::get_permissions(pool, user, query).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[get("/{id}")]
async fn get_permission_by_id(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<impl Responder, AppError> {
    let response = permission::get_permission_by_id(pool, user, path).await?;
    Ok(HttpResponse::Ok().json(response))
}
