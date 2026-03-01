use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use serde::Deserialize;
use tower_sessions::Session;

use crate::app::AppState;
use crate::auth;
use crate::error::WebError;
use crate::templates;

#[derive(Deserialize, Default)]
pub struct CalendarParams {
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub async fn get(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    Query(params): Query<CalendarParams>,
) -> Result<Html<String>, WebError> {
    let _token = auth::get_token_from_config_or_session(
        &state.config,
        auth::get_session_token(&session).await,
    )
    .ok_or(WebError::Unauthorized)?;

    // The v1 /sync/calendar endpoint requires server-side aggregation of plaintext
    // history data. The v2 records API stores encrypted records, so the server
    // cannot compute calendar aggregates. Return empty data with an informational message.
    let calendar = serde_json::Value::default();

    let is_htmx = headers
        .get("HX-Request")
        .map(|v| v == "true")
        .unwrap_or(false);

    let template = if is_htmx {
        "partials/calendar_grid.html"
    } else {
        "calendar.html"
    };

    let html = templates::render(
        &state.templates,
        template,
        minijinja::context! {
            calendar => calendar,
            year => params.year,
            month => params.month,
            active_page => "calendar",
        },
    )?;

    Ok(Html(html))
}
