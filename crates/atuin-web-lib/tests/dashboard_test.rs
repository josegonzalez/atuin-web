mod common;

async fn setup_dashboard_mocks(app: &mut common::TestApp) {
    app.mock_server
        .mock("GET", "/api/v0/me")
        .match_header("Authorization", "Token dash-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"username":"dashuser"}"#)
        .create_async()
        .await;

    app.mock_server
        .mock("GET", "/api/v0/record")
        .match_header("Authorization", "Token dash-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"hosts":{"host1":{"history":10}}}"#)
        .create_async()
        .await;

    app.mock_server
        .mock("GET", "/healthz")
        .with_status(200)
        .with_body("Ok")
        .create_async()
        .await;
}

#[tokio::test]
async fn test_dashboard_renders_with_data() {
    let mut app = common::spawn_app_with_token("dash-token").await;
    setup_dashboard_mocks(&mut app).await;

    let response = app.server.get("/").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(body.contains("dashuser"), "should contain username");
    assert!(body.contains("Healthy"), "should show health status");
}

#[tokio::test]
async fn test_dashboard_htmx_returns_partial() {
    let mut app = common::spawn_app_with_token("dash-token").await;
    setup_dashboard_mocks(&mut app).await;

    let response = app
        .server
        .get("/")
        .add_header("HX-Request", "true")
        .await;
    response.assert_status_ok();
    let body = response.text();
    // Partial should contain dashboard content but not the full HTML shell
    assert!(body.contains("dashuser"), "partial should contain username");
    assert!(!body.contains("<!DOCTYPE"), "partial should not contain doctype");
}

#[tokio::test]
async fn test_dashboard_handles_upstream_errors() {
    let mut app = common::spawn_app_with_token("dash-token").await;

    // Mock only healthz, let /api/v0/me and /api/v0/record return 500
    app.mock_server
        .mock("GET", "/api/v0/me")
        .with_status(500)
        .create_async()
        .await;

    app.mock_server
        .mock("GET", "/api/v0/record")
        .with_status(500)
        .create_async()
        .await;

    app.mock_server
        .mock("GET", "/healthz")
        .with_status(200)
        .with_body("Ok")
        .create_async()
        .await;

    let response = app.server.get("/").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(
        body.contains("Failed to fetch") || body.contains("error"),
        "should show error messages, got: {}",
        body
    );
}

#[tokio::test]
async fn test_dashboard_health_json_response() {
    let mut app = common::spawn_app_with_token("dash-token").await;

    app.mock_server
        .mock("GET", "/api/v0/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"username":"testuser"}"#)
        .create_async()
        .await;

    app.mock_server
        .mock("GET", "/api/v0/record")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"hosts":{}}"#)
        .create_async()
        .await;

    // Return JSON health response
    app.mock_server
        .mock("GET", "/healthz")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status":"healthy"}"#)
        .create_async()
        .await;

    let response = app.server.get("/").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(
        body.contains("Healthy"),
        "JSON healthy status should render as healthy badge, got: {}",
        body
    );
}

#[tokio::test]
async fn test_dashboard_health_plain_ok() {
    let mut app = common::spawn_app_with_token("dash-token").await;

    app.mock_server
        .mock("GET", "/api/v0/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"username":"testuser"}"#)
        .create_async()
        .await;

    app.mock_server
        .mock("GET", "/api/v0/record")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"hosts":{}}"#)
        .create_async()
        .await;

    app.mock_server
        .mock("GET", "/healthz")
        .with_status(200)
        .with_body("Ok")
        .create_async()
        .await;

    let response = app.server.get("/").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(
        body.contains("Healthy"),
        "Plain 'Ok' should render as healthy badge, got: {}",
        body
    );
}
