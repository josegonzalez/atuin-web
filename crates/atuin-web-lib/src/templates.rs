use minijinja::Environment;
use rust_embed::Embed;
use std::sync::Arc;

#[derive(Embed)]
#[folder = "../../templates/"]
pub struct TemplateAssets;

pub fn create_environment() -> Environment<'static> {
    let mut env = Environment::new();

    // Try loading templates from disk first (for dev hot-reload)
    #[cfg(all(debug_assertions, not(feature = "embed-templates")))]
    {
        let templates_dir = std::path::Path::new("templates");
        if templates_dir.exists() {
            env.set_loader(minijinja::path_loader("templates"));
            return env;
        }
    }

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
