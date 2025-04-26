use crate::errors::AppError;
use actix_web::{dev::Payload, Error, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ok, ready, Ready};
use std::{collections::HashSet, rc::Rc};

// 요청 확장(Extension)에 저장될 사용자 정보
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: i64,
    pub user_type_id: i64,
    pub username: String,
    pub permissions: Rc<HashSet<String>>,
}

// 핸들러에서 현재 사용자 정보를 얻기 위한 Extractor
impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let authenticated_user = req.extensions().get::<AuthenticatedUser>().cloned();
        match authenticated_user {
            Some(user) => ok(user),
            None => {
                tracing::warn!("Attempted to access AuthenticatedUser in a context where it's not available (likely unauthenticated route or middleware issue).");
                ready(Err(Error::from(AppError::unauthorized(
                    "Not authenticated",
                ))))
            }
        }
    }
}
