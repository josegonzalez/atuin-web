use atuin_web_lib::auth;
use atuin_web_lib::config::Config;
use clap::Parser;

#[test]
fn test_get_token_from_config() {
    let config = Config::parse_from(["atuin-web", "--token", "config-token"]);
    let result = auth::get_token_from_config_or_session(&config, None);
    assert_eq!(result, Some("config-token".to_string()));
}

#[test]
fn test_get_token_from_session() {
    let config = Config::parse_from::<[&str; 0], &str>([]);
    let result =
        auth::get_token_from_config_or_session(&config, Some("session-token".to_string()));
    assert_eq!(result, Some("session-token".to_string()));
}

#[test]
fn test_config_token_takes_priority() {
    let config = Config::parse_from(["atuin-web", "--token", "config-token"]);
    let result =
        auth::get_token_from_config_or_session(&config, Some("session-token".to_string()));
    assert_eq!(result, Some("config-token".to_string()));
}

#[test]
fn test_no_token_available() {
    let config = Config::parse_from::<[&str; 0], &str>([]);
    let result = auth::get_token_from_config_or_session(&config, None);
    assert!(result.is_none());
}
