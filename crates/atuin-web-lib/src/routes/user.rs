use axum::extract::{Path, State};
use axum::response::Html;

use crate::app::AppState;
use crate::error::WebError;
use crate::templates;

fn is_valid_username(username: &str) -> bool {
    !username.is_empty()
        && username.len() <= 128
        && username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub async fn get(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Html<String>, WebError> {
    if !is_valid_username(&username) {
        return Err(WebError::BadRequest("Invalid username".into()));
    }

    let client = &state.client;

    let user = client
        .get(&format!("/user/{}", username), "")
        .await
        .unwrap_or_default();

    let html = templates::render(
        &state.templates,
        "user.html",
        minijinja::context! {
            user => user,
            username => username,
        },
    )?;

    Ok(Html(html))
}
