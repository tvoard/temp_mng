use crate::{errors::AppError, middleware::auth::authenticated_user::AuthenticatedUser};
use actix_web::{dev::Payload, Error, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ready, Ready};

pub struct EnsurePermission;

impl FromRequest for EnsurePermission {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let extensions = req.extensions();
        let user_data = extensions.get::<AuthenticatedUser>();
        let required_permission = &req.match_info().query("permission").to_string();

        match user_data {
            Some(user) => {
                if user.permissions.contains("*") || user.permissions.contains(required_permission)
                {
                    tracing::debug!(
                        "Permission is granted for user {} to access '{}'",
                        user.username,
                        required_permission
                    );
                    ready(Ok(EnsurePermission))
                } else {
                    tracing::warn!(
                        "Permission is denied for user {} (ID: {}) attempting to access '{}'. User permissions: {:?}",
                        user.username, user.id, required_permission, user.permissions
                    );
                    ready(Err(Error::from(AppError::forbidden(
                        "Insufficient permissions",
                    ))))
                }
            }
            None => {
                tracing::warn!("Attempt to access protected resource without authentication. Ensure Authentication middleware runs first.");
                ready(Err(Error::from(AppError::unauthorized(
                    "Authentication required",
                ))))
            }
        }
    }
}
