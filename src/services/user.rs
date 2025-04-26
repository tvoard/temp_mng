use crate::{
    dto::{
        common::ListQueryParams,
        user::{CreateUserRequest, UserResponse},
    },
    errors::AppError,
    models::AdminUser,
    util::hash_password,
};
use actix_web::web;
use sqlx::Arguments;
use sqlx::SqlitePool;
use validator::Validate;

pub async fn create_user(
    pool: web::Data<SqlitePool>,
    // _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    req: web::Json<CreateUserRequest>,
) -> Result<i64, AppError> {
    req.validate()?;
    let password_hash = hash_password(&req.password).await?;
    let is_active = req.is_active.unwrap_or(true);

    let mut tx = pool.begin().await?;
    let result = sqlx::query!(
        "INSERT INTO admin_user (username, password_hash, user_type_id, is_active) VALUES (?, ?, ?, ?) RETURNING id",
        req.username,
        password_hash,
        req.user_type_id,
        is_active
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    result
        .id
        .ok_or_else(|| AppError::Conflict(String::from("Failed to create user")))
}

pub async fn get_user_array(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    query_params: web::Query<ListQueryParams>,
) -> Result<Vec<UserResponse>, AppError> {
    let limit = query_params.get_limit();
    let offset = query_params.get_offset();
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

    let base_query = "SELECT * FROM admin_user";
    let mut conditions = Vec::new();
    let mut args = sqlx::sqlite::SqliteArguments::default();

    if let Some(search_term) = &query_params.q {
        conditions.push("username LIKE ?");
        args.add(format!("%{}%", search_term));
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

    Ok(users.into_iter().map(UserResponse::from).collect())
}

pub async fn get_user_by_id(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    path: web::Path<i64>,
) -> Result<UserResponse, AppError> {
    let id = path.into_inner();
    let user = sqlx::query_as!(AdminUser, "SELECT * FROM admin_user WHERE id = ?", id)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|_| AppError::not_found("User not found"))?;

    Ok(UserResponse::from(user))
}
