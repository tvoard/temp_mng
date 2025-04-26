use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AdminUser {
    #[schema(example = 101)]
    pub id: i64,
    #[schema(example = "john_doe")]
    pub username: String,
    #[serde(skip_serializing)] // 비밀번호 해시는 응답에 포함하지 않음
    pub password_hash: String,
    #[schema(example = 2)]
    pub user_type_id: i64,
    #[schema(example = true)]
    pub is_active: bool,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
