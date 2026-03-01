use std::sync::Arc;

use atuin_web_lib::client::AtuinClient;
use atuin_web_lib::config::Config;
use atuin_web_lib::templates;
use clap::Parser;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_memory_store::MemoryStore;

#[cfg(feature = "hot-reload")]
#[hot_lib_reloader::hot_module(dylib = "atuin_web_lib")]
mod hot_lib {
    pub use atuin_web_lib::app::AppState;
    pub use axum::Router;

    hot_functions_from_file!("crates/atuin-web-lib/src/app.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
}

#[tokio::main]
async fn main() {
    let config = Config::parse();

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
    }

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(
            config.session_expiry as i64,
        )))
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    let client = AtuinClient::new(&config.atuin_server_url);
    let env = templates::create_environment();

    let bind_addr = config.bind.clone();

    #[cfg(not(feature = "hot-reload"))]
    {
        use atuin_web_lib::app::{self, AppState};

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

    #[cfg(feature = "hot-reload")]
    {
        use hot_lib::AppState;

        let state = AppState {
            config,
            client,
            templates: Arc::new(env),
        };

        loop {
            let router = hot_lib::create_router(state.clone());
            let app = router.layer(session_layer.clone());

            let listener = tokio::net::TcpListener::bind(&bind_addr)
                .await
                .expect("Failed to bind");

            tracing::info!("Listening on http://{}", listener.local_addr().unwrap());

            let subscriber = hot_lib::subscribe();
            let shutdown = async move {
                tokio::task::spawn_blocking(move || {
                    subscriber.wait_for_reload();
                })
                .await
                .ok();
            };

            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown)
                .await
                .expect("Server error");

            tracing::info!("Library reloaded, rebuilding router...");
        }
    }
}
