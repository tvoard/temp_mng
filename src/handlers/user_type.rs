use crate::{
    dto::{
        common::ListQueryParams,
        user_type::{CreateUserTypeRequest, UpdateUserTypeRequest},
    },
    errors::AppError,
    middleware::auth::authenticated_user::AuthenticatedUser,
    services::user_type,
};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, Scope};

pub fn route() -> Scope {
    web::scope("/user-types")
        .service(post_user_type)
        .service(get_user_type)
        .service(get_user_type_by_id)
        .service(put_user_type)
        .service(delete_user_type)
}

#[post("")]
async fn post_user_type(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    req: web::Json<CreateUserTypeRequest>,
) -> Result<impl Responder, AppError> {
    let response = user_type::create_user_type(pool, user, req).await?;
    Ok(HttpResponse::Created().json(response))
}

#[get("")]
async fn get_user_type(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    query: web::Query<ListQueryParams>,
) -> Result<impl Responder, AppError> {
    let response = user_type::get_user_type_array(pool, user, query).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[get("/{id}")]
async fn get_user_type_by_id(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let response = user_type::get_user_type_by_id(pool, user, path).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[put("/{id}")]
async fn put_user_type(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    path: web::Path<i64>,
    req: web::Json<UpdateUserTypeRequest>,
) -> Result<impl Responder, AppError> {
    let response = user_type::update_user_type(pool, user, path, req).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[delete("/{id}")]
async fn delete_user_type(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    user_type::delete_user_type(pool, user, path).await?;
    Ok(HttpResponse::NoContent().finish())
}
