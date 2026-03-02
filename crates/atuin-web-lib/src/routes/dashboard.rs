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

    let (me_res, records_res, health_res) = tokio::join!(
        state.client.get("/api/v0/me", &token),
        state.client.get("/api/v0/record", &token),
        state.client.healthz(),
    );

    let mut errors: Vec<String> = Vec::new();

    let health = match health_res {
        Ok(h) => {
            // The atuin server may return plain text "Ok" or JSON {"status":"healthy"}
            if h == "Ok" {
                "Ok".to_string()
            } else if let Ok(json) = serde_json::from_str::<serde_json::Value>(&h) {
                if json["status"].as_str() == Some("healthy") {
                    "Ok".to_string()
                } else {
                    h
                }
            } else {
                h
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to reach atuin server healthz endpoint");
            errors.push(format!("Health check: {}", e));
            String::new()
        }
    };

    let me = match me_res {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "failed to fetch user info from /api/v0/me");
            errors.push(format!("User info: {}", e));
            serde_json::Value::default()
        }
    };

    // /api/v0/record returns {"hosts":{"uuid":{"tag":max_idx,...},...}}
    // Sum values per tag across all hosts for per-tag counts,
    // and pass hosts as the sync status breakdown
    let (counts, status) = match records_res {
        Ok(v) => {
            let mut tag_totals: std::collections::HashMap<String, i64> =
                std::collections::HashMap::new();
            if let Some(hosts) = v["hosts"].as_object() {
                for (_host_id, tags) in hosts {
                    if let Some(tags_obj) = tags.as_object() {
                        for (tag, count) in tags_obj {
                            if let Some(n) = count.as_i64() {
                                *tag_totals.entry(tag.clone()).or_insert(0) += n;
                            }
                        }
                    }
                }
            }
            (tag_totals, v)
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to fetch record status from /api/v0/record");
            errors.push(format!("Record status: {}", e));
            (std::collections::HashMap::new(), serde_json::Value::default())
        }
    };

    let is_htmx = headers
        .get("HX-Request")
        .map(|v| v == "true")
        .unwrap_or(false);

    let template = if is_htmx {
        "partials/dashboard_content.html"
    } else {
        "dashboard.html"
    };

    let html = templates::render(
        &state.templates,
        template,
        minijinja::context! {
            me => me,
            counts => counts,
            status => status,
            health => health,
            errors => errors,
            server_url => state.config.atuin_server_url,
            active_page => "dashboard",
            tag => "",
            has_config_token => state.config.token.is_some(),
        },
    )?;

    Ok(Html(html))
}
