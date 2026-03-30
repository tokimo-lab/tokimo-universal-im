use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokimo_core::{AuthService, ImResult, ImError, AccessToken, Credentials};
use crate::client::DingTalkClient;

#[derive(Serialize)]
struct TokenRequest<'a> {
    #[serde(rename = "clientId")]
    client_id: &'a str,
    #[serde(rename = "clientSecret")]
    client_secret: &'a str,
    #[serde(rename = "grantType", skip_serializing_if = "Option::is_none")]
    grant_type: Option<&'a str>,
    #[serde(rename = "refreshToken", skip_serializing_if = "Option::is_none")]
    refresh_token: Option<&'a str>,
}

#[derive(Deserialize)]
struct TokenResponse {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(rename = "expireIn")]
    expire_in: Option<i64>,
}

#[async_trait]
impl AuthService for DingTalkClient {
    async fn get_access_token(&self, credentials: &Credentials) -> ImResult<AccessToken> {
        let body = TokenRequest {
            client_id: &credentials.client_id,
            client_secret: &credentials.client_secret,
            grant_type: Some("authorization_code"),
            refresh_token: None,
        };
        let resp = self
            .post_no_auth("/v1.0/oauth2/userAccessToken", &body)
            .await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Auth { message: text });
        }
        let data: TokenResponse = serde_json::from_str(&text)?;
        let token_str = data.access_token.ok_or_else(|| ImError::Auth {
            message: "no access_token in response".into(),
        })?;
        // Cache the token
        self.set_token(token_str.clone()).await;
        let expires_at = data.expire_in.map(|secs| {
            chrono::Utc::now() + chrono::Duration::seconds(secs)
        });
        Ok(AccessToken {
            token: token_str,
            expires_at,
            refresh_token: data.refresh_token,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> ImResult<AccessToken> {
        let body = TokenRequest {
            client_id: &self.client_id,
            client_secret: &self.client_secret,
            grant_type: Some("refresh_token"),
            refresh_token: Some(refresh_token),
        };
        let resp = self
            .post_no_auth("/v1.0/oauth2/userAccessToken", &body)
            .await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Auth { message: text });
        }
        let data: TokenResponse = serde_json::from_str(&text)?;
        let token_str = data.access_token.ok_or_else(|| ImError::Auth {
            message: "no access_token in refresh response".into(),
        })?;
        self.set_token(token_str.clone()).await;
        let expires_at = data.expire_in.map(|secs| {
            chrono::Utc::now() + chrono::Duration::seconds(secs)
        });
        Ok(AccessToken {
            token: token_str,
            expires_at,
            refresh_token: data.refresh_token,
        })
    }
}
