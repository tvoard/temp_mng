use crate::{
    config::env,
    errors::AppError,
    middleware::auth::authenticated_user::AuthenticatedUser,
    util::{validate_jwt, Claims},
};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderValue, AUTHORIZATION},
    web, Error, HttpMessage,
};
use futures_util::{
    future::{ok, LocalBoxFuture, Ready},
    FutureExt,
};
use sqlx::SqlitePool;
use std::{
    cell::RefCell,
    collections::HashSet,
    rc::Rc,
    task::{Context, Poll},
};

// 인증 미들웨어 팩토리
pub struct Authentication;

impl<S: 'static, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

// 실제 인증 로직을 수행하는 미들웨어
pub struct AuthenticationMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let config = req.app_data::<web::Data<env::Env>>().cloned();
        let pool = req.app_data::<web::Data<SqlitePool>>().cloned();

        async move {
            let config = config.ok_or_else(|| {
                tracing::error!("Config isn't found in app_data");
                AppError::InternalServerError(anyhow::anyhow!("Server configuration error"))
            })?;
            let pool = pool.ok_or_else(|| {
                tracing::error!("Database pool isn't found in app_data");
                AppError::InternalServerError(anyhow::anyhow!("Database connection error"))
            })?;

            let auth_header = req.headers().get(AUTHORIZATION);

            let claims = match extract_and_validate_token(auth_header, &config) {
                Ok(claims) => claims,
                Err(_e) if should_skip_auth(&req) => {
                    return service.call(req).await;
                }
                Err(e) => return Err(Error::from(e)),
            };

            let permissions = match fetch_user_permissions(&pool, claims.user_type_id).await {
                Ok(perms) => Rc::new(perms),
                Err(e) => return Err(Error::from(e)),
            };

            let authenticated_user = AuthenticatedUser {
                id: claims.sub,
                user_type_id: claims.user_type_id,
                username: claims.username,
                permissions,
            };
            req.extensions_mut().insert(authenticated_user);

            service.call(req).await
        }
        .boxed_local()
    }
}

// 인증 건너뛸 경로 확인 (예시)
fn should_skip_auth(req: &ServiceRequest) -> bool {
    let path = req.path();
    path == "/api/v1/auth/login"
        || path == "/api/v1/health"
        || path.starts_with("/swagger-ui")
        || path == "/api-docs/openapi.json"
}

// 토큰 추출 및 검증 헬퍼 함수
fn extract_and_validate_token(
    auth_header: Option<&HeaderValue>,
    config: &env::Env,
) -> Result<Claims, AppError> {
    let header_val =
        auth_header.ok_or_else(|| AppError::unauthorized("Authorization header missing"))?;
    let auth_str = header_val
        .to_str()
        .map_err(|_| AppError::unauthorized("Invalid authorization header format"))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::unauthorized(
            "Invalid token type, expected Bearer token",
        ));
    }

    let token = &auth_str["Bearer ".len()..];
    validate_jwt(token, config)
}

#[derive(sqlx::FromRow)]
struct PermissionCode {
    code: String,
}

async fn fetch_user_permissions(
    pool: &SqlitePool,
    user_type_id: i64,
) -> Result<HashSet<String>, AppError> {
    let permissions = sqlx::query_as!(
        PermissionCode,
        r#"
        SELECT p.code
        FROM permission p
        JOIN user_type_permission utp ON p.id = utp.permission_id
        WHERE utp.user_type_id = ?
        LIMIT 1000
        "#,
        user_type_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("권한 조회 실패: {}", e);
        AppError::DatabaseError(e)
    })?
    .into_iter()
    .map(|p| p.code)
    .collect::<HashSet<String>>();

    if permissions.is_empty() {
        tracing::warn!("사용자 타입 {}에 대한 권한이 없습니다", user_type_id);
    }

    Ok(permissions)
}
