use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Permission {
    #[schema(example = 1)]
    pub id: Option<i64>,
    #[schema(example = "content:publish")]
    pub code: String,
    #[schema(example = "Allows publishing content")]
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
