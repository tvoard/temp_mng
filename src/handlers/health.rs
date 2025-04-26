use crate::services::health;
use actix_web::{get, web, HttpResponse, Responder, Scope};
use utoipa;

pub fn route() -> Scope {
    web::scope("/health").service(get_health)
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Server is healthy", body = String)
    ),
    tag = "Health"
)]
#[get("")]
async fn get_health() -> impl Responder {
    let response = health::get_server_runtime();
    HttpResponse::Ok().json(response)
}
