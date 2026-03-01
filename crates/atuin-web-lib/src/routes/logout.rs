use axum::response::Redirect;
use tower_sessions::Session;

use crate::auth;
use crate::error::WebError;

pub async fn post(session: Session) -> Result<Redirect, WebError> {
    auth::clear_session(&session)
        .await
        .map_err(|_| WebError::BadRequest("Failed to clear session".into()))?;
    Ok(Redirect::to("/login"))
}
