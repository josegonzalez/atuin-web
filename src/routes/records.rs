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
    #[serde(default = "default_sort")]
    pub sort: String,
    pub host: Option<String>,
}

fn default_page() -> u64 {
    1
}
fn default_page_size() -> u64 {
    DEFAULT_PAGE_SIZE
}
fn default_sort() -> String {
    "desc".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HostOption {
    pub id: String,
    pub total_records: u64,
}

/// List every host holding records for a tag, largest first, ties broken by id.
///
/// The per-host number the API reports is the highest record index, so a host
/// that appears at all holds at least one record.
pub fn host_options(
    hosts: &serde_json::Map<String, serde_json::Value>,
    tag: &str,
) -> Vec<HostOption> {
    let mut options: Vec<HostOption> = hosts
        .iter()
        .filter_map(|(id, tags)| {
            tags.get(tag)
                .and_then(|v| v.as_u64())
                .map(|idx| HostOption {
                    id: id.clone(),
                    total_records: idx + 1,
                })
        })
        .collect();
    options.sort_by(|a, b| {
        b.total_records
            .cmp(&a.total_records)
            .then_with(|| a.id.cmp(&b.id))
    });
    options
}

/// Pick the host to display: the requested one when it holds records, else the largest.
///
/// Falling back to the largest matters on a multi-host account: picking whichever
/// host happened to come first showed one machine's records and hid the rest.
pub fn select_host(options: &[HostOption], requested: Option<&str>) -> Option<HostOption> {
    requested
        .and_then(|want| options.iter().find(|o| o.id == want))
        .or_else(|| options.first())
        .cloned()
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

/// Compute the (start, count) window for reverse pagination.
/// Page 1 returns the highest-index records, descending across pages.
pub fn reverse_pagination_window(page: u64, total_records: u64, page_size: u64) -> (u64, u64) {
    let reverse_start = total_records as i64 - (page * page_size) as i64;
    if reverse_start < 0 {
        (0, (page_size as i64 + reverse_start) as u64)
    } else {
        (reverse_start as u64, page_size)
    }
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
        total_records.div_ceil(page_size)
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
            let template = "records_index.html";

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

    let hosts = match &records {
        Ok(status) => status["hosts"]
            .as_object()
            .map(|hosts| host_options(hosts, &tag))
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    let selected = select_host(&hosts, query.host.as_deref());
    let total_records = selected.as_ref().map(|h| h.total_records).unwrap_or(0);
    let target_host = selected.as_ref().map(|h| h.id.clone());

    let page_size = clamp_page_size(query.page_size);
    let pagination = calculate_pagination(query.page, total_records, page_size);
    let sort = if query.sort == "asc" { "asc" } else { "desc" };
    let reverse = sort == "desc";

    // For history, paginate from the end so page 1 shows the newest records
    let (start, count) = if reverse {
        reverse_pagination_window(pagination.current_page, total_records, pagination.page_size)
    } else {
        (
            (pagination.current_page - 1) * pagination.page_size,
            pagination.page_size,
        )
    };

    // Fetch record/next for the target host
    let next = match &target_host {
        Some(host_id) => {
            let path = format!(
                "/api/v0/record/next?host={}&tag={}&start={}&count={}",
                host_id, tag, start, count
            );
            match state.client.get(&path, &token).await {
                Ok(mut v) => {
                    // Reverse the array so newest records appear first
                    if reverse {
                        if let Some(arr) = v.as_array_mut() {
                            arr.reverse();
                        }
                    }
                    v
                }
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
            sort => sort,
            hosts => hosts,
            host => target_host.unwrap_or_default(),
            has_config_token => state.config.token.is_some(),
        },
    )?;

    Ok(Html(html))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hosts_json(pairs: &[(&str, u64)]) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (id, idx) in pairs {
            map.insert(
                (*id).to_string(),
                serde_json::json!({ "history": idx, "kv": idx }),
            );
        }
        map
    }

    #[test]
    fn test_host_options_orders_by_size() {
        let hosts = hosts_json(&[("aaa", 0), ("bbb", 8121), ("ccc", 40)]);
        let options = host_options(&hosts, "history");
        assert_eq!(
            options.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(),
            vec!["bbb", "ccc", "aaa"]
        );
        assert_eq!(options[0].total_records, 8122);
        assert_eq!(options[2].total_records, 1);
    }

    #[test]
    fn test_host_options_skips_hosts_without_the_tag() {
        let mut hosts = hosts_json(&[("aaa", 3)]);
        hosts.insert("bbb".to_string(), serde_json::json!({ "script": 9 }));
        let options = host_options(&hosts, "history");
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].id, "aaa");
    }

    #[test]
    fn test_select_host_prefers_the_largest() {
        let hosts = hosts_json(&[("aaa", 0), ("bbb", 8121)]);
        let options = host_options(&hosts, "history");
        assert_eq!(select_host(&options, None).unwrap().id, "bbb");
    }

    #[test]
    fn test_select_host_honours_the_request() {
        let hosts = hosts_json(&[("aaa", 0), ("bbb", 8121)]);
        let options = host_options(&hosts, "history");
        assert_eq!(select_host(&options, Some("aaa")).unwrap().id, "aaa");
    }

    #[test]
    fn test_select_host_falls_back_when_the_request_is_unknown() {
        let hosts = hosts_json(&[("aaa", 0), ("bbb", 8121)]);
        let options = host_options(&hosts, "history");
        assert_eq!(select_host(&options, Some("nope")).unwrap().id, "bbb");
    }

    #[test]
    fn test_select_host_on_empty_account() {
        assert!(select_host(&[], None).is_none());
    }

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

    #[test]
    fn test_reverse_pagination_page1_even() {
        // 100 records, page_size 25: page 1 → start=75, count=25 (indices 75-99)
        let (start, count) = reverse_pagination_window(1, 100, 25);
        assert_eq!(start, 75);
        assert_eq!(count, 25);
    }

    #[test]
    fn test_reverse_pagination_last_page_even() {
        // 100 records, page_size 25: page 4 → start=0, count=25 (indices 0-24)
        let (start, count) = reverse_pagination_window(4, 100, 25);
        assert_eq!(start, 0);
        assert_eq!(count, 25);
    }

    #[test]
    fn test_reverse_pagination_middle_page() {
        // 100 records, page_size 25: page 2 → start=50, count=25 (indices 50-74)
        let (start, count) = reverse_pagination_window(2, 100, 25);
        assert_eq!(start, 50);
        assert_eq!(count, 25);
    }

    #[test]
    fn test_reverse_pagination_partial_last_page() {
        // 110 records, page_size 25: page 5 → start=0, count=10 (indices 0-9)
        let (start, count) = reverse_pagination_window(5, 110, 25);
        assert_eq!(start, 0);
        assert_eq!(count, 10);
    }

    #[test]
    fn test_reverse_pagination_single_page() {
        // 10 records, page_size 25: page 1 → start=0, count=10
        let (start, count) = reverse_pagination_window(1, 10, 25);
        assert_eq!(start, 0);
        assert_eq!(count, 10);
    }
}
