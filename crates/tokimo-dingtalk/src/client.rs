use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Low-level HTTP client for DingTalk API calls.
///
/// Manages access tokens and handles request signing.
pub struct DingTalkClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) access_token: Arc<RwLock<Option<String>>>,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
}

impl DingTalkClient {
    /// Create a new DingTalk client.
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: "https://api.dingtalk.com".into(),
            access_token: Arc::new(RwLock::new(None)),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }

    /// Override the base URL (useful for testing).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the access token directly (skip authentication flow).
    pub async fn set_token(&self, token: String) {
        let mut guard = self.access_token.write().await;
        *guard = Some(token);
    }

    /// Make an authenticated GET request.
    pub(crate) async fn get(&self, path: &str) -> Result<reqwest::Response, tokimo_core::ImError> {
        let token = self.token_or_err().await?;
        let url = format!("{}{}", self.base_url, path);
        self.http
            .get(&url)
            .header("x-acs-dingtalk-access-token", &token)
            .send()
            .await
            .map_err(|e| tokimo_core::ImError::Network(e.to_string()))
    }

    /// Make an authenticated POST request with JSON body.
    pub(crate) async fn post(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, tokimo_core::ImError> {
        let token = self.token_or_err().await?;
        let url = format!("{}{}", self.base_url, path);
        self.http
            .post(&url)
            .header("x-acs-dingtalk-access-token", &token)
            .json(body)
            .send()
            .await
            .map_err(|e| tokimo_core::ImError::Network(e.to_string()))
    }

    /// Make an unauthenticated POST request (for token exchange).
    pub(crate) async fn post_no_auth(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, tokimo_core::ImError> {
        let url = format!("{}{}", self.base_url, path);
        self.http
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| tokimo_core::ImError::Network(e.to_string()))
    }

    async fn token_or_err(&self) -> Result<String, tokimo_core::ImError> {
        self.access_token
            .read()
            .await
            .clone()
            .ok_or_else(|| tokimo_core::ImError::Auth {
                message: "no access token set; call auth first".into(),
            })
    }
}
