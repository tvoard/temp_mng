use crate::{errors::AppError, middleware::auth::authenticated_user::AuthenticatedUser};
use actix_web::{dev::Payload, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ready, Ready};

// 권한 확인을 위한 Extractor
pub struct RequirePermission(pub &'static str);

impl FromRequest for RequirePermission {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let authenticated_user = req.extensions().get::<AuthenticatedUser>().cloned();
        match authenticated_user {
            Some(_user) => ready(Err(AppError::InternalServerError(anyhow::anyhow!(
                "RequirePermission Extractor not fully implemented"
            )))),
            None => ready(Err(AppError::unauthorized("Authentication required"))),
        }
    }
}
