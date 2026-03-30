use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    ChatListService, ImResult, ImError,
    Conversation, ConversationType, Page, ListConversationsRequest,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct ChatListResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    chats: Vec<WcChat>,
    has_more: Option<bool>,
    next_cursor: Option<String>,
}

#[derive(Deserialize)]
struct WcChat {
    chat_id: Option<String>,
    chat_name: Option<String>,
    last_msg_time: Option<String>,
    msg_count: Option<u32>,
}

#[async_trait]
impl ChatListService for WeComClient {
    async fn list_conversations(&self, req: ListConversationsRequest) -> ImResult<Page<Conversation>> {
        let start = req.start_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default();
        let end = req.end_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default();
        let body = serde_json::json!({
            "begin_time": start,
            "end_time": end,
            "cursor": req.cursor.unwrap_or_default(),
        });
        let resp = self.post("/cgi-bin/message/get_chat_list", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: ChatListResp = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.errcode.unwrap_or(-1),
                message: data.errmsg.unwrap_or(text),
            });
        }
        Ok(Page {
            items: data.chats.into_iter().map(|c| {
                let ts = c.last_msg_time.and_then(|s|
                    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()
                ).map(|dt| dt.and_utc().timestamp_millis());
                Conversation {
                    chat_id: c.chat_id.unwrap_or_default(),
                    name: c.chat_name.unwrap_or_default(),
                    chat_type: ConversationType::Group,
                    last_message_time: ts,
                    message_count: c.msg_count,
                    unread_count: None,
                    extra: serde_json::Value::Null,
                }
            }).collect(),
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next_cursor,
        })
    }
}
