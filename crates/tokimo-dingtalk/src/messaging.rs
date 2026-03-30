use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokimo_core::{
    MessagingService, ImResult, ImError,
    Message, MessageContent,
    Page, SendMessageRequest, SendMessageResponse,
    ListMessagesRequest, RecallMessageRequest, ChatTarget,
};
use crate::client::DingTalkClient;

// --- Send by Bot ---

#[derive(Serialize)]
struct BotSendRequest<'a> {
    #[serde(rename = "robotCode")]
    robot_code: &'a str,
    #[serde(rename = "openConversationId", skip_serializing_if = "Option::is_none")]
    open_conversation_id: Option<&'a str>,
    #[serde(rename = "userIds", skip_serializing_if = "Option::is_none")]
    user_ids: Option<Vec<&'a str>>,
    #[serde(rename = "msgKey")]
    msg_key: &'a str,
    #[serde(rename = "msgParam")]
    msg_param: String,
}

#[derive(Deserialize)]
struct BotSendResponse {
    #[serde(rename = "processQueryKey")]
    process_query_key: Option<String>,
}

// --- Get messages (via MCP tool call pattern) ---

#[derive(Serialize)]
#[allow(dead_code)]
struct WebhookSendRequest<'a> {
    msgtype: &'a str,
    text: Option<WebhookText<'a>>,
    markdown: Option<WebhookMarkdown<'a>>,
    at: Option<WebhookAt<'a>>,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct WebhookText<'a> {
    content: &'a str,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct WebhookMarkdown<'a> {
    title: &'a str,
    text: &'a str,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct WebhookAt<'a> {
    #[serde(rename = "atMobiles", skip_serializing_if = "Option::is_none")]
    at_mobiles: Option<&'a [String]>,
    #[serde(rename = "atUserIds", skip_serializing_if = "Option::is_none")]
    at_user_ids: Option<&'a [String]>,
    #[serde(rename = "isAtAll")]
    is_at_all: bool,
}

// --- Recall ---

#[derive(Serialize)]
struct BotRecallRequest<'a> {
    #[serde(rename = "robotCode")]
    robot_code: &'a str,
    #[serde(rename = "processQueryKeys")]
    process_query_keys: Vec<&'a str>,
    #[serde(rename = "openConversationId", skip_serializing_if = "Option::is_none")]
    open_conversation_id: Option<&'a str>,
}

#[async_trait]
impl MessagingService for DingTalkClient {
    async fn send_message(&self, req: SendMessageRequest) -> ImResult<SendMessageResponse> {
        let bot_id = req.bot_id.as_deref().ok_or_else(|| ImError::InvalidParam {
            message: "bot_id (robot_code) is required for DingTalk".into(),
        })?;

        let (msg_key, msg_param) = match &req.content {
            MessageContent::Text(tc) => {
                ("sampleText", serde_json::json!({ "content": tc.text }).to_string())
            }
            MessageContent::Markdown(md) => {
                let title = md.title.as_deref().unwrap_or("通知");
                (
                    "sampleMarkdown",
                    serde_json::json!({ "title": title, "text": md.text }).to_string(),
                )
            }
            MessageContent::Image(img) => {
                (
                    "sampleImageMsg",
                    serde_json::json!({ "photoURL": img.url.as_deref().unwrap_or(&img.media_key) }).to_string(),
                )
            }
            _ => {
                return Err(ImError::NotSupported {
                    feature: format!("message type {:?}", std::mem::discriminant(&req.content)),
                    platform: "dingtalk".into(),
                });
            }
        };

        let (conv_id, user_ids) = match &req.target {
            ChatTarget::Group(id) => (Some(id.as_str()), None),
            ChatTarget::User(id) => (None, Some(vec![id.as_str()])),
        };

        let body = BotSendRequest {
            robot_code: bot_id,
            open_conversation_id: conv_id,
            user_ids,
            msg_key,
            msg_param,
        };

        let resp = self.post("/v1.0/robot/oToMessages/batchSend", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(ImError::Platform {
                code: status.as_u16() as i64,
                message: text,
            });
        }

        let data: BotSendResponse = serde_json::from_str(&text).unwrap_or(BotSendResponse {
            process_query_key: None,
        });

        Ok(SendMessageResponse {
            message_id: data.process_query_key.clone().unwrap_or_default(),
            extra: serde_json::json!({
                "processQueryKey": data.process_query_key
            }),
        })
    }

    async fn list_messages(&self, _req: ListMessagesRequest) -> ImResult<Page<Message>> {
        // DingTalk doesn't have a direct "list messages" REST API for bots.
        // Messages are received via callback/webhook. This returns an empty page.
        tracing::warn!("DingTalk does not support list_messages via REST API; use webhook callbacks");
        Ok(Page {
            items: vec![],
            has_more: false,
            next_cursor: None,
        })
    }

    async fn recall_message(&self, req: RecallMessageRequest) -> ImResult<()> {
        let bot_id = req.bot_id.as_deref().ok_or_else(|| ImError::InvalidParam {
            message: "bot_id (robot_code) is required for DingTalk recall".into(),
        })?;

        let body = BotRecallRequest {
            robot_code: bot_id,
            process_query_keys: vec![&req.message_id],
            open_conversation_id: req.chat_id.as_deref(),
        };

        let resp = self.post("/v1.0/robot/otoMessages/batchRecall", &body).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform {
                code: status.as_u16() as i64,
                message: text,
            });
        }
        Ok(())
    }
}
