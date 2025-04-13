use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQueryParams {
    #[param(example = 1)]
    pub page: Option<i64>,
    #[param(example = 20)]
    pub limit: Option<i64>,
    #[param(example = "name")]
    pub sort_by: Option<String>, // 예: "name", "created_at"
    #[param(example = "asc")]
    pub order: Option<String>, // "asc" or "desc"
    // 다른 필터링 파라미터 추가 가능 (예: q=search_term, is_active=true)
    pub q: Option<String>, // 검색어
}

impl ListQueryParams {
    pub fn get_limit(&self) -> i64 {
        self.limit.unwrap_or(20).max(1).min(100) // 기본값 20, 최소 1, 최대 100
    }

    pub fn get_offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.get_limit()
    }

    // 정렬 문자열 생성 (SQL Injection 주의 - 컬럼명 화이트리스트 방식 권장)
    pub fn get_order_by(&self, allowed_columns: &[&str]) -> String {
        let sort_col = self.sort_by.as_deref().unwrap_or("id"); // 기본 정렬 컬럼
        let order_dir = self.order.as_deref().unwrap_or("asc");

        if allowed_columns.contains(&sort_col) && (order_dir == "asc" || order_dir == "desc") {
            format!("{} {}", sort_col, order_dir.to_uppercase())
        } else {
            "id ASC".to_string() // 기본값 또는 안전한 값
        }
    }
}
