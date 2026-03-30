use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Low-level HTTP client for WeCom API calls.
pub struct WeComClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) access_token: Arc<RwLock<Option<String>>>,
    pub(crate) corp_id: String,
    pub(crate) corp_secret: String,
}

impl WeComClient {
    pub fn new(corp_id: impl Into<String>, corp_secret: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: "https://qyapi.weixin.qq.com".into(),
            access_token: Arc::new(RwLock::new(None)),
            corp_id: corp_id.into(),
            corp_secret: corp_secret.into(),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub async fn set_token(&self, token: String) {
        let mut guard = self.access_token.write().await;
        *guard = Some(token);
    }

    #[allow(dead_code)]
    pub(crate) async fn get(&self, path: &str) -> Result<reqwest::Response, tokimo_core::ImError> {
        let token = self.token_or_err().await?;
        let url = if path.contains('?') {
            format!("{}{}&access_token={}", self.base_url, path, token)
        } else {
            format!("{}{}?access_token={}", self.base_url, path, token)
        };
        self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| tokimo_core::ImError::Network(e.to_string()))
    }

    pub(crate) async fn post(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, tokimo_core::ImError> {
        let token = self.token_or_err().await?;
        let url = if path.contains('?') {
            format!("{}{}&access_token={}", self.base_url, path, token)
        } else {
            format!("{}{}?access_token={}", self.base_url, path, token)
        };
        self.http
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| tokimo_core::ImError::Network(e.to_string()))
    }

    #[allow(dead_code)]
    pub(crate) async fn post_no_auth(
        &self,
        url: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, tokimo_core::ImError> {
        self.http
            .post(url)
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
