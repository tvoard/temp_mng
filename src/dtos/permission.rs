use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePermissionRequest {
    #[schema(example = "report:generate")]
    #[validate(length(min = 1, message = "Code cannot be empty"))]
    pub code: String,
    #[schema(example = "Allows generating new reports")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePermissionRequest {
    #[validate(length(min = 1, message = "Code cannot be empty"))]
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct PermissionResponse {
    #[schema(example = 10)]
    pub id: i64,
    #[schema(example = "report:generate")]
    pub code: String,
    #[schema(example = "Allows generating new reports")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::Permission> for PermissionResponse {
    fn from(p: crate::models::Permission) -> Self {
        Self {
            id: p.id,
            code: p.code,
            description: p.description,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
