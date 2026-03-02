mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn test_healthz_returns_ok() {
    let app = common::spawn_app().await;
    let response = app.server.get("/healthz").await;
    response.assert_status(StatusCode::OK);
    response.assert_json(&serde_json::json!({"status": "ok"}));
}

#[tokio::test]
async fn test_security_headers_present() {
    let app = common::spawn_app().await;
    let response = app.server.get("/healthz").await;
    let headers = response.headers();
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert!(headers.get("referrer-policy").is_some());
    assert!(headers.get("content-security-policy").is_some());
}

#[tokio::test]
async fn test_fallback_404() {
    let app = common::spawn_app().await;
    let response = app.server.get("/nonexistent").await;
    response.assert_status(StatusCode::NOT_FOUND);
    let body = response.text();
    assert!(
        body.contains("404") || body.contains("Not Found"),
        "body should indicate 404: {}",
        body
    );
}

#[tokio::test]
async fn test_unauthenticated_redirects_to_login() {
    let app = common::spawn_app().await;
    let response = app.server.get("/").await;
    // Should redirect to /login
    response.assert_status(StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn test_authenticated_with_config_token() {
    let mut app = common::spawn_app_with_token("test-token").await;

    // Mock the upstream APIs the dashboard calls
    let _me = app
        .mock_server
        .mock("GET", "/api/v0/me")
        .match_header("Authorization", "Token test-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"username":"testuser"}"#)
        .create_async()
        .await;

    let _record = app
        .mock_server
        .mock("GET", "/api/v0/record")
        .match_header("Authorization", "Token test-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"hosts":{}}"#)
        .create_async()
        .await;

    let _health = app
        .mock_server
        .mock("GET", "/healthz")
        .with_status(200)
        .with_body("Ok")
        .create_async()
        .await;

    let response = app.server.get("/").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(body.contains("testuser"), "should contain username");
}
