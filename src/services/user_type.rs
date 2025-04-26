use crate::{
    dto::{
        common::ListQueryParams,
        user_type::{CreateUserTypeRequest, UpdateUserTypeRequest, UserTypeResponse},
    },
    errors::AppError,
    models::UserType,
};
use actix_web::web;
use sqlx::Arguments;
use sqlx::SqlitePool;
use validator::Validate;

pub async fn create_user_type(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    req: web::Json<CreateUserTypeRequest>,
) -> Result<UserTypeResponse, AppError> {
    req.validate()?;

    let result = sqlx::query!(
        "INSERT INTO user_type (name, description) VALUES (?, ?) RETURNING id",
        req.name,
        req.description
    )
    .fetch_one(pool.get_ref())
    .await?;

    let created_type = sqlx::query_as!(UserType, "SELECT * FROM user_type WHERE id = ?", result.id)
        .fetch_one(pool.get_ref())
        .await?;

    Ok(UserTypeResponse::from(created_type))
}

pub async fn get_user_type_array(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    query: web::Query<ListQueryParams>,
) -> Result<Vec<UserTypeResponse>, AppError> {
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

    Ok(user_types.into_iter().map(UserTypeResponse::from).collect())
}

pub async fn get_user_type_by_id(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    path: web::Path<i64>,
) -> Result<UserTypeResponse, AppError> {
    let type_id = path.into_inner();
    let user_type = sqlx::query_as!(UserType, "SELECT * FROM user_type WHERE id = ?", type_id)
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| AppError::not_found("User type not found"))?;

    Ok(UserTypeResponse::from(user_type))
}

pub async fn update_user_type(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    path: web::Path<i64>,
    req: web::Json<UpdateUserTypeRequest>,
) -> Result<UserTypeResponse, AppError> {
    req.validate()?;
    let type_id = path.into_inner();

    let mut set_clauses = Vec::new();
    let mut args = sqlx::sqlite::SqliteArguments::default();
    if let Some(name) = &req.name {
        set_clauses.push("name = ?");
        args.add(name);
    }
    if req.description.is_some() {
        set_clauses.push("description = ?");
        args.add(req.description.as_ref());
    }

    if set_clauses.is_empty() {
        return Err(AppError::bad_request("No fields to update"));
    }
    set_clauses.push("updated_at = CURRENT_TIMESTAMP");

    args.add(type_id);
    let query_str = format!(
        "UPDATE user_type SET {} WHERE id =?",
        set_clauses.join(", ")
    );

    let result = sqlx::query_with(&query_str, args)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("User type not found"));
    }

    let updated_type = sqlx::query_as!(UserType, "SELECT * FROM user_type WHERE id =?", type_id)
        .fetch_one(pool.get_ref())
        .await?;

    Ok(UserTypeResponse::from(updated_type))
}

pub async fn delete_user_type(
    pool: web::Data<SqlitePool>,
    _user: crate::middleware::auth::authenticated_user::AuthenticatedUser,
    path: web::Path<i64>,
) -> Result<(), AppError> {
    let type_id = path.into_inner();
    let mut tx = pool.begin().await?;

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
        return Err(AppError::not_found("User type not found"));
    }

    Ok(())
}
