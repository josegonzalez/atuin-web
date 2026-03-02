use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use serde::Deserialize;
use tower_sessions::Session;

use crate::app::AppState;
use crate::auth;
use crate::error::WebError;
use crate::templates;

const DEFAULT_PAGE_SIZE: u64 = 25;
const ALLOWED_PAGE_SIZES: [u64; 3] = [25, 50, 100];

pub const ALLOWED_TAGS: [&str; 5] = [
    "history",
    "kv",
    "config-shell-alias",
    "dotfiles-var",
    "script",
];

pub fn tag_label(tag: &str) -> &str {
    match tag {
        "history" => "History",
        "kv" => "Key-Value",
        "config-shell-alias" => "Aliases",
        "dotfiles-var" => "Variables",
        "script" => "Scripts",
        _ => "Records",
    }
}

#[derive(Debug, Deserialize)]
pub struct RecordsQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    pub tag: Option<String>,
}

fn default_page() -> u64 {
    1
}
fn default_page_size() -> u64 {
    DEFAULT_PAGE_SIZE
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PaginationInfo {
    pub current_page: u64,
    pub total_pages: u64,
    pub total_records: u64,
    pub page_size: u64,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_page: u64,
    pub next_page: u64,
    pub page_numbers: Vec<u64>,
    pub page_sizes: Vec<u64>,
}

pub fn clamp_page_size(size: u64) -> u64 {
    ALLOWED_PAGE_SIZES
        .iter()
        .min_by_key(|&&s| (s as i64 - size as i64).unsigned_abs())
        .copied()
        .unwrap_or(DEFAULT_PAGE_SIZE)
}

pub fn calculate_pagination(page: u64, total_records: u64, page_size: u64) -> PaginationInfo {
    let page_size = clamp_page_size(page_size);
    let total_pages = if total_records == 0 {
        1
    } else {
        (total_records + page_size - 1) / page_size
    };
    let current_page = page.max(1).min(total_pages);

    // Sliding window of up to 5 page numbers centered on current_page
    let window_size: u64 = 5;
    let half = window_size / 2;
    let (win_start, win_end) = if total_pages <= window_size {
        (1, total_pages)
    } else if current_page <= half + 1 {
        (1, window_size)
    } else if current_page + half >= total_pages {
        (total_pages - window_size + 1, total_pages)
    } else {
        (current_page - half, current_page + half)
    };
    let page_numbers: Vec<u64> = (win_start..=win_end).collect();

    PaginationInfo {
        current_page,
        total_pages,
        total_records,
        page_size,
        has_prev: current_page > 1,
        has_next: current_page < total_pages,
        prev_page: if current_page > 1 {
            current_page - 1
        } else {
            1
        },
        next_page: if current_page < total_pages {
            current_page + 1
        } else {
            total_pages
        },
        page_numbers,
        page_sizes: ALLOWED_PAGE_SIZES.to_vec(),
    }
}

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<RecordsQuery>,
    session: Session,
    headers: HeaderMap,
) -> Result<Html<String>, WebError> {
    let token = auth::get_token_from_config_or_session(
        &state.config,
        auth::get_session_token(&session).await,
    )
    .ok_or(WebError::Unauthorized)?;

    // Validate tag — if missing or not in ALLOWED_TAGS, show landing page
    let tag = match &query.tag {
        Some(t) if ALLOWED_TAGS.contains(&t.as_str()) => t.clone(),
        _ => {
            let is_htmx = headers
                .get("HX-Request")
                .map(|v| v == "true")
                .unwrap_or(false);

            let template = if is_htmx {
                "records_index.html"
            } else {
                "records_index.html"
            };

            let html = templates::render(
                &state.templates,
                template,
                minijinja::context! {
                    active_page => "records",
                    tag => "",
                    has_config_token => state.config.token.is_some(),
                },
            )?;
            return Ok(Html(html));
        }
    };

    let label = tag_label(&tag);

    let records = state.client.get("/api/v0/record", &token).await;

    // Extract total record count for the selected tag
    // Find the first host that has records for this tag
    let (total_records, target_host) = match &records {
        Ok(status) => {
            if let Some(hosts) = status["hosts"].as_object() {
                let mut found_host = None;
                let mut count = 0u64;
                for (host_id, tags) in hosts {
                    if let Some(n) = tags.get(&tag).and_then(|v| v.as_u64()) {
                        if found_host.is_none() {
                            found_host = Some(host_id.clone());
                            count = n + 1;
                            break;
                        }
                    }
                }
                (count, found_host)
            } else {
                (0, None)
            }
        }
        Err(_) => (0, None),
    };

    let page_size = clamp_page_size(query.page_size);
    let pagination = calculate_pagination(query.page, total_records, page_size);
    let start = (pagination.current_page - 1) * pagination.page_size;

    // Fetch record/next for the target host
    let next = match target_host {
        Some(host_id) => {
            let path = format!(
                "/api/v0/record/next?host={}&tag={}&start={}&count={}",
                host_id, tag, start, pagination.page_size
            );
            match state.client.get(&path, &token).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(error = %e, "failed to fetch next records from /api/v0/record/next");
                    serde_json::Value::default()
                }
            }
        }
        None => serde_json::Value::default(),
    };

    let is_htmx = headers
        .get("HX-Request")
        .map(|v| v == "true")
        .unwrap_or(false);

    let template = if is_htmx {
        "partials/record_table.html"
    } else {
        "records.html"
    };

    let html = templates::render(
        &state.templates,
        template,
        minijinja::context! {
            next => next,
            pagination => pagination,
            active_page => "records",
            tag => tag,
            tag_label => label,
            has_config_token => state.config.token.is_some(),
        },
    )?;

    Ok(Html(html))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_page_size_exact() {
        assert_eq!(clamp_page_size(25), 25);
        assert_eq!(clamp_page_size(50), 50);
        assert_eq!(clamp_page_size(100), 100);
    }

    #[test]
    fn test_clamp_page_size_nearest() {
        assert_eq!(clamp_page_size(1), 25);
        assert_eq!(clamp_page_size(30), 25);
        assert_eq!(clamp_page_size(40), 50);
        assert_eq!(clamp_page_size(75), 50);
        assert_eq!(clamp_page_size(76), 100);
        assert_eq!(clamp_page_size(200), 100);
    }

    #[test]
    fn test_pagination_empty_records() {
        let p = calculate_pagination(1, 0, 25);
        assert_eq!(p.current_page, 1);
        assert_eq!(p.total_pages, 1);
        assert_eq!(p.total_records, 0);
        assert!(!p.has_prev);
        assert!(!p.has_next);
        assert_eq!(p.page_numbers, vec![1]);
    }

    #[test]
    fn test_pagination_single_page() {
        let p = calculate_pagination(1, 10, 25);
        assert_eq!(p.current_page, 1);
        assert_eq!(p.total_pages, 1);
        assert!(!p.has_prev);
        assert!(!p.has_next);
        assert_eq!(p.page_numbers, vec![1]);
    }

    #[test]
    fn test_pagination_multi_page() {
        let p = calculate_pagination(2, 100, 25);
        assert_eq!(p.current_page, 2);
        assert_eq!(p.total_pages, 4);
        assert!(p.has_prev);
        assert!(p.has_next);
        assert_eq!(p.prev_page, 1);
        assert_eq!(p.next_page, 3);
    }

    #[test]
    fn test_pagination_last_page() {
        let p = calculate_pagination(4, 100, 25);
        assert_eq!(p.current_page, 4);
        assert!(p.has_prev);
        assert!(!p.has_next);
        assert_eq!(p.next_page, 4);
    }

    #[test]
    fn test_pagination_page_clamped_to_max() {
        let p = calculate_pagination(999, 50, 25);
        assert_eq!(p.current_page, 2);
        assert_eq!(p.total_pages, 2);
    }

    #[test]
    fn test_pagination_page_clamped_to_min() {
        let p = calculate_pagination(0, 50, 25);
        assert_eq!(p.current_page, 1);
    }

    #[test]
    fn test_pagination_window_at_start() {
        let p = calculate_pagination(1, 250, 25);
        assert_eq!(p.total_pages, 10);
        assert_eq!(p.page_numbers, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_pagination_window_at_middle() {
        let p = calculate_pagination(5, 250, 25);
        assert_eq!(p.page_numbers, vec![3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_pagination_window_at_end() {
        let p = calculate_pagination(10, 250, 25);
        assert_eq!(p.page_numbers, vec![6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_pagination_fewer_pages_than_window() {
        let p = calculate_pagination(1, 75, 25);
        assert_eq!(p.total_pages, 3);
        assert_eq!(p.page_numbers, vec![1, 2, 3]);
    }

    #[test]
    fn test_pagination_page_sizes_list() {
        let p = calculate_pagination(1, 100, 25);
        assert_eq!(p.page_sizes, vec![25, 50, 100]);
    }

    #[test]
    fn test_pagination_page_size_50() {
        let p = calculate_pagination(1, 100, 50);
        assert_eq!(p.page_size, 50);
        assert_eq!(p.total_pages, 2);
    }

    #[test]
    fn test_tag_label() {
        assert_eq!(tag_label("history"), "History");
        assert_eq!(tag_label("kv"), "Key-Value");
        assert_eq!(tag_label("config-shell-alias"), "Aliases");
        assert_eq!(tag_label("dotfiles-var"), "Variables");
        assert_eq!(tag_label("script"), "Scripts");
        assert_eq!(tag_label("unknown"), "Records");
    }

    #[test]
    fn test_allowed_tags_contains_all() {
        assert!(ALLOWED_TAGS.contains(&"history"));
        assert!(ALLOWED_TAGS.contains(&"kv"));
        assert!(ALLOWED_TAGS.contains(&"config-shell-alias"));
        assert!(ALLOWED_TAGS.contains(&"dotfiles-var"));
        assert!(ALLOWED_TAGS.contains(&"script"));
    }

    #[test]
    fn test_allowed_tags_rejects_invalid() {
        assert!(!ALLOWED_TAGS.contains(&"invalid"));
        assert!(!ALLOWED_TAGS.contains(&""));
        assert!(!ALLOWED_TAGS.contains(&"History"));
    }
}
