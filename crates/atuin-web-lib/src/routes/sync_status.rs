use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Html;
use tower_sessions::Session;

use crate::app::AppState;
use crate::auth;
use crate::error::WebError;
use crate::templates;

pub async fn get(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Result<Html<String>, WebError> {
    let token = auth::get_token_from_config_or_session(
        &state.config,
        auth::get_session_token(&session).await,
    )
    .ok_or(WebError::Unauthorized)?;

    let status = match state.client.get("/api/v0/record", &token).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "failed to fetch record status from /api/v0/record");
            serde_json::Value::default()
        }
    };

    let is_htmx = headers
        .get("HX-Request")
        .map(|v| v == "true")
        .unwrap_or(false);

    let template = if is_htmx {
        "partials/sync_detail.html"
    } else {
        "sync_status.html"
    };

    let html = templates::render(
        &state.templates,
        template,
        minijinja::context! {
            status => status,
            active_page => "sync",
            has_config_token => state.config.token.is_some(),
        },
    )?;

    Ok(Html(html))
}
