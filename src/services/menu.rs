use std::collections::HashMap;

use crate::{
    dto::menu::{CreateMenuRequest, MenuResponse},
    errors::AppError,
    middleware::auth::authenticated_user::AuthenticatedUser,
    models::MenuItem,
};
use actix_web::web;
use sqlx::SqlitePool;
use validator::Validate;

pub async fn create_menu(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
    req: web::Json<CreateMenuRequest>,
) -> Result<MenuResponse, AppError> {
    req.validate()?;

    let display_order = req.display_order.unwrap_or(0);
    let is_visible = req.is_visible.unwrap_or(true);

    let result = sqlx::query_as!(
        MenuItem,
        r#"
        INSERT INTO menu_item (name, path, icon, parent_id, display_order, is_visible, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING *
        "#,
        req.name,
        req.path,
        req.icon,
        req.parent_id,
        display_order,
        is_visible
    )
        .fetch_one(pool.get_ref())
        .await?;

    Ok(MenuResponse::from(result))
}

/// Fetches all menu items from the database, ordered by display_order,
/// and converts them into a hierarchical structure of `MenuResponse`.
///
/// # Arguments
///
/// * `pool` - A reference to the SqlitePool for database access.
/// * `_user` - The authenticated user making the request. Currently unused.
///
/// # Returns
///
/// * `Result<Vec<MenuResponse>, AppError>` - On success, returns a vector of `MenuResponse`
///   representing the menu items in a hierarchical tree structure. Returns `AppError` on failure.
pub async fn get_menu_array(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser,
) -> Result<Vec<MenuResponse>, AppError> {
    // 모든 메뉴 항목 조회 (display_order 순으로)
    let all_menus = sqlx::query_as!(
        MenuItem,
        "SELECT * FROM menu_item ORDER BY display_order ASC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    // 계층 구조로 변환
    Ok(build_menu_tree(all_menus))
}

// 메뉴 계층 구조 빌드 헬퍼 함수
fn build_menu_tree(menus: Vec<MenuItem>) -> Vec<MenuResponse> {
    let mut map: HashMap<i64, MenuResponse> = HashMap::new();
    let mut root_menus: Vec<MenuResponse> = Vec::new();

    // 1단계: 모든 메뉴를 DTO로 변환하고 Map에 저장
    for menu_model in menus {
        let menu_dto = MenuResponse::from(menu_model);
        map.insert(menu_dto.id, menu_dto);
    }

    // 2단계: 각 메뉴를 부모의 children으로 이동
    let mut processed_ids = std::collections::HashSet::new(); // 무한 루프 방지용
    let all_ids: Vec<i64> = map.keys().cloned().collect(); // map 변경 중 순회 문제 방지

    for id in all_ids {
        // 이미 처리된 노드는 건너뜀
        if processed_ids.contains(&id) {
            continue;
        }

        if let Some(mut menu) = map.remove(&id) {
            // map에서 노드 소유권 가져오기
            processed_ids.insert(id);

            // 부모 ID가 없으면 루트 메뉴
            if menu.parent_id.is_none() {
                root_menus.push(menu);
                continue;
            }

            // 부모가 map에 없으면 루트로 간주
            let parent_id = menu.parent_id.unwrap();
            if map.get_mut(&parent_id).is_none() {
                menu.parent_id = None;
                root_menus.push(menu);
                continue;
            }

            let parent = map.get_mut(&parent_id).unwrap();
            if parent.children.is_none() {
                parent.children = Some(Box::new(Vec::new()));
            }
            parent.children.as_mut().unwrap().push(menu); // 자식으로 추가
        }
    }

    // 남은 노드 처리 (부모가 먼저 처리되어 map에 남아있는 경우) - 필요한가?
    for (_, remaining_menu) in map {
        if remaining_menu.parent_id.is_none() {
            root_menus.push(remaining_menu);
        }
        // 부모가 있는 남은 노드는 데이터 문제일 수 있음 (로깅 권장)
    }

    // 자식 메뉴들도 정렬 (선택적)
    sort_menu_children_recursive(&mut root_menus);

    root_menus
}

// 재귀적으로 자식 메뉴 정렬
fn sort_menu_children_recursive(menus: &mut Vec<MenuResponse>) {
    menus.sort_by_key(|m| m.display_order);
    for menu in menus {
        if let Some(children) = menu.children.as_mut() {
            sort_menu_children_recursive(children);
        }
    }
}
