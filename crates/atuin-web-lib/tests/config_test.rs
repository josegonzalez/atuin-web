use atuin_web_lib::config::Config;
use clap::Parser;

#[test]
fn test_default_config() {
    let config = Config::parse_from::<[&str; 0], &str>([]);
    assert_eq!(config.bind, "127.0.0.1:8080");
    assert_eq!(config.atuin_server_url, "http://127.0.0.1:8888");
    assert!(config.token.is_none());
    assert_eq!(config.session_expiry, 86400);
    assert_eq!(config.log_level, "info");
    assert!(!config.secure_cookies);
}

#[test]
fn test_config_with_args() {
    let config = Config::parse_from([
        "atuin-web",
        "--bind",
        "0.0.0.0:9090",
        "--atuin-server-url",
        "http://remote:8888",
        "--token",
        "test-token-123",
        "--session-expiry",
        "3600",
        "--log-level",
        "debug",
        "--secure-cookies",
    ]);
    assert_eq!(config.bind, "0.0.0.0:9090");
    assert_eq!(config.atuin_server_url, "http://remote:8888");
    assert_eq!(config.token, Some("test-token-123".to_string()));
    assert_eq!(config.session_expiry, 3600);
    assert_eq!(config.log_level, "debug");
    assert!(config.secure_cookies);
}

#[test]
fn test_version_flag() {
    let result = Config::try_parse_from(["atuin-web", "--version"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
}
