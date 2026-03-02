#![allow(dead_code)]

use atuin_web_lib::app::{AppState, create_router};
use atuin_web_lib::client::AtuinClient;
use atuin_web_lib::config::Config;
use atuin_web_lib::templates;
use axum::Router;
use axum_test::TestServerConfig;
use clap::Parser;
use std::sync::Arc;
use tower_sessions::MemoryStore;
use tower_sessions::SessionManagerLayer;

pub struct TestApp {
    pub server: axum_test::TestServer,
    pub mock_server: mockito::ServerGuard,
}

fn build_router(config: Config, mock_url: &str) -> Router {
    let client = AtuinClient::new(mock_url);
    let env = Arc::new(templates::create_environment());
    let state = AppState {
        config,
        client,
        templates: env,
    };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    create_router(state).layer(session_layer)
}

fn test_config() -> TestServerConfig {
    TestServerConfig {
        save_cookies: true,
        ..Default::default()
    }
}

pub async fn spawn_app() -> TestApp {
    let mock_server = mockito::Server::new_async().await;
    let config = Config::parse_from::<[&str; 0], &str>([]);
    let router = build_router(config, &mock_server.url());
    let server = axum_test::TestServer::new_with_config(router, test_config());
    TestApp {
        server,
        mock_server,
    }
}

pub async fn spawn_app_with_token(token: &str) -> TestApp {
    let mock_server = mockito::Server::new_async().await;
    let config = Config::parse_from(["atuin-web", "--token", token]);
    let router = build_router(config, &mock_server.url());
    let server = axum_test::TestServer::new_with_config(router, test_config());
    TestApp {
        server,
        mock_server,
    }
}
