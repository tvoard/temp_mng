use serde::Serialize;

#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub uptime: u64, // 예: 서버 동작 시간(초)
}
