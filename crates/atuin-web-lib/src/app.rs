use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use minijinja::Environment;
use std::sync::Arc;

use crate::assets;
use crate::auth::require_auth;
use crate::client::AtuinClient;
use crate::config::Config;
use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub client: AtuinClient,
    pub templates: Arc<Environment<'static>>,
}

pub fn create_router(state: AppState) -> Router {
    let authed = Router::new()
        .route("/", get(routes::dashboard::get))
        .route("/records", get(routes::records::get))
        .route("/logout", post(routes::logout::post))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let public = Router::new()
        .route("/login", get(routes::login::get).post(routes::login::post))
        .route("/user/{username}", get(routes::user::get))
        .route("/assets/{*path}", get(assets::serve_asset))
        .route("/favicon.ico", get(assets::serve_favicon));

    Router::new().merge(authed).merge(public).with_state(state)
}
