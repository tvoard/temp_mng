use crate::{
    dto::menu::{CreateMenuRequest, MenuResponse},
    errors::AppError,
    middleware::auth::AuthenticatedUser,
    models::MenuItem,
};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use sqlx::SqlitePool;
use std::collections::HashMap;
use utoipa;
use validator::Validate;

/// Create a Menu Item
#[utoipa::path(tag = "Menu Management")]
#[post("")]
async fn create_menu(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("menu:create") 적용
    req: web::Json<CreateMenuRequest>,
) -> Result<impl Responder, AppError> {
    req.validate()?;
    // TODO: parent_id 유효성 검사 (존재하는 menu_item.id인지)
    // ... 생성 로직 ...
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

    let menu_response = MenuResponse::from(result);
    Ok(HttpResponse::Created().json(menu_response))
}

/// Get list of Menu Items (Hierarchical)
/// Returns menu items structured as a tree based on parent_id.
#[utoipa::path(tag = "Menu Management")]
#[get("")]
async fn get_menus(
    pool: web::Data<SqlitePool>,
    _user: AuthenticatedUser, // TODO: RequirePermission("menu:read") 적용
                              // query: web::Query<ListQueryParams>, // 페이지네이션 대신 전체 계층 구조 반환
) -> Result<impl Responder, AppError> {
    // 모든 메뉴 항목 조회 (display_order 순으로)
    let all_menus = sqlx::query_as!(
        MenuItem,
        "SELECT * FROM menu_item ORDER BY display_order ASC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    // 계층 구조로 변환
    let menu_tree = build_menu_tree(all_menus);

    Ok(HttpResponse::Ok().json(menu_tree))
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

// Get Menu by ID (구현 필요)
// Update Menu (구현 필요, parent_id 변경 시 순환 참조 방지 로직 필요)
// Delete Menu (구현 필요, 자식 메뉴 처리 방식 결정 필요 - CASCADE, SET NULL 등)

pub fn configure_routes() -> Scope {
    web::scope("/menus").service(create_menu).service(get_menus) // 계층 구조 반환 API
                                                                 // .service(get_menu_by_id)
                                                                 // .service(update_menu)
                                                                 // .service(delete_menu)
}
