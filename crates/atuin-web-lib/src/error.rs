use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error("upstream API error: {0}")]
    Upstream(#[from] reqwest::Error),

    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),

    #[error("not authenticated")]
    Unauthorized,

    #[error("not found")]
    NotFound,

    #[error("{0}")]
    BadRequest(String),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let status = match &self {
            WebError::Upstream(_) => StatusCode::BAD_GATEWAY,
            WebError::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WebError::Unauthorized => StatusCode::UNAUTHORIZED,
            WebError::NotFound => StatusCode::NOT_FOUND,
            WebError::BadRequest(_) => StatusCode::BAD_REQUEST,
        };

        tracing::error!(%status, error = %self);

        (status, self.to_string()).into_response()
    }
}
