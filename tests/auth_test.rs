mod common;

use atuin_web::auth;
use atuin_web::config::Config;
use axum::http::StatusCode;
use clap::Parser;

#[test]
fn test_get_token_from_config() {
    let config = Config::parse_from(["atuin-web", "--token", "config-token"]);
    let result = auth::get_token_from_config_or_session(&config, None);
    assert_eq!(result, Some("config-token".to_string()));
}

#[test]
fn test_get_token_from_session() {
    let config = Config::parse_from::<[&str; 0], &str>([]);
    let result = auth::get_token_from_config_or_session(&config, Some("session-token".to_string()));
    assert_eq!(result, Some("session-token".to_string()));
}

#[test]
fn test_config_token_takes_priority() {
    let config = Config::parse_from(["atuin-web", "--token", "config-token"]);
    let result = auth::get_token_from_config_or_session(&config, Some("session-token".to_string()));
    assert_eq!(result, Some("config-token".to_string()));
}

#[test]
fn test_no_token_available() {
    let config = Config::parse_from::<[&str; 0], &str>([]);
    let result = auth::get_token_from_config_or_session(&config, None);
    assert!(result.is_none());
}

#[tokio::test]
async fn test_session_token_roundtrip() {
    let mut app = common::spawn_app().await;

    // Login to set a session token
    let _m = app
        .mock_server
        .mock("POST", "/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"session":"roundtrip-tok"}"#)
        .create_async()
        .await;

    let login_resp = app
        .server
        .post("/login")
        .content_type("application/x-www-form-urlencoded")
        .bytes("username=user&password=pass".into())
        .await;
    login_resp.assert_status(StatusCode::SEE_OTHER);

    // Now access a protected route — should work because session has a token
    let _me = app
        .mock_server
        .mock("GET", "/api/v0/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"username":"testuser"}"#)
        .create_async()
        .await;

    let _record = app
        .mock_server
        .mock("GET", "/api/v0/record")
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

    let dashboard_resp = app.server.get("/").await;
    dashboard_resp.assert_status_ok();
}

#[tokio::test]
async fn test_get_session_token_empty() {
    let app = common::spawn_app().await;

    // Without logging in, accessing / should redirect to /login
    let response = app.server.get("/").await;
    response.assert_status(StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn test_clear_session_removes_token() {
    let mut app = common::spawn_app().await;

    // Login
    let _m = app
        .mock_server
        .mock("POST", "/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"session":"tok"}"#)
        .create_async()
        .await;

    app.server
        .post("/login")
        .content_type("application/x-www-form-urlencoded")
        .bytes("username=u&password=p".into())
        .await;

    // Logout clears session
    app.server.post("/logout").await;

    // Should redirect to login
    let response = app.server.get("/").await;
    response.assert_status(StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn test_require_auth_with_config_token() {
    let mut app = common::spawn_app_with_token("cfg-token").await;

    app.mock_server
        .mock("GET", "/api/v0/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"username":"cfguser"}"#)
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

    // Config token should allow access without session
    let response = app.server.get("/").await;
    response.assert_status_ok();
}

#[tokio::test]
async fn test_require_auth_with_session_token() {
    let mut app = common::spawn_app().await;

    // Login to get session token
    let _m = app
        .mock_server
        .mock("POST", "/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"session":"sess-tok"}"#)
        .create_async()
        .await;

    app.server
        .post("/login")
        .content_type("application/x-www-form-urlencoded")
        .bytes("username=u&password=p".into())
        .await;

    // Mock dashboard endpoints
    app.mock_server
        .mock("GET", "/api/v0/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"username":"sessuser"}"#)
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
}

#[tokio::test]
async fn test_require_auth_no_token_redirects() {
    let app = common::spawn_app().await;

    let response = app.server.get("/").await;
    response.assert_status(StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}
