use crate::{
    dto::menu::CreateMenuRequest, errors::AppError,
    middleware::auth::authenticated_user::AuthenticatedUser, services::menu,
};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use sqlx::SqlitePool;
use utoipa;

pub fn route() -> Scope {
    web::scope("/menu").service(post_menu).service(get_menu) // 계층 구조 반환 API
}

/// Create a Menu Item
#[utoipa::path(tag = "Menu Management")]
#[post("")]
async fn post_menu(
    pool: web::Data<SqlitePool>,
    user: AuthenticatedUser,
    req: web::Json<CreateMenuRequest>,
) -> Result<impl Responder, AppError> {
    let response = menu::create_menu(pool, user, req).await?;
    Ok(HttpResponse::Created().json(response))
}

/// Get list of Menu Items (Hierarchical)
/// Returns menu items structured as a tree based on parent_id.
#[utoipa::path(tag = "Menu Management")]
#[get("")]
async fn get_menu(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
) -> Result<impl Responder, AppError> {
    let response = menu::get_menu_array(pool, _user).await?;
    Ok(HttpResponse::Ok().json(response))
}
