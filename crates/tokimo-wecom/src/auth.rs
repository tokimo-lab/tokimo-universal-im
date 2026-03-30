use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{AuthService, ImResult, ImError, AccessToken, Credentials};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct TokenResponse {
    errcode: Option<i64>,
    errmsg: Option<String>,
    access_token: Option<String>,
    expires_in: Option<i64>,
}

#[async_trait]
impl AuthService for WeComClient {
    async fn get_access_token(&self, credentials: &Credentials) -> ImResult<AccessToken> {
        let url = format!(
            "{}/cgi-bin/gettoken?corpid={}&corpsecret={}",
            self.base_url, credentials.client_id, credentials.client_secret
        );
        let resp = self.http.get(&url).send().await
            .map_err(|e| ImError::Network(e.to_string()))?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: TokenResponse = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Auth {
                message: data.errmsg.unwrap_or(text),
            });
        }
        let token_str = data.access_token.ok_or_else(|| ImError::Auth {
            message: "no access_token in response".into(),
        })?;
        self.set_token(token_str.clone()).await;
        let expires_at = data.expires_in.map(|secs| {
            chrono::Utc::now() + chrono::Duration::seconds(secs)
        });
        Ok(AccessToken {
            token: token_str,
            expires_at,
            refresh_token: None,
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> ImResult<AccessToken> {
        // WeCom doesn't use refresh tokens; re-fetch with credentials.
        self.get_access_token(&Credentials {
            client_id: self.corp_id.clone(),
            client_secret: self.corp_secret.clone(),
        }).await
    }
}
