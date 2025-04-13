use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin")]
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,
    #[schema(example = "password123")]
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    #[schema(example = "Bearer")]
    pub token_type: String,
}

// 현재 로그인한 사용자 정보 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentUserResponse {
    #[schema(example = 1)]
    pub id: i64,
    #[schema(example = "admin")]
    pub username: String,
    #[schema(example = 1)]
    pub user_type_id: i64,
    pub user_type: Option<super::user_type::UserTypeResponse>, // 사용자 종류 정보 포함 가능
    #[schema()]
    pub permissions: Vec<String>,               // 사용자 종류에 부여된 권한 코드 목록
                                                               // 필요시 메뉴 정보도 포함 가능
}
