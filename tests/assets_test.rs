mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn test_serve_favicon() {
    let app = common::spawn_app().await;
    let response = app.server.get("/favicon.ico").await;
    response.assert_status(StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "image/x-icon");
    assert!(headers.get("cache-control").is_some());
}

#[tokio::test]
async fn test_serve_css_asset() {
    let app = common::spawn_app().await;
    let response = app.server.get("/assets/css/app.css").await;
    response.assert_status(StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        content_type.contains("text/css"),
        "expected text/css, got: {}",
        content_type
    );
}

#[tokio::test]
async fn test_serve_js_asset() {
    let app = common::spawn_app().await;
    let response = app.server.get("/assets/js/decrypt.js").await;
    response.assert_status(StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        content_type.contains("javascript"),
        "expected javascript content-type, got: {}",
        content_type
    );
}

#[tokio::test]
async fn test_serve_asset_cache_control_matches_build_profile() {
    let app = common::spawn_app().await;
    let response = app.server.get("/assets/css/app.css").await;
    response.assert_status_ok();
    let cache_control = response
        .headers()
        .get("cache-control")
        .unwrap()
        .to_str()
        .unwrap();
    // serve_asset revalidates every request in debug and caches hard in release.
    let expected = if cfg!(debug_assertions) {
        "no-cache"
    } else {
        "public, max-age=31536000, immutable"
    };
    assert_eq!(
        cache_control, expected,
        "unexpected cache-control for this build profile, got: {}",
        cache_control
    );
}

#[tokio::test]
async fn test_serve_missing_asset_returns_404() {
    let app = common::spawn_app().await;
    let response = app.server.get("/assets/nonexistent.xyz").await;
    response.assert_status(StatusCode::NOT_FOUND);
}
