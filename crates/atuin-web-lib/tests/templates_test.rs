use atuin_web_lib::routes::records::PaginationInfo;
use atuin_web_lib::templates;

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
        },
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    // With 0 records, pagination section should be completely hidden
    assert!(!html.contains("Page 1 of 1"));
    assert!(!html.contains("per page"));
}
