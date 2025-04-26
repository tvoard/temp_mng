use chrono::NaiveDateTime;
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
