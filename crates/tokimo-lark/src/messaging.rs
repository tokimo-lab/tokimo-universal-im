use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MessagingService, ImResult, ImError,
    Message, MessageContent, MessageSender, TextContent, MarkdownContent,
    ImageContent, FileContent, AudioContent, VideoContent,
    Page, SendMessageRequest, SendMessageResponse,
    ListMessagesRequest, RecallMessageRequest, ChatTarget,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct SendData {
    message_id: Option<String>,
}

#[derive(Deserialize)]
struct ListData {
    #[serde(default)]
    items: Vec<LarkMessage>,
    has_more: Option<bool>,
    page_token: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct LarkMessage {
    message_id: Option<String>,
    chat_id: Option<String>,
    msg_type: Option<String>,
    body: Option<MessageBody>,
    sender: Option<LarkSender>,
    create_time: Option<String>,
}

#[derive(Deserialize)]
struct MessageBody {
    content: Option<String>,
}

#[derive(Deserialize)]
struct LarkSender {
    id: Option<String>,
    sender_type: Option<String>,
}

fn parse_lark_content(msg_type: &str, raw: &str) -> MessageContent {
    let val: serde_json::Value = serde_json::from_str(raw).unwrap_or(serde_json::Value::Null);
    match msg_type {
        "text" => MessageContent::Text(TextContent {
            text: val.get("text").and_then(|v| v.as_str()).unwrap_or(raw).to_string(),
            mentions: vec![],
        }),
        "post" | "markdown" => MessageContent::Markdown(MarkdownContent {
            title: val.get("title").and_then(|v| v.as_str()).map(String::from),
            text: val.get("text").and_then(|v| v.as_str())
                .or_else(|| val.get("content").and_then(|v| v.as_str()))
                .unwrap_or(raw)
                .to_string(),
        }),
        "image" => MessageContent::Image(ImageContent {
            media_key: val.get("image_key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            name: None,
            url: None,
        }),
        "file" => MessageContent::File(FileContent {
            media_key: val.get("file_key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            name: val.get("file_name").and_then(|v| v.as_str()).map(String::from),
            size: None,
            mime_type: None,
        }),
        "audio" => MessageContent::Audio(AudioContent {
            media_key: val.get("file_key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            duration_ms: None,
        }),
        "media" => MessageContent::Video(VideoContent {
            media_key: val.get("file_key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            name: None,
            cover_key: val.get("image_key").and_then(|v| v.as_str()).map(String::from),
        }),
        "interactive" => {
            MessageContent::Card(val)
        }
        other => MessageContent::Unknown {
            msg_type: other.to_string(),
            raw: val,
        },
    }
}

impl From<LarkMessage> for Message {
    fn from(m: LarkMessage) -> Self {
        let msg_type = m.msg_type.as_deref().unwrap_or("text");
        let raw_content = m.body.and_then(|b| b.content).unwrap_or_default();
        let content = parse_lark_content(msg_type, &raw_content);
        let ts: i64 = m.create_time.as_deref().unwrap_or("0").parse().unwrap_or(0);

        Message {
            id: m.message_id.unwrap_or_default(),
            chat_id: m.chat_id.unwrap_or_default(),
            sender: MessageSender {
                id: m.sender.as_ref().and_then(|s| s.id.clone()).unwrap_or_default(),
                name: None,
                is_bot: m.sender.as_ref().and_then(|s| s.sender_type.as_deref()) == Some("app"),
            },
            content,
            timestamp: ts,
            extra: serde_json::Value::Null,
        }
    }
}

pub(crate) fn build_content(content: &MessageContent) -> ImResult<(&'static str, String)> {
    match content {
        MessageContent::Text(tc) => {
            Ok(("text", serde_json::json!({"text": tc.text}).to_string()))
        }
        MessageContent::Markdown(md) => {
            // Convert to post format for maximum compatibility
            let title = md.title.as_deref().unwrap_or("");
            let post = serde_json::json!({
                "zh_cn": {
                    "title": title,
                    "content": [[{"tag": "text", "text": md.text}]]
                }
            });
            Ok(("post", post.to_string()))
        }
        MessageContent::Image(img) => {
            Ok(("image", serde_json::json!({"image_key": img.media_key}).to_string()))
        }
        MessageContent::File(f) => {
            Ok(("file", serde_json::json!({"file_key": f.media_key}).to_string()))
        }
        MessageContent::Audio(a) => {
            Ok(("audio", serde_json::json!({"file_key": a.media_key}).to_string()))
        }
        MessageContent::Video(v) => {
            let mut j = serde_json::json!({"file_key": v.media_key});
            if let Some(ref ck) = v.cover_key {
                j["image_key"] = serde_json::json!(ck);
            }
            Ok(("media", j.to_string()))
        }
        MessageContent::Card(val) => {
            Ok(("interactive", val.to_string()))
        }
        _ => Err(ImError::NotSupported {
            feature: "unknown message type".into(),
            platform: "lark".into(),
        }),
    }
}

#[async_trait]
impl MessagingService for LarkClient {
    async fn send_message(&self, req: SendMessageRequest) -> ImResult<SendMessageResponse> {
        let (msg_type, content) = build_content(&req.content)?;

        let (receive_id_type, receive_id) = match &req.target {
            ChatTarget::User(id) => ("open_id", id.as_str()),
            ChatTarget::Group(id) => ("chat_id", id.as_str()),
        };

        let body = serde_json::json!({
            "receive_id": receive_id,
            "msg_type": msg_type,
            "content": content,
        });

        let path = format!("/open-apis/im/v1/messages?receive_id_type={}", receive_id_type);
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SendData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let msg_id = data.data.and_then(|d| d.message_id).unwrap_or_default();
        Ok(SendMessageResponse {
            message_id: msg_id,
            extra: serde_json::Value::Null,
        })
    }

    async fn list_messages(&self, req: ListMessagesRequest) -> ImResult<Page<Message>> {
        let mut path = format!(
            "/open-apis/im/v1/messages?container_id_type=chat&container_id={}",
            req.chat_id
        );
        if let Some(ref start) = req.start_time {
            path.push_str(&format!("&start_time={}", start.timestamp()));
        }
        if let Some(ref end) = req.end_time {
            path.push_str(&format!("&end_time={}", end.timestamp()));
        }
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        if let Some(limit) = req.limit {
            path.push_str(&format!("&page_size={}", limit));
        }

        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ListData {
            items: vec![],
            has_more: Some(false),
            page_token: None,
        });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn recall_message(&self, req: RecallMessageRequest) -> ImResult<()> {
        let path = format!("/open-apis/im/v1/messages/{}", req.message_id);
        let resp = self.delete(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<serde_json::Value> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(())
    }
}
