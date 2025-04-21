use crate::{
    dto::common::ListQueryParams,
    dto::user::{CreateUserRequest, UserResponse},
    errors::AppError,
    errors::ErrorResponse,
    middleware::auth::AuthenticatedUser,
    models::AdminUser,
    util::hash_password,
};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use sqlx::{Arguments, Row, SqlitePool};
use utoipa;
use validator::Validate;

/// Create a new Admin User
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden (Insufficient permissions)", body = ErrorResponse),
        (status = 409, description = "Username already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User Management"
)]
#[post("")]
async fn create_user(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // 인증 확인용 (권한은 아래에서 명시적 확인)
    req: web::Json<CreateUserRequest>,
) -> Result<impl Responder, AppError> {
    req.validate()?; // 입력값 유효성 검사

    // TODO: 실제로는 RequirePermission Extractor를 사용하는 것이 더 좋음

    // 비밀번호 해싱
    let password_hash = hash_password(&req.password).await?;
    let is_active = req.is_active.unwrap_or(true);

    // 트랜잭션 시작
    let mut tx = pool.begin().await?;

    // 사용자 종류 ID 유효성 검사 (선택적이지만 권장)
    let type_exists: bool = sqlx::query("SELECT EXISTS(SELECT 1 FROM user_type WHERE id = ?)")
        .bind(req.user_type_id)
        .fetch_one(&mut *tx) // 트랜잭션 사용
        .await?
        .get(0);

    if !type_exists {
        // tx.rollback().await?; // 롤백은 자동으로 되지만 명시 가능
        return Err(AppError::bad_request("Invalid user_type_id"));
    }

    // 사용자 생성
    let result = sqlx::query!(
        "INSERT INTO admin_user (username, password_hash, user_type_id, is_active) VALUES (?, ?, ?, ?) RETURNING id",
        req.username,
        password_hash,
        req.user_type_id,
        is_active
    )
        .fetch_one(&mut *tx) // 트랜잭션 사용
        .await;

    match result {
        Ok(record) => {
            // 생성된 사용자 정보 다시 조회
            let created_user = sqlx::query_as!(
                AdminUser,
                "SELECT * FROM admin_user WHERE id = ?",
                record.id
            )
            .fetch_one(&mut *tx) // 트랜잭션 사용
            .await?;

            tx.commit().await?; // 트랜잭션 커밋
            Ok(HttpResponse::Created().json(UserResponse::from(created_user)))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            // tx.rollback().await?;
            Err(AppError::conflict("Username already exists"))
        }
        Err(e) => {
            // tx.rollback().await?;
            Err(AppError::from(e))
        }
    }
}

/// Get list of Admin Users
/// Requires 'user:read' permission. Supports pagination and sorting.
#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(ListQueryParams),
    responses(
        (status = 200, description = "List of users", body = Vec<UserResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User Management"
)]
#[get("")]
async fn get_users(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // 인증 확인 및 권한 검사 필요
    query_params: web::Query<ListQueryParams>,
) -> Result<impl Responder, AppError> {
    // TODO: RequirePermission("user:read") 적용 필요

    let limit = query_params.get_limit();
    let offset = query_params.get_offset();
    // 허용되는 정렬 컬럼 목록 정의 (SQL Injection 방지)
    let allowed_sort_columns = [
        "id",
        "username",
        "user_type_id",
        "is_active",
        "last_login_at",
        "created_at",
        "updated_at",
    ];
    let order_by = query_params.get_order_by(&allowed_sort_columns);

    // 동적 쿼리 생성 (검색 기능 추가 시 더 복잡해짐)
    let base_query = "SELECT * FROM admin_user";
    let mut conditions = Vec::new();
    let mut args = sqlx::sqlite::SqliteArguments::default(); // 인자 바인딩용

    if let Some(search_term) = &query_params.q {
        conditions.push("username LIKE ?");
        args.add(format!("%{}%", search_term)); // '%' 와일드카드 추가
    }

    let where_clause = if conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let query_str = format!(
        "{} {} ORDER BY {} LIMIT ? OFFSET ?",
        base_query, where_clause, order_by
    );
    args.add(limit);
    args.add(offset);
    let users = sqlx::query_as_with::<_, AdminUser, _>(&query_str, args)
        .fetch_all(pool.get_ref())
        .await?;

    let user_responses: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(HttpResponse::Ok().json(user_responses))
}

#[utoipa::path(get, path = "/api/v1/users/{id}", tag = "User Management")]
#[get("/{id}")]
async fn get_user_by_id(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("user:read") 적용
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let user = sqlx::query_as!(AdminUser, "SELECT * FROM admin_user WHERE id = ?", id)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|_| AppError::NotFound("User not found".to_string()))?;
    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

// --- 기타 CRUD 핸들러 (get_user_by_id, update_user, delete_user) ---
// - 경로 파라미터 사용: web::Path<i64>
// - 권한 검사 적용: RequirePermission("user:read"), RequirePermission("user:update"), RequirePermission("user:delete")
// - 입력 유효성 검사: req.validate()?
// - 트랜잭션 사용 (필요시)
// - 적절한 HTTP 상태 코드 및 응답 반환

pub fn configure_routes() -> Scope {
    web::scope("/users")
        .service(create_user)
        .service(get_users)
        .service(get_user_by_id)
    // .service(update_user) // PUT /users/{id}
    // .service(delete_user) // DELETE /users/{id}
}
