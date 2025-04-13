use actix_web::{
    http::{header, StatusCode},
    HttpResponse, ResponseError,
};
use http::HeaderValue;
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Password hashing error: {0}")]
    PasswordHashingError(#[from] bcrypt::BcryptError),

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Validation error")]
    ValidationError(#[from] ValidationErrors), // 추가

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")] // 추가 (예: 중복 데이터)
    Conflict(String),

    #[error("Internal server error")]
    InternalServerError(#[from] anyhow::Error), // anyhow::Error 처리 추가
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>, // Validation 에러 상세 정보
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            AppError::DatabaseError(sqlx::Error::Database(db_err))
                if db_err.is_unique_violation() =>
            {
                StatusCode::CONFLICT
            }
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JwtError(kind) => match kind.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken
                | jsonwebtoken::errors::ErrorKind::InvalidSignature
                | jsonwebtoken::errors::ErrorKind::ExpiredSignature
                | jsonwebtoken::errors::ErrorKind::InvalidAudience
                | jsonwebtoken::errors::ErrorKind::InvalidIssuer
                | jsonwebtoken::errors::ErrorKind::ImmatureSignature => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let (message, details) = match self {
            AppError::InternalServerError(ref e) => {
                tracing::error!("Internal Server Error: {:?}", e); // 상세 에러 로깅 (tracing 사용)
                ("An internal server error occurred.".to_string(), None)
            }
            AppError::DatabaseError(ref e) if status == StatusCode::INTERNAL_SERVER_ERROR => {
                tracing::error!("Internal Database Error: {:?}", e);
                ("An internal database error occurred.".to_string(), None)
            }
            AppError::PasswordHashingError(ref e) => {
                tracing::error!("Password Hashing Error: {:?}", e);
                (
                    "An internal error occurred during password processing.".to_string(),
                    None,
                )
            }
            AppError::JwtError(ref e) if status == StatusCode::INTERNAL_SERVER_ERROR => {
                tracing::error!("Internal JWT Error: {:?}", e);
                (
                    "An internal error occurred during authentication processing.".to_string(),
                    None,
                )
            }
            AppError::ValidationError(ref e) => (
                "Input validation failed".to_string(),
                Some(serde_json::to_value(e.field_errors()).unwrap_or_default()),
            ),
            _ => (self.to_string(), None),
        };

        let error_response = ErrorResponse {
            code: status.as_u16(),
            error: status.canonical_reason().unwrap_or("Error").to_string(),
            message,
            details,
        };

        let mut response = HttpResponse::build(status).json(error_response);
        // Unauthorized 시 WWW-Authenticate 헤더 추가 (선택적)
        if status == StatusCode::UNAUTHORIZED {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_str("Bearer").unwrap(),
            );
        }
        response
    }
}

// 편의 생성자
impl AppError {
    pub fn not_found(message: &str) -> Self {
        AppError::NotFound(message.to_string())
    }
    pub fn bad_request(message: &str) -> Self {
        AppError::BadRequest(message.to_string())
    }
    pub fn unauthorized(message: &str) -> Self {
        AppError::Unauthorized(message.to_string())
    }
    pub fn forbidden(message: &str) -> Self {
        AppError::Forbidden(message.to_string())
    }
    pub fn conflict(message: &str) -> Self {
        AppError::Conflict(message.to_string())
    }
    // 필요시 다른 생성자 추가
}
