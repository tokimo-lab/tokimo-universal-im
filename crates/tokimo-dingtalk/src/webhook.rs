use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokimo_core::{
    WebhookService, ImResult, ImError,
    WebhookMessageRequest, WebhookMessageResponse, MessageContent,
};
use crate::client::DingTalkClient;

type HmacSha256 = Hmac<Sha256>;

fn sign_url(webhook_url: &str, secret: &str) -> Result<String, ImError> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let string_to_sign = format!("{}\n{}", timestamp, secret);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| ImError::Internal(format!("hmac error: {}", e)))?;
    mac.update(string_to_sign.as_bytes());
    let result = mac.finalize().into_bytes();
    let sign = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result);
    let sign_encoded = url::form_urlencoded::byte_serialize(sign.as_bytes()).collect::<String>();

    let sep = if webhook_url.contains('?') { "&" } else { "?" };
    Ok(format!("{}{}&timestamp={}&sign={}", webhook_url, sep, timestamp, sign_encoded))
}

fn build_body(content: &MessageContent, at_user_ids: &[String], at_all: bool) -> serde_json::Value {
    let mut body = match content {
        MessageContent::Text(t) => {
            serde_json::json!({
                "msgtype": "text",
                "text": { "content": t.text }
            })
        }
        MessageContent::Markdown(md) => {
            serde_json::json!({
                "msgtype": "markdown",
                "markdown": {
                    "title": md.title.as_deref().unwrap_or(""),
                    "text": md.text
                }
            })
        }
        _ => {
            serde_json::json!({
                "msgtype": "text",
                "text": { "content": format!("{:?}", content) }
            })
        }
    };

    if !at_user_ids.is_empty() || at_all {
        body["at"] = serde_json::json!({
            "atUserIds": at_user_ids,
            "isAtAll": at_all
        });
    }

    body
}

#[async_trait]
impl WebhookService for DingTalkClient {
    async fn send_webhook(&self, req: WebhookMessageRequest) -> ImResult<WebhookMessageResponse> {
        let url = if let Some(ref secret) = req.secret {
            sign_url(&req.webhook_url, secret)?
        } else {
            req.webhook_url.clone()
        };

        let body = build_body(&req.content, &req.at_user_ids, req.at_all);

        let resp = self.http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;

        let val: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
        let errcode = val.get("errcode").and_then(|v| v.as_i64()).unwrap_or(0);
        let errmsg = val.get("errmsg").and_then(|v| v.as_str()).map(String::from);

        if !status.is_success() || errcode != 0 {
            return Ok(WebhookMessageResponse {
                success: false,
                error_code: Some(errcode),
                error_message: errmsg,
                extra: val,
            });
        }

        Ok(WebhookMessageResponse {
            success: true,
            error_code: None,
            error_message: None,
            extra: val,
        })
    }
}
