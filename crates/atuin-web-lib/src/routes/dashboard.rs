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
    // Sum all "history" values across hosts for the total count,
    // and pass hosts as the sync status breakdown
    let (count, status) = match records_res {
        Ok(v) => {
            let mut total: i64 = 0;
            if let Some(hosts) = v["hosts"].as_object() {
                for (_host_id, tags) in hosts {
                    if let Some(n) = tags["history"].as_i64() {
                        total += n;
                    }
                }
            }
            let count = serde_json::json!({"count": total});
            (count, v)
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to fetch record status from /api/v0/record");
            errors.push(format!("Record status: {}", e));
            (serde_json::Value::default(), serde_json::Value::default())
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
            count => count,
            status => status,
            health => health,
            errors => errors,
            server_url => state.config.atuin_server_url,
            active_page => "dashboard",
            has_config_token => state.config.token.is_some(),
        },
    )?;

    Ok(Html(html))
}
