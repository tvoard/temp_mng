// src/handlers/health.rs
use actix_web::{get, HttpResponse, Responder};
use utoipa;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Server is healthy", body = String)
    ),
    tag = "Health"
)]
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
