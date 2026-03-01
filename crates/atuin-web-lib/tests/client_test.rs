use atuin_web_lib::client::AtuinClient;

#[tokio::test]
async fn test_client_creation() {
    let client = AtuinClient::new("http://localhost:8888");
    // Client should be created without error
    // We can't test actual requests without a server, but we verify construction
    let _ = client;
}

#[tokio::test]
async fn test_client_trailing_slash() {
    let client = AtuinClient::new("http://localhost:8888/");
    // Should handle trailing slash gracefully
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
