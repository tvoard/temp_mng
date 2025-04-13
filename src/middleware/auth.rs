// src/middleware/auth.rs
use crate::{
    config::Config,
    errors::AppError,
    models::Permission, // 권한 모델 사용
    utils::{validate_jwt, Claims},
};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderValue, AUTHORIZATION},
    web,
    Error,
    FromRequest,
    HttpMessage, // HttpMessage 추가
};
use futures_util::{
    future::ready,
    future::{ok, FutureExt, LocalBoxFuture, Ready},
};
use sqlx::SqlitePool;
use std::{
    collections::HashSet,
    rc::Rc,
    task::{Context, Poll},
};

// 요청 확장(Extension)에 저장될 사용자 정보
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: i64,
    pub user_type_id: i64,
    pub username: String,
    pub permissions: Rc<HashSet<String>>, // 권한 코드 목록 (Rc로 복사 비용 절감)
}

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
            service: Rc::new(service),
        })
    }
}

// 실제 인증 로직을 수행하는 미들웨어
pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
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
        let config = req.app_data::<web::Data<Config>>().cloned();
        let pool = req.app_data::<web::Data<SqlitePool>>().cloned();

        async move {
            let config = config.ok_or_else(|| {
                tracing::error!("Config not found in app_data");
                AppError::InternalServerError(anyhow::anyhow!("Server configuration error"))
            })?;
            let pool = pool.ok_or_else(|| {
                tracing::error!("Database pool not found in app_data");
                AppError::InternalServerError(anyhow::anyhow!("Database connection error"))
            })?;

            let auth_header = req.headers().get(AUTHORIZATION);

            let claims = match extract_and_validate_token(auth_header, &config) {
                Ok(claims) => claims,
                // 특정 경로는 인증 없이 허용 (예: 로그인, 회원가입, health check, swagger)
                // 경로 기반으로 예외 처리 추가 필요
                Err(e) if should_skip_auth(&req) => {
                    // 인증 없이 다음 미들웨어/핸들러로 진행
                    return service.call(req).await;
                }
                Err(e) => return Err(Error::from(e)), // 인증 실패 시 바로 에러 반환
            };

            // DB에서 사용자 권한 조회
            let permissions = match fetch_user_permissions(&pool, claims.user_type_id).await {
                Ok(perms) => Rc::new(perms),
                Err(e) => return Err(Error::from(e)),
            };

            // 인증된 사용자 정보를 요청 컨텍스트에 삽입
            let authenticated_user = AuthenticatedUser {
                id: claims.sub,
                user_type_id: claims.user_type_id,
                username: claims.username,
                permissions,
            };
            req.extensions_mut().insert(authenticated_user); // Rc 덕분에 클론 비용 저렴

            // 다음 미들웨어 또는 핸들러 호출
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
    // 필요에 따라 다른 공용 경로 추가
}

// 토큰 추출 및 검증 헬퍼 함수
fn extract_and_validate_token(
    auth_header: Option<&HeaderValue>,
    config: &Config,
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

// 사용자 종류 ID 기반 권한 조회
async fn fetch_user_permissions(
    pool: &SqlitePool,
    user_type_id: i64,
) -> Result<HashSet<String>, AppError> {
    let permissions = sqlx::query_as!(
        Permission,
        r#"
        SELECT p.id, p.code, p.description, p.created_at, p.updated_at
        FROM permission p
        JOIN user_type_permission utp ON p.id = utp.permission_id
        WHERE utp.user_type_id = ?
        "#,
        user_type_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|p| p.code)
    .collect::<HashSet<String>>();

    Ok(permissions)
}

// 권한 확인을 위한 Extractor
pub struct RequirePermission(String); // 필요한 권한 코드

pub struct EnsurePermission;

impl FromRequest for EnsurePermission {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        // 요청 Extension에서 AuthenticatedUser 정보 가져오기
        let extensions = req.extensions();
        let user_data = extensions.get::<AuthenticatedUser>();
        let required_permission = &req.match_info().query("permission").to_string();

        match user_data {
            Some(user) => {
                // Rc<HashSet<String>> 에서 권한 확인
                if user.permissions.contains("*") || user.permissions.contains(required_permission)
                {
                    // 권한 있음: Ok(Self) 반환 -> Actix가 핸들러 실행
                    tracing::debug!(
                        "Permission granted for user {} to access '{}'",
                        user.username,
                        required_permission
                    );
                    ready(Ok(EnsurePermission)) // Ok 안에 Self (EnsurePermission) 인스턴스
                } else {
                    // 권한 없음: Err(Error) 반환 -> Actix가 403 Forbidden 응답
                    tracing::warn!(
                        "Permission denied for user {} (ID: {}) attempting to access '{}'. User permissions: {:?}",
                        user.username, user.id, required_permission, user.permissions
                    );
                    ready(Err(Error::from(AppError::forbidden(
                        "Insufficient permissions",
                    )))) // Err 안에 actix_web::Error
                }
            }
            None => {
                // 인증 정보 없음: Err(Error) 반환 -> Actix가 401 Unauthorized 응답
                tracing::warn!("Attempt to access protected resource without authentication. Ensure Authentication middleware runs first.");
                ready(Err(Error::from(AppError::unauthorized(
                    "Authentication required",
                )))) // Err 안에 actix_web::Error
            }
        }
    }
}

// 핸들러에서 현재 사용자 정보를 얻기 위한 Extractor
impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match req.extensions().get::<AuthenticatedUser>().cloned() {
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
