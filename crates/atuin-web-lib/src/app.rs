use axum::extract::{DefaultBodyLimit, OriginalUri, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use minijinja::Environment;
use std::sync::Arc;

use crate::assets;
use crate::auth::require_auth;
use crate::client::AtuinClient;
use crate::config::Config;
use crate::routes;
use crate::templates;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub client: AtuinClient,
    pub templates: Arc<Environment<'static>>,
}

async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("x-frame-options", "DENY".parse().unwrap());
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        "content-security-policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:".parse().unwrap(),
    );
    response
}

pub fn create_router(state: AppState) -> Router {
    let authed = Router::new()
        .route("/", get(routes::dashboard::get))
        .route("/records", get(routes::records::get))
        .route("/logout", post(routes::logout::post))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let login = Router::new()
        .route("/login", get(routes::login::get).post(routes::login::post))
        .layer(DefaultBodyLimit::max(16_384));

    let public = Router::new()
        .merge(login)
        .route("/assets/{*path}", get(assets::serve_asset))
        .route("/favicon.ico", get(assets::serve_favicon));

    Router::new()
        .merge(authed)
        .merge(public)
        .fallback(fallback_404)
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}

async fn fallback_404(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
) -> impl IntoResponse {
    let html = templates::render(
        &state.templates,
        "404.html",
        minijinja::context! { path => uri.path() },
    );

    match html {
        Ok(body) => (StatusCode::NOT_FOUND, Html(body)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "404 — Page Not Found").into_response(),
    }
}
