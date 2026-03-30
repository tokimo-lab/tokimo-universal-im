use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MessagingService, ImResult, ImError,
    Message, MessageContent, MessageSender, TextContent,
    Page, SendMessageRequest, SendMessageResponse,
    ListMessagesRequest, RecallMessageRequest, ChatTarget,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct WeComResponse {
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ChatListResponse {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    chats: Vec<serde_json::Value>,
    next_cursor: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct MessageListResponse {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    messages: Vec<RawMessage>,
    next_cursor: Option<String>,
}

#[derive(Deserialize)]
struct RawMessage {
    #[serde(default)]
    userid: String,
    #[serde(default)]
    send_time: String,
    #[serde(default)]
    msgtype: String,
    text: Option<TextBody>,
    image: Option<MediaBody>,
    file: Option<MediaBody>,
    voice: Option<MediaBody>,
    video: Option<MediaBody>,
}

#[derive(Deserialize)]
struct TextBody {
    content: Option<String>,
}

#[derive(Deserialize)]
struct MediaBody {
    media_id: Option<String>,
    name: Option<String>,
}

impl From<RawMessage> for Message {
    fn from(m: RawMessage) -> Self {
        let content = match m.msgtype.as_str() {
            "text" => MessageContent::Text(TextContent {
                text: m.text.and_then(|t| t.content).unwrap_or_default(),
                mentions: vec![],
            }),
            "image" => MessageContent::Image(tokimo_core::ImageContent {
                media_key: m.image.as_ref().and_then(|i| i.media_id.clone()).unwrap_or_default(),
                name: m.image.and_then(|i| i.name),
                url: None,
            }),
            "file" => MessageContent::File(tokimo_core::FileContent {
                media_key: m.file.as_ref().and_then(|f| f.media_id.clone()).unwrap_or_default(),
                name: m.file.and_then(|f| f.name),
                size: None,
                mime_type: None,
            }),
            "voice" => MessageContent::Audio(tokimo_core::AudioContent {
                media_key: m.voice.and_then(|v| v.media_id).unwrap_or_default(),
                duration_ms: None,
            }),
            "video" => MessageContent::Video(tokimo_core::VideoContent {
                media_key: m.video.as_ref().and_then(|v| v.media_id.clone()).unwrap_or_default(),
                name: m.video.and_then(|v| v.name),
                cover_key: None,
            }),
            other => MessageContent::Unknown {
                msg_type: other.to_string(),
                raw: serde_json::Value::Null,
            },
        };

        let ts = chrono::NaiveDateTime::parse_from_str(&m.send_time, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc().timestamp_millis())
            .unwrap_or(0);

        Message {
            id: String::new(), // WeCom doesn't return per-message IDs
            chat_id: String::new(),
            sender: MessageSender {
                id: m.userid,
                name: None,
                is_bot: false,
            },
            content,
            timestamp: ts,
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl MessagingService for WeComClient {
    async fn send_message(&self, req: SendMessageRequest) -> ImResult<SendMessageResponse> {
        let (chat_type, chatid) = match &req.target {
            ChatTarget::User(id) => (1, id.as_str()),
            ChatTarget::Group(id) => (2, id.as_str()),
        };

        let text_content = match &req.content {
            MessageContent::Text(tc) => tc.text.clone(),
            MessageContent::Markdown(md) => md.text.clone(),
            _ => {
                return Err(ImError::NotSupported {
                    feature: "only text messages can be sent via WeCom bot API".into(),
                    platform: "wecom".into(),
                });
            }
        };

        let body = serde_json::json!({
            "chat_type": chat_type,
            "chatid": chatid,
            "msgtype": "text",
            "text": { "content": text_content },
        });

        let resp = self.post("/cgi-bin/message/send", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: WeComResponse = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.errcode.unwrap_or(-1),
                message: data.errmsg.unwrap_or(text),
            });
        }

        Ok(SendMessageResponse {
            message_id: String::new(), // WeCom doesn't return a message ID
            extra: serde_json::Value::Null,
        })
    }

    async fn list_messages(&self, req: ListMessagesRequest) -> ImResult<Page<Message>> {
        let start = req.start_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default();
        let end = req.end_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default();

        let body = serde_json::json!({
            "chat_type": 2, // default to group; adjust per use case
            "chatid": req.chat_id,
            "begin_time": start,
            "end_time": end,
            "cursor": req.cursor.unwrap_or_default(),
        });

        let resp = self.post("/cgi-bin/message/get_message", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: MessageListResponse = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.errcode.unwrap_or(-1),
                message: data.errmsg.unwrap_or(text),
            });
        }

        Ok(Page {
            items: data.messages.into_iter().map(Into::into).collect(),
            has_more: data.next_cursor.is_some(),
            next_cursor: data.next_cursor,
        })
    }

    async fn recall_message(&self, _req: RecallMessageRequest) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "message recall".into(),
            platform: "wecom".into(),
        })
    }
}
