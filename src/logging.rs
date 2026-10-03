use std::fmt;
use std::io::IsTerminal;

use clap::ValueEnum;
use serde_json::{Map, Value};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::{FormatTime, SystemTime};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::EnvFilter;

/// Elastic Common Schema version the `ecs` format targets.
pub const ECS_VERSION: &str = "8.11.0";

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Human-readable lines; colored only when stdout is a terminal
    Text,
    /// One JSON object per line (tracing-subscriber's JSON format)
    Json,
    /// One Elastic Common Schema (ECS) JSON object per line
    Ecs,
}

/// Install the global tracing subscriber. `RUST_LOG` takes precedence over `level`.
pub fn init(level: &str, format: LogFormat) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| level.parse().unwrap_or_default());
    let builder = tracing_subscriber::fmt().with_env_filter(filter);

    match format {
        LogFormat::Text => builder.with_ansi(std::io::stdout().is_terminal()).init(),
        LogFormat::Json => builder.json().flatten_event(true).init(),
        LogFormat::Ecs => builder.event_format(EcsFormat).init(),
    }
}

/// Formats each event as a single ECS JSON line.
///
/// The event's `message` becomes `message`; any other event fields are
/// stringified into `labels`, which ECS reserves for custom key/value pairs.
pub struct EcsFormat;

impl<S, N> FormatEvent<S, N> for EcsFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut timestamp = String::new();
        SystemTime.format_time(&mut Writer::new(&mut timestamp))?;

        let meta = event.metadata();
        let mut fields = FieldVisitor::default();
        event.record(&mut fields);

        let mut doc = Map::new();
        doc.insert("@timestamp".into(), timestamp.into());
        doc.insert(
            "log.level".into(),
            meta.level().as_str().to_ascii_lowercase().into(),
        );
        doc.insert("log.logger".into(), meta.target().into());
        if let Some(file) = meta.file() {
            doc.insert("log.origin.file.name".into(), file.into());
        }
        if let Some(line) = meta.line() {
            doc.insert("log.origin.file.line".into(), line.into());
        }
        doc.insert("message".into(), fields.message.unwrap_or_default().into());
        if !fields.labels.is_empty() {
            doc.insert("labels".into(), Value::Object(fields.labels));
        }
        doc.insert("ecs.version".into(), ECS_VERSION.into());
        doc.insert("service.name".into(), env!("CARGO_PKG_NAME").into());
        doc.insert("service.version".into(), env!("CARGO_PKG_VERSION").into());

        let line = serde_json::to_string(&doc).map_err(|_| fmt::Error)?;
        writeln!(writer, "{line}")
    }
}

#[derive(Default)]
struct FieldVisitor {
    message: Option<String>,
    labels: Map<String, Value>,
}

impl FieldVisitor {
    fn insert(&mut self, field: &Field, value: String) {
        if field.name() == "message" {
            self.message = Some(value);
        } else {
            self.labels.insert(field.name().to_string(), value.into());
        }
    }
}

impl Visit for FieldVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.insert(field, format!("{value:?}"));
    }
}
