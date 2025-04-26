use crate::{
    dto::{common::ListQueryParams, user::CreateUserRequest},
    errors::AppError,
    middleware::auth::authenticated_user::AuthenticatedUser,
    services::user,
};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};

pub fn route() -> Scope {
    web::scope("/user")
        .service(post_user)
        .service(get_user)
        .service(get_user_by_id)
}

#[post("")]
async fn post_user(
    pool: web::Data<sqlx::SqlitePool>,
    // user: AuthenticatedUser,
    req: web::Json<CreateUserRequest>,
) -> Result<impl Responder, AppError> {
    let response = user::create_user(pool,  req).await?;
    Ok(HttpResponse::Created().json(response))
}

#[get("")]
async fn get_user(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    query_params: web::Query<ListQueryParams>,
) -> Result<impl Responder, AppError> {
    let response = user::get_user_array(pool, user, query_params).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[get("/{id}")]
async fn get_user_by_id(
    pool: web::Data<sqlx::SqlitePool>,
    user: AuthenticatedUser,
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let response = user::get_user_by_id(pool, user, path).await?;
    Ok(HttpResponse::Ok().json(response))
}
