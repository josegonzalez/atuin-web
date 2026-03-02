use atuin_web::routes::records::PaginationInfo;
use atuin_web::templates;

#[test]
fn test_create_environment() {
    let env = templates::create_environment();
    // In test mode (debug), templates load from disk
    // Just verify the environment can be created without panicking
    assert!(env.get_template("login.html").is_ok());
}

#[test]
fn test_render_login() {
    let env = templates::create_environment();
    let result = templates::render(
        &env,
        "login.html",
        minijinja::context! {
            error => false,
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Sign in"));
    assert!(html.contains("Username"));
    assert!(html.contains("Password"));
}

#[test]
fn test_render_login_with_error() {
    let env = templates::create_environment();
    let result = templates::render(
        &env,
        "login.html",
        minijinja::context! {
            error => "Bad credentials",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Bad credentials"));
}

#[test]
fn test_render_pagination_multi_page() {
    let env = templates::create_environment();
    let pagination = PaginationInfo {
        current_page: 2,
        total_pages: 4,
        total_records: 100,
        page_size: 25,
        has_prev: true,
        has_next: true,
        prev_page: 1,
        next_page: 3,
        page_numbers: vec![1, 2, 3, 4],
        page_sizes: vec![25, 50, 100],
    };
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([]),
            pagination => pagination,
            tag => "history",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Page 2 of 4"));
    assert!(html.contains("100 total records"));
    assert!(html.contains("page=1&page_size=25"));
    assert!(html.contains("page=3&page_size=25"));
    assert!(html.contains("hx-get"));
    assert!(html.contains("hx-target"));
    assert!(html.contains("25 per page"));
}

#[test]
fn test_render_pagination_hidden_single_page() {
    let env = templates::create_environment();
    let pagination = PaginationInfo {
        current_page: 1,
        total_pages: 1,
        total_records: 5,
        page_size: 25,
        has_prev: false,
        has_next: false,
        prev_page: 1,
        next_page: 1,
        page_numbers: vec![1],
        page_sizes: vec![25, 50, 100],
    };
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([]),
            pagination => pagination,
            tag => "history",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    // Should show summary but no pagination nav (single page)
    assert!(html.contains("Page 1 of 1"));
    assert!(!html.contains("aria-label=\"Records pagination\""));
}

#[test]
fn test_render_pagination_hidden_zero_records() {
    let env = templates::create_environment();
    let pagination = PaginationInfo {
        current_page: 1,
        total_pages: 1,
        total_records: 0,
        page_size: 25,
        has_prev: false,
        has_next: false,
        prev_page: 1,
        next_page: 1,
        page_numbers: vec![1],
        page_sizes: vec![25, 50, 100],
    };
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([]),
            pagination => pagination,
            tag => "history",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    // With 0 records, pagination section should be completely hidden
    assert!(!html.contains("Page 1 of 1"));
    assert!(!html.contains("per page"));
}

#[test]
fn test_render_record_table_with_each_tag() {
    let env = templates::create_environment();
    let tags = [
        "history",
        "kv",
        "config-shell-alias",
        "dotfiles-var",
        "script",
    ];
    for tag in &tags {
        let pagination = PaginationInfo {
            current_page: 1,
            total_pages: 1,
            total_records: 0,
            page_size: 25,
            has_prev: false,
            has_next: false,
            prev_page: 1,
            next_page: 1,
            page_numbers: vec![1],
            page_sizes: vec![25, 50, 100],
        };
        let result = templates::render(
            &env,
            "partials/record_table.html",
            minijinja::context! {
                next => serde_json::json!([]),
                pagination => pagination,
                tag => *tag,
                sort => "desc",
            },
        );
        assert!(
            result.is_ok(),
            "Failed to render record_table for tag: {}",
            tag
        );
    }
}

#[test]
fn test_render_pagination_preserves_tag() {
    let env = templates::create_environment();
    let pagination = PaginationInfo {
        current_page: 2,
        total_pages: 4,
        total_records: 100,
        page_size: 25,
        has_prev: true,
        has_next: true,
        prev_page: 1,
        next_page: 3,
        page_numbers: vec![1, 2, 3, 4],
        page_sizes: vec![25, 50, 100],
    };
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([]),
            pagination => pagination,
            tag => "kv",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(
        html.contains("tag=kv"),
        "Pagination URLs should contain tag=kv"
    );
}

#[test]
fn test_render_records_index() {
    let env = templates::create_environment();
    let result = templates::render(
        &env,
        "records_index.html",
        minijinja::context! {
            active_page => "records",
            tag => "",
            has_config_token => false,
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("tag=history"));
    assert!(html.contains("tag=kv"));
    assert!(html.contains("tag=config-shell-alias"));
    assert!(html.contains("tag=dotfiles-var"));
    assert!(html.contains("tag=script"));
    assert!(html.contains("Select a record type"));
}

#[test]
fn test_render_dashboard() {
    let env = templates::create_environment();

    let result = templates::render(
        &env,
        "partials/dashboard_content.html",
        minijinja::context! {
            me => serde_json::json!({"username": "testuser"}),
            status => serde_json::json!({"hosts": {}}),
            health => "Ok",
            errors => Vec::<String>::new(),
            tag => "",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Healthy"), "Should show healthy badge");
    assert!(html.contains("testuser"), "Should show username");
}

#[test]
fn test_uuid7_timestamp_filter() {
    let env = templates::create_environment();
    // UUIDv7 encoding 2024-01-15 14:30:05 UTC (1705325405000 ms)
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([{
                "id": "018d0d50-a348-7000-8000-000000000000",
                "idx": 42,
                "host": {"id": "host1"},
                "data": {"data": "", "content_encryption_key": ""},
                "tag": "history",
                "version": "v0",
            }]),
            pagination => PaginationInfo {
                current_page: 1,
                total_pages: 1,
                total_records: 1,
                page_size: 25,
                has_prev: false,
                has_next: false,
                prev_page: 1,
                next_page: 1,
                page_numbers: vec![1],
                page_sizes: vec![25, 50, 100],
            },
            tag => "history",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(
        html.contains(" ago"),
        "Should show relative duration, got: {}",
        html
    );
    assert!(
        html.contains(">42<")
            || html.contains("> 42 <")
            || html.contains(">\n                42\n"),
        "Should show idx 42"
    );
}

#[test]
fn test_uuid7_timestamp_filter_invalid_input() {
    let env = templates::create_environment();
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([{
                "id": "not-a-uuid",
                "idx": 1,
                "host": {"id": "host1"},
                "data": {"data": "", "content_encryption_key": ""},
                "tag": "history",
                "version": "v0",
            }]),
            pagination => PaginationInfo {
                current_page: 1,
                total_pages: 1,
                total_records: 1,
                page_size: 25,
                has_prev: false,
                has_next: false,
                prev_page: 1,
                next_page: 1,
                page_numbers: vec![1],
                page_sizes: vec![25, 50, 100],
            },
            tag => "history",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(
        html.contains("\u{2014}"),
        "Invalid UUID should show em-dash fallback"
    );
}

#[test]
fn test_uuid7_timestamp_filter_empty_input() {
    let env = templates::create_environment();
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([{
                "id": "",
                "idx": 1,
                "host": {"id": "host1"},
                "data": {"data": "", "content_encryption_key": ""},
                "tag": "history",
                "version": "v0",
            }]),
            pagination => PaginationInfo {
                current_page: 1,
                total_pages: 1,
                total_records: 1,
                page_size: 25,
                has_prev: false,
                has_next: false,
                prev_page: 1,
                next_page: 1,
                page_numbers: vec![1],
                page_sizes: vec![25, 50, 100],
            },
            tag => "history",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(
        html.contains("\u{2014}"),
        "Empty string should show em-dash fallback"
    );
}

#[test]
fn test_render_nonexistent_template_returns_error() {
    let env = templates::create_environment();
    let result = templates::render(&env, "nonexistent.html", minijinja::context! {});
    assert!(
        result.is_err(),
        "Rendering a nonexistent template should return an error"
    );
}

#[test]
fn test_uuid7_timestamp_filter_non_hex_chars() {
    let env = templates::create_environment();
    // 12+ non-dash chars but invalid hex digits
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([{
                "id": "gggggggggggg-7000-8000-000000000000",
                "idx": 1,
                "host": {"id": "host1"},
                "data": {"data": "", "content_encryption_key": ""},
                "tag": "history",
                "version": "v0",
            }]),
            pagination => PaginationInfo {
                current_page: 1,
                total_pages: 1,
                total_records: 1,
                page_size: 25,
                has_prev: false,
                has_next: false,
                prev_page: 1,
                next_page: 1,
                page_numbers: vec![1],
                page_sizes: vec![25, 50, 100],
            },
            tag => "history",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(
        html.contains("\u{2014}"),
        "Non-hex UUID should show em-dash fallback"
    );
}

#[test]
fn test_uuid7_timestamp_filter_future_timestamp() {
    let env = templates::create_environment();
    // fffffffffff0 hex = ~year 10889, far in the future
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([{
                "id": "fffffffffff0-7000-8000-000000000000",
                "idx": 1,
                "host": {"id": "host1"},
                "data": {"data": "", "content_encryption_key": ""},
                "tag": "history",
                "version": "v0",
            }]),
            pagination => PaginationInfo {
                current_page: 1,
                total_pages: 1,
                total_records: 1,
                page_size: 25,
                has_prev: false,
                has_next: false,
                prev_page: 1,
                next_page: 1,
                page_numbers: vec![1],
                page_sizes: vec![25, 50, 100],
            },
            tag => "history",
            sort => "desc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(
        html.contains("\u{2014}"),
        "Future timestamp should show em-dash fallback"
    );
}

#[test]
fn test_render_pagination_preserves_sort() {
    let env = templates::create_environment();
    let pagination = PaginationInfo {
        current_page: 2,
        total_pages: 4,
        total_records: 100,
        page_size: 25,
        has_prev: true,
        has_next: true,
        prev_page: 1,
        next_page: 3,
        page_numbers: vec![1, 2, 3, 4],
        page_sizes: vec![25, 50, 100],
    };
    let result = templates::render(
        &env,
        "partials/record_table.html",
        minijinja::context! {
            next => serde_json::json!([]),
            pagination => pagination,
            tag => "history",
            sort => "asc",
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(
        html.contains("sort=asc"),
        "Pagination URLs should preserve sort=asc"
    );
    // The Index header toggle link should offer the opposite sort direction
    assert!(
        html.contains("sort=desc"),
        "Index header should link to sort=desc when current is asc"
    );
    // Should show ascending arrow
    assert!(
        html.contains("▲") || html.contains("&#9650;"),
        "Should show ascending arrow"
    );
}
