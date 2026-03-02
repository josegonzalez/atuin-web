use std::sync::Arc;

use atuin_web::app::{self, AppState};
use atuin_web::client::AtuinClient;
use atuin_web::config::Config;
use atuin_web::templates;
use clap::Parser;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_memory_store::MemoryStore;

#[tokio::main]
async fn main() {
    let config = Config::parse();

    if config.healthcheck {
        let url = format!("http://{}/healthz", config.bind);
        let ok = reqwest::get(&url)
            .await
            .is_ok_and(|r| r.status().is_success());
        std::process::exit(if ok { 0 } else { 1 });
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.parse().unwrap_or_default()),
        )
        .init();

    tracing::info!("Starting atuin-web on {}", config.bind);
    tracing::info!("Proxying to atuin server at {}", config.atuin_server_url);

    if config.token.is_some() {
        tracing::info!("Using pre-configured auth token");

        let bind_host = config.bind.split(':').next().unwrap_or("");
        if bind_host != "127.0.0.1" && bind_host != "localhost" && bind_host != "::1" {
            tracing::warn!(
                "Config token is set with non-localhost bind address '{}'; all visitors will have unauthenticated access",
                config.bind
            );
        }
    }

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(
            config.session_expiry as i64,
        )))
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_secure(config.secure_cookies);

    let client = AtuinClient::new(&config.atuin_server_url);
    let env = templates::create_environment();

    let bind_addr = config.bind.clone();

    let state = AppState {
        config,
        client,
        templates: Arc::new(env),
    };

    let app = app::create_router(state).layer(session_layer);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind");

    tracing::info!("Listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("Server error");
}
