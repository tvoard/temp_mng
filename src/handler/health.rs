use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;
use std::sync::Once;
use std::time::Instant;
use utoipa;

static mut SERVER_START_TIME: Option<Instant> = None;
static INIT: Once = Once::new();

#[derive(Serialize)]
struct HealthCheckResponse {
    status: String,
    uptime: u64, // 예: 서버 동작 시간(초)
}

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
    let response = HealthCheckResponse {
        status: "OK".to_string(),
        uptime: get_server_runtime(), // 서버 동작시간 계산 함수
    };
    HttpResponse::Ok().json(response)
}

pub(crate) fn initialize_server_start_time() {
    unsafe {
        INIT.call_once(|| {
            SERVER_START_TIME = Some(Instant::now());
        });
    }
}

fn get_server_runtime() -> u64 {
    unsafe {
        SERVER_START_TIME
            .map(|start_time| start_time.elapsed().as_secs())
            .unwrap_or(0)
    }
}
