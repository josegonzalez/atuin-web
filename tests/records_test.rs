mod common;

fn mock_record_status() -> &'static str {
    r#"{"hosts":{"host-abc":{"history":100}}}"#
}

fn mock_record_next() -> String {
    let records: Vec<serde_json::Value> = (0..25)
        .map(|i| {
            serde_json::json!({
                "id": format!("018d0d50-a348-7000-8000-{:012x}", i),
                "idx": i,
                "host": {"id": "host-abc"},
                "data": {"data": "", "content_encryption_key": ""},
                "tag": "history",
                "version": "v0",
            })
        })
        .collect();
    serde_json::to_string(&records).unwrap()
}

#[tokio::test]
async fn test_records_no_tag_shows_index() {
    let app = common::spawn_app_with_token("rec-token").await;

    let response = app.server.get("/records").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(
        body.contains("Select a record type"),
        "should show index page, got: {}",
        body
    );
}

#[tokio::test]
async fn test_records_invalid_tag_shows_index() {
    let app = common::spawn_app_with_token("rec-token").await;

    let response = app.server.get("/records?tag=invalid").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(
        body.contains("Select a record type"),
        "invalid tag should show index page, got: {}",
        body
    );
}

#[tokio::test]
async fn test_records_with_valid_tag() {
    let mut app = common::spawn_app_with_token("rec-token").await;

    app.mock_server
        .mock("GET", "/api/v0/record")
        .match_header("Authorization", "Token rec-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_record_status())
        .create_async()
        .await;

    app.mock_server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/api/v0/record/next\?.*".to_string()),
        )
        .match_header("Authorization", "Token rec-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_record_next())
        .create_async()
        .await;

    let response = app.server.get("/records?tag=history").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(
        body.contains("History") || body.contains("history"),
        "should show history records page, got: {}",
        body
    );
}

#[tokio::test]
async fn test_records_htmx_returns_partial() {
    let mut app = common::spawn_app_with_token("rec-token").await;

    app.mock_server
        .mock("GET", "/api/v0/record")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_record_status())
        .create_async()
        .await;

    app.mock_server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/api/v0/record/next\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_record_next())
        .create_async()
        .await;

    let response = app
        .server
        .get("/records?tag=history")
        .add_header("HX-Request", "true")
        .await;
    response.assert_status_ok();
    let body = response.text();
    assert!(
        !body.contains("<!DOCTYPE"),
        "htmx response should be partial, not full page"
    );
}

#[tokio::test]
async fn test_records_sort_asc() {
    let mut app = common::spawn_app_with_token("rec-token").await;

    app.mock_server
        .mock("GET", "/api/v0/record")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_record_status())
        .create_async()
        .await;

    app.mock_server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/api/v0/record/next\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_record_next())
        .create_async()
        .await;

    let response = app.server.get("/records?tag=history&sort=asc").await;
    response.assert_status_ok();
    let body = response.text();
    // Verify sort is preserved in page links
    assert!(
        body.contains("sort=asc"),
        "should preserve sort=asc in links, got: {}",
        body
    );
}

#[tokio::test]
async fn test_records_pagination() {
    let mut app = common::spawn_app_with_token("rec-token").await;

    app.mock_server
        .mock("GET", "/api/v0/record")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_record_status())
        .create_async()
        .await;

    app.mock_server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"/api/v0/record/next\?.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_record_next())
        .create_async()
        .await;

    let response = app
        .server
        .get("/records?tag=history&page=2&page_size=25")
        .await;
    response.assert_status_ok();
    let body = response.text();
    // 100 records / 25 per page = 4 pages, page 2 should render
    assert!(body.contains("Page 2"), "should show page 2, got: {}", body);
}
