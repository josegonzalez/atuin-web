use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../assets/"]
pub struct StaticAssets;

pub async fn serve_asset(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Result<Response, StatusCode> {
    let file = StaticAssets::get(&path).ok_or(StatusCode::NOT_FOUND)?;

    let mime = mime_guess::from_path(&path).first_or_octet_stream();

    let cache_control = if cfg!(debug_assertions) {
        "no-cache".to_string()
    } else {
        "public, max-age=31536000, immutable".to_string()
    };

    Ok((
        [
            (header::CONTENT_TYPE, mime.as_ref().to_string()),
            (header::CACHE_CONTROL, cache_control),
        ],
        file.data,
    )
        .into_response())
}
