use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[schema(example = "new_user")]
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub username: String,
    #[schema(example = "Str0ngP@ssw0rd!")]
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    #[schema(example = 2)]
    #[validate(range(min = 1, message = "Invalid user type ID"))]
    pub user_type_id: i64,
    #[schema(example = true)]
    pub is_active: Option<bool>, // 생성 시 선택적 활성화
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    // 유효성 검사: 모든 필드가 Option이므로,至少 하나는 있어야 한다는 커스텀 검증 필요 가능
    #[schema(example = "updated_user")]
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub username: Option<String>,
    #[schema(example = 3)]
    #[validate(range(min = 1, message = "Invalid user type ID"))]
    pub user_type_id: Option<i64>,
    #[schema(example = false)]
    pub is_active: Option<bool>,
    // 비밀번호 변경은 별도 API 권장 (현재 비밀번호 확인 등)
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct UserResponse {
    #[schema(example = 101)]
    pub id: i64,
    #[schema(example = "john_doe")]
    pub username: String,
    #[schema(example = 2)]
    pub user_type_id: i64,
    #[schema(example = true)]
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 모델 -> 응답 DTO 변환
impl From<crate::models::AdminUser> for UserResponse {
    fn from(user: crate::models::AdminUser) -> Self {
        Self {
            id: user.id,
            username: user.username,
            user_type_id: user.user_type_id,
            is_active: user.is_active,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
