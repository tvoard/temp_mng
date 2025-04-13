use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateMenuRequest {
    #[schema(example = "Settings")]
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    #[schema(example = "/admin/settings")]
    #[validate(length(min = 1, message = "Path cannot be empty"))]
    pub path: String, // TODO: 경로 형식 검증 (예: '/' 시작)
    #[schema(example = "icon-settings")]
    pub icon: Option<String>,
    #[schema(example = 1, value_type = Option<i64>)]
    pub parent_id: Option<i64>, // 부모 ID 유효성 검증 필요
    #[schema(example = 99)]
    pub display_order: Option<i32>,
    #[schema(example = true)]
    pub is_visible: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateMenuRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: Option<String>,
    #[validate(length(min = 1, message = "Path cannot be empty"))]
    pub path: Option<String>,
    pub icon: Option<String>,
    #[schema(value_type = Option<i64>)]
    pub parent_id: Option<Option<i64>>, // null로 변경 가능하도록 Option<Option<>>
    pub display_order: Option<i32>,
    pub is_visible: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct MenuResponse {
    #[schema(example = 5)]
    pub id: i64,
    #[schema(example = "Settings")]
    pub name: String,
    #[schema(example = "/admin/settings")]
    pub path: String,
    #[schema(example = "icon-settings")]
    pub icon: Option<String>,
    #[schema(example = 1, value_type = Option<i64>)]
    pub parent_id: Option<i64>,
    #[schema(example = 99)]
    pub display_order: i32,
    #[schema(example = true)]
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")] // 재귀적으로 자식 메뉴 포함 시 사용
    pub children: Option<Vec<MenuResponse>>,
}

impl From<crate::models::MenuItem> for MenuResponse {
    fn from(m: crate::models::MenuItem) -> Self {
        Self {
            id: m.id,
            name: m.name,
            path: m.path,
            icon: m.icon,
            parent_id: m.parent_id,
            display_order: m.display_order,
            is_visible: m.is_visible,
            created_at: m.created_at,
            updated_at: m.updated_at,
            children: None, // 기본적으로 None, 필요시 별도 로직으로 채움
        }
    }
}
