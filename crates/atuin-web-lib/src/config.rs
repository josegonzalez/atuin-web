use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "atuin-web", about = "Read-only web UI for Atuin")]
pub struct Config {
    /// Bind address
    #[arg(long, env = "ATUIN_WEB_BIND", default_value = "127.0.0.1:8080")]
    pub bind: String,

    /// Upstream atuin server URL
    #[arg(
        long,
        env = "ATUIN_WEB_SERVER_URL",
        default_value = "http://127.0.0.1:8888"
    )]
    pub atuin_server_url: String,

    /// Pre-configured auth token (skips login)
    #[arg(long, env = "ATUIN_WEB_TOKEN")]
    pub token: Option<String>,

    /// Session expiry in seconds
    #[arg(long, env = "ATUIN_WEB_SESSION_EXPIRY", default_value = "86400")]
    pub session_expiry: u64,

    /// Log level
    #[arg(long, env = "ATUIN_WEB_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    /// Set Secure flag on session cookies (enable when behind HTTPS)
    #[arg(long, env = "ATUIN_WEB_SECURE_COOKIES", default_value = "false")]
    pub secure_cookies: bool,
}
