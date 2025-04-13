// src/utils.rs
use crate::config::Config;
use crate::errors::AppError;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// 비밀번호 해싱
pub async fn hash_password(password: &str) -> Result<String, AppError> {
    let password_bytes = password.as_bytes().to_vec(); // bcrypt는 비동기 아님, 스레드 풀에서 실행
    tokio::task::spawn_blocking(move || hash(password_bytes, DEFAULT_COST))
        .await
        .map_err(|e| {
            AppError::InternalServerError(
                anyhow::Error::from(e).context("Password hashing task failed"),
            )
        })?
        .map_err(AppError::PasswordHashingError)
}

// 비밀번호 검증
pub async fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let password_bytes = password.as_bytes().to_vec();
    let hash_str = hash.to_string(); // 해시값 복사
    tokio::task::spawn_blocking(move || verify(password_bytes, &hash_str))
        .await
        .map_err(|e| {
            AppError::InternalServerError(
                anyhow::Error::from(e).context("Password verification task failed"),
            )
        })?
        .map_err(AppError::PasswordHashingError)
}

// --- JWT 관련 ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,          // Subject (user id)
    pub user_type_id: i64, // 사용자 종류 ID 추가
    pub username: String,  // 사용자 이름 추가 (선택적)
    pub exp: usize,        // Expiration time (as timestamp)
                           // 필요시 다른 클레임 추가 (e.g., roles, permissions 직접 포함은 비권장)
}

// JWT 생성
pub fn create_jwt(
    user_id: i64,
    user_type_id: i64,
    username: &str,
    config: &Config,
) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::seconds(config.jwt_expires_in_seconds))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        user_type_id,
        username: username.to_string(),
        exp: expiration as usize,
    };

    let header = Header::new(Algorithm::HS256);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
    .map_err(AppError::JwtError)
}

// JWT 검증 및 Claims 반환
pub fn validate_jwt(token: &str, config: &Config) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::HS256);
    // validation 설정 추가 가능 (e.g., audience, issuer)

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_ref()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(AppError::JwtError)
}
