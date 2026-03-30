use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{AuthService, ImResult, ImError, AccessToken, Credentials};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct TenantTokenResp {
    code: Option<i64>,
    msg: Option<String>,
    tenant_access_token: Option<String>,
    expire: Option<i64>,
}

#[async_trait]
impl AuthService for LarkClient {
    async fn get_access_token(&self, credentials: &Credentials) -> ImResult<AccessToken> {
        let body = serde_json::json!({
            "app_id": credentials.client_id,
            "app_secret": credentials.client_secret,
        });
        let resp = self
            .post_no_auth("/open-apis/auth/v3/tenant_access_token/internal", &body)
            .await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: TenantTokenResp = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Auth {
                message: data.msg.unwrap_or(text),
            });
        }
        let token_str = data.tenant_access_token.ok_or_else(|| ImError::Auth {
            message: "no tenant_access_token in response".into(),
        })?;
        self.set_token(token_str.clone(), false).await;
        let expires_at = data.expire.map(|secs| {
            chrono::Utc::now() + chrono::Duration::seconds(secs)
        });
        Ok(AccessToken {
            token: token_str,
            expires_at,
            refresh_token: None,
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> ImResult<AccessToken> {
        // Tenant tokens just re-fetch; user tokens need proper refresh
        self.get_access_token(&Credentials {
            client_id: self.app_id.clone(),
            client_secret: self.app_secret.clone(),
        }).await
    }
}
