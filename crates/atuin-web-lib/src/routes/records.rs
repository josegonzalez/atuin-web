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

    let records = state.client.get("/api/v0/record", &token).await;

    // Fetch record/next for the first host found in the record status
    let next = match &records {
        Ok(status) => {
            if let Some(hosts) = status["hosts"].as_object() {
                if let Some(host_id) = hosts.keys().next() {
                    let path = format!(
                        "/api/v0/record/next?host={}&tag=history&count=100",
                        host_id
                    );
                    match state.client.get(&path, &token).await {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::warn!(error = %e, "failed to fetch next records from /api/v0/record/next");
                            serde_json::Value::default()
                        }
                    }
                } else {
                    serde_json::Value::default()
                }
            } else {
                serde_json::Value::default()
            }
        }
        Err(_) => serde_json::Value::default(),
    };

    let is_htmx = headers
        .get("HX-Request")
        .map(|v| v == "true")
        .unwrap_or(false);

    let template = if is_htmx {
        "partials/record_table.html"
    } else {
        "records.html"
    };

    let html = templates::render(
        &state.templates,
        template,
        minijinja::context! {
            records => match records {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(error = %e, "failed to fetch records from /api/v0/record");
                    serde_json::Value::default()
                }
            },
            next => next,
            active_page => "records",
        },
    )?;

    Ok(Html(html))
}
