use atuin_web::client::AtuinClient;
use atuin_web::error::WebError;

const TEST_PASSWORD: &str = "test-password";

#[tokio::test]
async fn test_client_creation() {
    let client = AtuinClient::new("http://localhost:8888");
    let _ = client;
}

#[tokio::test]
async fn test_client_trailing_slash() {
    let client = AtuinClient::new("http://localhost:8888/");
    let _ = client;
}

#[tokio::test]
async fn test_healthz_unreachable() {
    let client = AtuinClient::new("http://127.0.0.1:1");
    let result = client.healthz().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_login_unreachable() {
    let client = AtuinClient::new("http://127.0.0.1:1");
    let result = client.login("user", "pass").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_unreachable() {
    let client = AtuinClient::new("http://127.0.0.1:1");
    let result = client.get("/api/v0/me", "token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_login_success() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"session":"tok123"}"#)
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.login("user", TEST_PASSWORD).await;
    assert_eq!(result.unwrap(), "tok123");
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/login")
        .with_status(401)
        .with_body("Unauthorized")
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.login("user", "wrong").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebError::BadRequest(_)));
}

#[tokio::test]
async fn test_login_missing_session_field() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.login("user", "pass").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebError::BadRequest(_)));
}

#[tokio::test]
async fn test_get_success() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/api/v0/me")
        .match_header("Authorization", "Token mytoken")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"username":"testuser"}"#)
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.get("/api/v0/me", "mytoken").await;
    let val = result.unwrap();
    assert_eq!(val["username"], "testuser");
}

#[tokio::test]
async fn test_get_unauthorized() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/api/v0/me")
        .with_status(401)
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.get("/api/v0/me", "badtoken").await;
    assert!(matches!(result.unwrap_err(), WebError::Unauthorized));
}

#[tokio::test]
async fn test_get_bad_status() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/api/v0/me")
        .with_status(500)
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.get("/api/v0/me", "token").await;
    assert!(matches!(result.unwrap_err(), WebError::BadRequest(_)));
}

#[tokio::test]
async fn test_get_text_success() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/some/path")
        .match_header("Authorization", "Token mytoken")
        .with_status(200)
        .with_body("hello text")
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.get_text("/some/path", "mytoken").await;
    assert_eq!(result.unwrap(), "hello text");
}

#[tokio::test]
async fn test_get_text_unauthorized() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/some/path")
        .with_status(401)
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.get_text("/some/path", "badtoken").await;
    assert!(matches!(result.unwrap_err(), WebError::Unauthorized));
}

#[tokio::test]
async fn test_healthz_success() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/healthz")
        .with_status(200)
        .with_body("Ok")
        .create_async()
        .await;

    let client = AtuinClient::new(&server.url());
    let result = client.healthz().await;
    assert_eq!(result.unwrap(), "Ok");
}
