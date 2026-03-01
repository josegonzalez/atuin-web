use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use tower_sessions::Session;

use crate::app::AppState;
use crate::config::Config;

const SESSION_TOKEN_KEY: &str = "atuin_token";

pub async fn require_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    // Check config token first
    if state.config.token.is_some() {
        return next.run(request).await;
    }

    // Check session token
    match session.get::<String>(SESSION_TOKEN_KEY).await {
        Ok(Some(_)) => next.run(request).await,
        _ => Redirect::to("/login").into_response(),
    }
}

pub fn get_token_from_config_or_session(
    config: &Config,
    session_token: Option<String>,
) -> Option<String> {
    config.token.clone().or(session_token)
}

pub async fn get_session_token(session: &Session) -> Option<String> {
    session
        .get::<String>(SESSION_TOKEN_KEY)
        .await
        .ok()
        .flatten()
}

pub async fn set_session_token(session: &Session, token: &str) -> Result<(), StatusCode> {
    session
        .insert(SESSION_TOKEN_KEY, token.to_string())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn clear_session(session: &Session) -> Result<(), StatusCode> {
    session
        .flush()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
