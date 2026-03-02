use minijinja::Environment;
use rust_embed::Embed;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

#[derive(Embed)]
#[folder = "templates/"]
pub struct TemplateAssets;

fn uuid7_timestamp(value: &str) -> String {
    let hex: String = value.chars().filter(|c| *c != '-').collect();
    if hex.len() < 12 {
        return "\u{2014}".to_string();
    }

    let ms = match u64::from_str_radix(&hex[..12], 16) {
        Ok(ms) => ms,
        Err(_) => return "\u{2014}".to_string(),
    };

    let created = SystemTime::UNIX_EPOCH + Duration::from_millis(ms);
    match SystemTime::now().duration_since(created) {
        Ok(ago) => {
            let truncated = Duration::from_secs(ago.as_secs() / 60 * 60);
            format!("{} ago", humantime::format_duration(truncated))
        }
        Err(_) => "\u{2014}".to_string(),
    }
}

fn register_filters(env: &mut Environment<'static>) {
    env.add_filter("uuid7_timestamp", uuid7_timestamp);
}

pub fn create_environment() -> Environment<'static> {
    let mut env = Environment::new();

    #[allow(unused_mut)]
    let mut loaded = false;

    // Try loading templates from disk first (for dev hot-reload)
    #[cfg(all(debug_assertions, not(feature = "embed-templates")))]
    {
        let templates_dir = std::path::Path::new("templates");
        if templates_dir.exists() {
            env.set_loader(minijinja::path_loader("templates"));
            loaded = true;
        }
    }

    if !loaded {
        // Fall back to embedded templates (release mode, tests, or embed-templates feature)
        for path in TemplateAssets::iter() {
            if let Some(file) = TemplateAssets::get(&path) {
                let content = std::str::from_utf8(file.data.as_ref())
                    .expect("template is valid UTF-8")
                    .to_string();
                env.add_template_owned(path.to_string(), content)
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to load template {}: {}", path, e);
                    });
            }
        }
    }

    register_filters(&mut env);

    env
}

pub type TemplateEnv = Arc<Environment<'static>>;

pub fn render(
    env: &Environment<'_>,
    template_name: &str,
    ctx: minijinja::value::Value,
) -> Result<String, minijinja::Error> {
    let tmpl = env.get_template(template_name)?;
    tmpl.render(ctx)
}
