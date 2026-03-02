mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn test_login_page_renders() {
    let app = common::spawn_app().await;
    let response = app.server.get("/login").await;
    response.assert_status_ok();
    let body = response.text();
    assert!(body.contains("Sign in"), "login page should contain 'Sign in'");
}

#[tokio::test]
async fn test_login_page_redirects_when_authed() {
    let app = common::spawn_app_with_token("test-token").await;
    let response = app.server.get("/login").await;
    response.assert_status(StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/");
}

#[tokio::test]
async fn test_login_success_redirects_to_dashboard() {
    let mut app = common::spawn_app().await;

    let _mock = app
        .mock_server
        .mock("POST", "/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"session":"tok123"}"#)
        .create_async()
        .await;

    let response = app
        .server
        .post("/login")
        .content_type("application/x-www-form-urlencoded")
        .bytes("username=testuser&password=testpass".into())
        .await;

    response.assert_status(StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/");
}

#[tokio::test]
async fn test_login_failure_shows_error() {
    let mut app = common::spawn_app().await;

    let _mock = app
        .mock_server
        .mock("POST", "/login")
        .with_status(401)
        .with_body("Unauthorized")
        .create_async()
        .await;

    let response = app
        .server
        .post("/login")
        .content_type("application/x-www-form-urlencoded")
        .bytes("username=testuser&password=wrongpass".into())
        .await;

    response.assert_status_ok();
    let body = response.text();
    assert!(
        body.contains("Invalid username or password"),
        "should show error message, got: {}",
        body
    );
}
