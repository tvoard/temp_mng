use crate::{
    dto::auth::LoginRequest, errors::AppError,
    middleware::auth::authenticated_user::AuthenticatedUser, services::auth,
};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};

pub fn route() -> Scope {
    web::scope("/auth")
        .service(post_auth_login)
        .service(get_auth_me)
}

#[post("/login")]
async fn post_auth_login(
    pool: web::Data<sqlx::SqlitePool>,
    config: web::Data<crate::config::env::Env>,
    req: web::Json<LoginRequest>,
) -> Result<impl Responder, AppError> {
    let response = auth::login(pool, config, req).await?;
    Ok(HttpResponse::Ok().json(response))
}

#[get("/me")]
async fn get_auth_me(
    pool: web::Data<sqlx::SqlitePool>,
    current_user: AuthenticatedUser,
) -> Result<impl Responder, AppError> {
    let response = auth::get_current_user(pool, current_user).await?;
    Ok(HttpResponse::Ok().json(response))
}
