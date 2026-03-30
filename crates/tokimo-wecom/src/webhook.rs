use async_trait::async_trait;
use tokimo_core::{
    WebhookService, ImResult, ImError,
    WebhookMessageRequest, WebhookMessageResponse, MessageContent,
};
use crate::client::WeComClient;

#[async_trait]
impl WebhookService for WeComClient {
    async fn send_webhook(&self, req: WebhookMessageRequest) -> ImResult<WebhookMessageResponse> {
        let body = build_webhook_body(&req)?;
        let resp = self.post_no_auth(&req.webhook_url, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: serde_json::Value = serde_json::from_str(&text)?;
        let errcode = data.get("errcode").and_then(|v| v.as_i64()).unwrap_or(-1);
        let errmsg = data.get("errmsg").and_then(|v| v.as_str()).unwrap_or("").to_string();

        if errcode != 0 {
            return Ok(WebhookMessageResponse {
                success: false,
                error_code: Some(errcode),
                error_message: Some(errmsg),
                extra: data,
            });
        }

        Ok(WebhookMessageResponse {
            success: true,
            error_code: None,
            error_message: None,
            extra: data,
        })
    }
}

fn build_webhook_body(req: &WebhookMessageRequest) -> ImResult<serde_json::Value> {
    match &req.content {
        MessageContent::Text(tc) => {
            let mut mentioned_list: Vec<String> = req.at_user_ids.clone();
            let mentioned_mobile_list: Vec<String> = if req.at_all {
                if !mentioned_list.contains(&"@all".to_string()) {
                    mentioned_list.push("@all".to_string());
                }
                vec!["@all".to_string()]
            } else {
                vec![]
            };
            Ok(serde_json::json!({
                "msgtype": "text",
                "text": {
                    "content": tc.text,
                    "mentioned_list": mentioned_list,
                    "mentioned_mobile_list": mentioned_mobile_list,
                }
            }))
        }
        MessageContent::Markdown(mc) => {
            Ok(serde_json::json!({
                "msgtype": "markdown",
                "markdown": {
                    "content": mc.text,
                }
            }))
        }
        MessageContent::Image(ic) => {
            // WeCom webhook image requires base64 + md5 in the media_key field.
            // Convention: media_key = "base64:<base64data>:md5:<md5hash>"
            // Or just pass raw base64 as media_key and empty md5.
            let (b64, md5) = parse_image_key(&ic.media_key);
            Ok(serde_json::json!({
                "msgtype": "image",
                "image": {
                    "base64": b64,
                    "md5": md5,
                }
            }))
        }
        other => {
            Err(ImError::NotSupported {
                feature: format!("webhook message type: {:?}", std::mem::discriminant(other)),
                platform: "wecom".into(),
            })
        }
    }
}

/// Parse image key in the format "base64_data:md5_hash" or just base64 data.
fn parse_image_key(key: &str) -> (&str, &str) {
    // Try splitting by last colon to separate base64 from md5
    if let Some(pos) = key.rfind(':') {
        let (b64, md5_with_sep) = key.split_at(pos);
        let md5 = &md5_with_sep[1..];
        // md5 should be 32 hex chars
        if md5.len() == 32 && md5.chars().all(|c| c.is_ascii_hexdigit()) {
            return (b64, md5);
        }
    }
    (key, "")
}
