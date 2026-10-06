use std::io;
use std::sync::{Arc, Mutex};

use atuin_web::config::Config;
use atuin_web::logging::{EcsFormat, LogFormat, ECS_VERSION};
use clap::Parser;
use serde_json::Value;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl io::Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Buffer {
    type Writer = Buffer;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

fn capture_ecs(emit: impl FnOnce()) -> Vec<Value> {
    let buffer = Buffer::default();
    let subscriber = tracing_subscriber::fmt()
        .event_format(EcsFormat)
        .with_writer(buffer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, emit);

    let output = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("each line is one JSON object"))
        .collect()
}

#[test]
fn test_default_log_format_is_text() {
    let config = Config::parse_from::<[&str; 0], &str>([]);
    assert_eq!(config.log_format, LogFormat::Text);
}

#[test]
fn test_log_format_arg() {
    for (arg, expected) in [
        ("text", LogFormat::Text),
        ("json", LogFormat::Json),
        ("ecs", LogFormat::Ecs),
    ] {
        let config = Config::parse_from(["atuin-web", "--log-format", arg]);
        assert_eq!(config.log_format, expected);
    }
}

#[test]
fn test_invalid_log_format_is_rejected() {
    assert!(Config::try_parse_from(["atuin-web", "--log-format", "xml"]).is_err());
}

#[test]
fn test_ecs_format_core_fields() {
    let docs = capture_ecs(|| tracing::info!("Listening on http://{}", "[::]:8080"));
    assert_eq!(docs.len(), 1);
    let doc = &docs[0];

    assert_eq!(doc["message"], "Listening on http://[::]:8080");
    assert_eq!(doc["log.level"], "info");
    assert_eq!(doc["log.logger"], "logging_test");
    assert_eq!(doc["ecs.version"], ECS_VERSION);
    assert_eq!(doc["service.name"], "atuin-web");
    assert_eq!(doc["service.version"], env!("CARGO_PKG_VERSION"));
    assert!(doc["log.origin.file.line"].is_u64());

    let timestamp = doc["@timestamp"].as_str().unwrap();
    assert!(
        timestamp.ends_with('Z'),
        "UTC RFC 3339 timestamp: {timestamp}"
    );
    assert!(doc.get("labels").is_none());
}

#[test]
fn test_ecs_format_puts_extra_fields_in_labels() {
    let docs = capture_ecs(|| tracing::warn!(status = 502, path = "/records", "upstream failed"));
    let doc = &docs[0];

    assert_eq!(doc["message"], "upstream failed");
    assert_eq!(doc["log.level"], "warn");
    assert_eq!(doc["labels"]["status"], "502");
    assert_eq!(doc["labels"]["path"], "/records");
}

#[test]
fn test_ecs_format_escapes_message() {
    let docs = capture_ecs(|| tracing::error!("bad \"quote\"\nand newline"));
    assert_eq!(docs.len(), 1, "a multi-line message stays one JSON line");
    assert_eq!(docs[0]["message"], "bad \"quote\"\nand newline");
}
