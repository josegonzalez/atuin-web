use atuin_web::error::WebError;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[tokio::test]
async fn test_upstream_error_returns_502() {
    let err: reqwest::Error = reqwest::get("https://[::0]:1/bad")
        .await
        .expect_err("should fail to connect");
    let web_err = WebError::Upstream(err);
    let response = web_err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn test_template_error_returns_500() {
    let err = minijinja::Error::new(
        minijinja::ErrorKind::InvalidOperation,
        "test template error",
    );
    let web_err = WebError::Template(err);
    let response = web_err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_unauthorized_returns_401() {
    let response = WebError::Unauthorized.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_not_found_returns_404() {
    let response = WebError::NotFound.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_bad_request_returns_400() {
    let response = WebError::BadRequest("something went wrong".into()).into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
