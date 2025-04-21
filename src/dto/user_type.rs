use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserTypeRequest {
    #[schema(example = "ReportViewer")]
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
    #[schema(example = "Can view generated reports")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserTypeRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: Option<String>,
    pub description: Option<String>, // description은 null로 업데이트 가능하게 Option<Option<String>> 고려 가능
}

#[derive(Debug, Serialize, ToSchema, Clone)] // Clone 추가
pub struct UserTypeResponse {
    #[schema(example = 3)]
    pub id: i64,
    #[schema(example = "ReportViewer")]
    pub name: String,
    #[schema(example = "Can view generated reports")]
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::models::UserType> for UserTypeResponse {
    fn from(ut: crate::models::UserType) -> Self {
        Self {
            id: ut.id,
            name: ut.name,
            description: ut.description.unwrap_or("".to_string()),
            created_at: Utc.from_utc_datetime(&ut.created_at),
            updated_at: Utc.from_utc_datetime(&ut.updated_at),
        }
    }
}
