use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserType {
    #[schema(example = 1)]
    pub id: i64,
    #[schema(example = "ContentEditor")]
    pub name: String,
    #[schema(example = "Manages website content")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Permission {
    #[schema(example = 1)]
    pub id: i64,
    #[schema(example = "content:publish")]
    pub code: String, // 권한 코드 (예: "user:create", "post:edit", "*")
    #[schema(example = "Allows publishing content")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct MenuItem {
    #[schema(example = 1)]
    pub id: i64,
    #[schema(example = "User Management")]
    pub name: String,
    #[schema(example = "/admin/users")]
    pub path: String,
    #[schema(example = "icon-users")]
    pub icon: Option<String>,
    #[schema(example = 1, value_type=Option<i64>)]
    pub parent_id: Option<i64>,
    #[schema(example = 10)]
    pub display_order: i32,
    #[schema(example = true)]
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
