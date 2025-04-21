use crate::dto::permission::PermissionResponse;
use crate::{
    dto::{
        common::ListQueryParams,
        user_type::{CreateUserTypeRequest, UpdateUserTypeRequest, UserTypeResponse},
    },
    errors::AppError,
    middleware::auth::AuthenticatedUser, // 권한 검사
    models::{Permission, UserType},
};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, Scope};
use sqlx::Arguments;
use sqlx::SqlitePool;
use utoipa;
use validator::Validate;

/// Create a User Type (Role)
#[utoipa::path(tag = "User Type Management")]
#[post("")]
async fn create_user_type(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("user_type:create") 적용
    req: web::Json<CreateUserTypeRequest>,
) -> Result<impl Responder, AppError> {
    req.validate()?;

    let result = sqlx::query!(
        "INSERT INTO user_type (name, description) VALUES (?, ?) RETURNING id",
        req.name,
        req.description
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(record) => {
            let created_type =
                sqlx::query_as!(UserType, "SELECT * FROM user_type WHERE id = ?", record.id)
                    .fetch_one(pool.get_ref())
                    .await?;
            Ok(HttpResponse::Created().json(UserTypeResponse::from(created_type)))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(AppError::conflict("User type name already exists"))
        }
        Err(e) => Err(AppError::from(e)),
    }
}

/// Get list of User Types
/// Requires 'user_type:read' permission.
#[utoipa::path(params(ListQueryParams), tag = "User Type Management")]
#[get("")]
async fn get_user_types(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
    query: web::Query<ListQueryParams>,
) -> Result<impl Responder, AppError> {
    let limit = query.get_limit();
    let offset = query.get_offset();
    let allowed_sort_columns = ["id", "name", "created_at", "updated_at"];
    let order_by = query.get_order_by(&allowed_sort_columns);

    let query_str = format!(
        "SELECT * FROM user_type ORDER BY {} LIMIT {} OFFSET {}",
        order_by, limit, offset
    );

    let user_types =
        sqlx::query_as_with::<_, UserType, _>(&query_str, sqlx::sqlite::SqliteArguments::default())
            .fetch_all(pool.get_ref())
            .await?;

    let response: Vec<UserTypeResponse> =
        user_types.into_iter().map(UserTypeResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}

// Get User Type by ID (구현 필요)
#[utoipa::path(tag = "User Type Management")]
#[get("/{type_id}")]
async fn get_user_type_by_id(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("user_type:read") 적용
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let type_id = path.into_inner();
    let user_type = sqlx::query_as!(UserType, "SELECT * FROM user_type WHERE id = ?", type_id)
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| AppError::not_found("User type not found"))?;

    Ok(HttpResponse::Ok().json(UserTypeResponse::from(user_type)))
}

// Update User Type (구현 필요)
#[utoipa::path(tag = "User Type Management")]
#[put("/{type_id}")]
async fn update_user_type(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("user_type:update") 적용
    path: web::Path<i64>,
    req: web::Json<UpdateUserTypeRequest>,
) -> Result<impl Responder, AppError> {
    req.validate()?;
    let type_id = path.into_inner();

    // 업데이트할 필드만 동적으로 구성 (모든 필드가 Option이므로)
    let mut set_clauses = Vec::new();
    let mut args = sqlx::sqlite::SqliteArguments::default();
    if let Some(name) = &req.name {
        set_clauses.push("name = ?");
        args.add(name);
    }
    if req.description.is_some() {
        set_clauses.push("description = ?");
        args.add(req.description.as_ref()); // Option<&String> -> Option<&str> (sqlx가 처리)
    }

    if set_clauses.is_empty() {
        return Err(AppError::bad_request("No fields to update"));
    }
    set_clauses.push("updated_at = CURRENT_TIMESTAMP"); // updated_at 갱신

    args.add(type_id); // WHERE 절의 id
    let query_str = format!(
        "UPDATE user_type SET {} WHERE id =?",
        set_clauses.join(", ")
    );

    let result = sqlx::query_with(&query_str, args)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(exec_result) => {
            if exec_result.rows_affected() == 0 {
                Err(AppError::not_found("User type not found"))
            } else {
                // 업데이트된 정보 다시 조회해서 반환
                let updated_type =
                    sqlx::query_as!(UserType, "SELECT * FROM user_type WHERE id =?", type_id)
                        .fetch_one(pool.get_ref())
                        .await?;
                Ok(HttpResponse::Ok().json(UserTypeResponse::from(updated_type)))
            }
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(AppError::conflict("User type name already exists"))
        }
        Err(e) => Err(AppError::from(e)),
    }
}

// Delete User Type (구현 필요)
#[utoipa::path(tag = "User Type Management")]
#[delete("/{type_id}")]
async fn delete_user_type(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("user_type:delete") 적용
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let type_id = path.into_inner();
    let mut tx = pool.begin().await?;

    // TODO: 해당 UserType을 사용하는 AdminUser가 있는지 확인하는 로직 추가 (ON DELETE RESTRICT 때문)
    let user_count: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM admin_user WHERE user_type_id =?")
            .bind(type_id)
            .fetch_one(&mut *tx)
            .await?;

    if user_count > 0 {
        return Err(AppError::conflict(
            "Cannot delete a user type: it is currently assigned to users.",
        ));
    }

    let result = sqlx::query!("DELETE FROM user_type WHERE id = ?", type_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        Err(AppError::not_found("User type not found"))
    } else {
        Ok(HttpResponse::NoContent().finish()) // 204 No Content
    }
}

// --- User Type - Permission/Menu Management ---

/// Get Permissions for a User Type
#[utoipa::path(tag = "User Type Management")]
#[get("/{type_id}/permissions")]
async fn get_user_type_permissions(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
    path: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let type_id = path.into_inner();
    let permissions = sqlx::query_as!(
        Permission,
        "SELECT 
        p.id as id,
        p.code as code,
        p.description as description,
        p.created_at as created_at,
        p.updated_at as updated_at
    FROM 
        permission p
    JOIN 
        user_type_permission utp ON p.id = utp.permission_id 
    WHERE 
        utp.user_type_id = ? AND p.id IS NOT NULL",
        type_id
    )
    .fetch_all(pool.get_ref())
    .await?;

    // Permission 모델 -> PermissionResponse DTO로 변환 (필요시)
    let response: Vec<PermissionResponse> = permissions.into_iter().map(|p| p.into()).collect();
    Ok(HttpResponse::Ok().json(response))
}

/// Add Permission to User Type
#[utoipa::path(tag = "User Type Management")]
#[post("/{type_id}/permissions/{permission_id}")]
async fn add_permission_to_user_type(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
    path: web::Path<(i64, i64)>,
) -> Result<impl Responder, AppError> {
    let (type_id, permission_id) = path.into_inner();
    // TODO: type_id, permission_id 유효성 검사 (DB에 존재하는지)
    sqlx::query!(
        "INSERT OR IGNORE INTO user_type_permission (user_type_id, permission_id) VALUES (?, ?)",
        type_id,
        permission_id
    )
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::NoContent().finish()) // 204 No Content
}

/// Remove Permission from User Type
#[utoipa::path(tag = "User Type Management")]
#[delete("/{type_id}/permissions/{permission_id}")]
async fn remove_permission_from_user_type(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("user_type:update") or specific
    path: web::Path<(i64, i64)>,
) -> Result<impl Responder, AppError> {
    let (type_id, permission_id) = path.into_inner();
    sqlx::query!(
        "DELETE FROM user_type_permission WHERE user_type_id = ? AND permission_id = ?",
        type_id,
        permission_id
    )
    .execute(pool.get_ref())
    .await?; // 삭제된 행 없어도 에러 아님

    Ok(HttpResponse::NoContent().finish())
}

// --- 메뉴 관련 핸들러 (get_user_type_menus, add_menu_to_user_type, remove_menu_from_user_type) ---
// 위의 권한 관리 핸들러와 유사하게 구현
// 모델: MenuItem, DTO: MenuResponse 사용

pub fn configure_routes() -> Scope {
    web::scope("/user-types")
        .service(create_user_type)
        .service(get_user_type_by_id)
        .service(update_user_type)
        .service(delete_user_type)
        .service(add_permission_to_user_type)
        .service(remove_permission_from_user_type)
}
