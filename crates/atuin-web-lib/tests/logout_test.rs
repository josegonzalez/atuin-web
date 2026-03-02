mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn test_logout_redirects_to_login() {
    let mut app = common::spawn_app_with_token("test-token").await;

    // Mock upstream so dashboard can render (need to be authed first)
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

    let response = app.server.post("/logout").await;
    response.assert_status(StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn test_logout_clears_session() {
    let mut app = common::spawn_app().await;

    // Login first to establish a session
    let _mock_login = app
        .mock_server
        .mock("POST", "/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"session":"tok123"}"#)
        .create_async()
        .await;

    let login_response = app
        .server
        .post("/login")
        .content_type("application/x-www-form-urlencoded")
        .bytes("username=user&password=pass".into())
        .await;
    login_response.assert_status(StatusCode::SEE_OTHER);

    // Logout
    let logout_response = app.server.post("/logout").await;
    logout_response.assert_status(StatusCode::SEE_OTHER);

    // Accessing / should redirect to /login since session is cleared
    let response = app.server.get("/").await;
    response.assert_status(StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}
