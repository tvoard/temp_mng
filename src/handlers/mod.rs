pub mod auth;
pub mod health;
pub mod menu;
pub mod permission;
pub mod user;
pub mod user_type;

use crate::handlers;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(handlers::auth::route())
            .service(handlers::health::route())
            .service(handlers::menu::route())
            .service(handlers::permission::route())
            .service(handlers::user::route())
            .service(handlers::user_type::route()),
    );
}
