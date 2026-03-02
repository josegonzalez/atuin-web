use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use serde::Deserialize;
use tower_sessions::Session;

use crate::app::AppState;
use crate::auth;
use crate::error::WebError;
use crate::templates;

pub async fn get(State(state): State<AppState>, session: Session) -> Result<Response, WebError> {
    // Redirect to dashboard if already authenticated
    if state.config.token.is_some() || auth::get_session_token(&session).await.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    let html = templates::render(
        &state.templates,
        "login.html",
        minijinja::context! {
            error => false,
        },
    )?;
    Ok(Html(html).into_response())
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn post(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Result<Response, WebError> {
    match state.client.login(&form.username, &form.password).await {
        Ok(token) => {
            auth::set_session_token(&session, &token)
                .await
                .map_err(|_| WebError::BadRequest("Failed to store session".into()))?;
            Ok(Redirect::to("/").into_response())
        }
        Err(_) => {
            let html = templates::render(
                &state.templates,
                "login.html",
                minijinja::context! {
                    error => "Invalid username or password",
                },
            )?;
            Ok(Html(html).into_response())
        }
    }
}
