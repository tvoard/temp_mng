use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

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
    pub display_order: i64,
    #[schema(example = true)]
    pub is_visible: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
