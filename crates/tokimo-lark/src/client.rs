use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Whether to use Feishu (China) or Lark (International) endpoints.
#[derive(Debug, Clone, Copy)]
pub enum LarkRegion {
    /// `open.feishu.cn` — China mainland
    Feishu,
    /// `open.larksuite.com` — International
    Lark,
}

/// Low-level HTTP client for Lark/Feishu API calls.
pub struct LarkClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) access_token: Arc<RwLock<Option<String>>>,
    pub(crate) token_type: Arc<RwLock<TokenType>>,
    pub(crate) app_id: String,
    pub(crate) app_secret: String,
}

#[derive(Clone, Copy)]
pub(crate) enum TokenType {
    Tenant,
    User,
}

impl LarkClient {
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>, region: LarkRegion) -> Self {
        let base = match region {
            LarkRegion::Feishu => "https://open.feishu.cn",
            LarkRegion::Lark => "https://open.larksuite.com",
        };
        Self {
            http: Client::new(),
            base_url: base.into(),
            access_token: Arc::new(RwLock::new(None)),
            token_type: Arc::new(RwLock::new(TokenType::Tenant)),
            app_id: app_id.into(),
            app_secret: app_secret.into(),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub async fn set_token(&self, token: String, is_user: bool) {
        *self.access_token.write().await = Some(token);
        *self.token_type.write().await = if is_user { TokenType::User } else { TokenType::Tenant };
    }

    pub(crate) async fn get(&self, path: &str) -> Result<reqwest::Response, tokimo_core::ImError> {
        let token = self.token_or_err().await?;
        let url = format!("{}{}", self.base_url, path);
        self.http
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
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
        let url = format!("{}{}", self.base_url, path);
        self.http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await
            .map_err(|e| tokimo_core::ImError::Network(e.to_string()))
    }

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

    pub(crate) async fn delete(&self, path: &str) -> Result<reqwest::Response, tokimo_core::ImError> {
        let token = self.token_or_err().await?;
        let url = format!("{}{}", self.base_url, path);
        self.http
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| tokimo_core::ImError::Network(e.to_string()))
    }

    pub(crate) async fn put(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, tokimo_core::ImError> {
        let token = self.token_or_err().await?;
        let url = format!("{}{}", self.base_url, path);
        self.http
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
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
