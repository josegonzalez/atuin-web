use std::time::Duration;

use reqwest::Client;
use serde_json::Value;
use tracing::debug;

use crate::error::WebError;

#[derive(Clone)]
pub struct AtuinClient {
    http: Client,
    base_url: String,
}

impl AtuinClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<String, WebError> {
        let url = format!("{}/login", self.base_url);
        debug!(url = %url, "POST request");
        let resp = self
            .http
            .post(&url)
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await?;
        debug!(url = %url, status = %resp.status(), "POST response");

        if !resp.status().is_success() {
            return Err(WebError::BadRequest("Invalid credentials".into()));
        }

        let body: Value = resp.json().await?;
        body["session"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| WebError::BadRequest("No session token in response".into()))
    }

    pub async fn get(&self, path: &str, token: &str) -> Result<Value, WebError> {
        let url = format!("{}{}", self.base_url, path);
        debug!(url = %url, "GET request");
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Token {}", token))
            .send()
            .await?;
        debug!(url = %url, status = %resp.status(), "GET response");

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(WebError::Unauthorized);
        }

        if !resp.status().is_success() {
            return Err(WebError::BadRequest(format!(
                "API returned {}",
                resp.status()
            )));
        }

        Ok(resp.json().await?)
    }

    pub async fn get_text(&self, path: &str, token: &str) -> Result<String, WebError> {
        let url = format!("{}{}", self.base_url, path);
        debug!(url = %url, "GET text request");
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Token {}", token))
            .send()
            .await?;
        debug!(url = %url, status = %resp.status(), "GET text response");

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(WebError::Unauthorized);
        }

        if !resp.status().is_success() {
            return Err(WebError::BadRequest(format!(
                "API returned {}",
                resp.status()
            )));
        }

        Ok(resp.text().await?)
    }

    pub async fn healthz(&self) -> Result<String, WebError> {
        let url = format!("{}/healthz", self.base_url);
        debug!(url = %url, "GET healthz request");
        let resp = self.http.get(&url).send().await?;
        debug!(url = %url, status = %resp.status(), "GET healthz response");

        Ok(resp.text().await?)
    }
}
