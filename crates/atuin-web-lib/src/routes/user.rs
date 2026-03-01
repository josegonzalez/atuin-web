use axum::extract::{Path, State};
use axum::response::Html;

use crate::app::AppState;
use crate::error::WebError;
use crate::templates;

pub async fn get(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Html<String>, WebError> {
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
