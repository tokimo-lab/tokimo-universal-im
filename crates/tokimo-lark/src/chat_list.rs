use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    ChatListService, ImResult, ImError,
    Conversation, ConversationType, Page, ListConversationsRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct ChatListData {
    #[serde(default)]
    items: Vec<LarkChat>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct LarkChat {
    chat_id: Option<String>,
    name: Option<String>,
    chat_type: Option<String>,
}

#[async_trait]
impl ChatListService for LarkClient {
    async fn list_conversations(&self, req: ListConversationsRequest) -> ImResult<Page<Conversation>> {
        let mut path = "/open-apis/im/v1/chats?user_id_type=open_id".to_string();
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        if let Some(limit) = req.limit {
            path.push_str(&format!("&page_size={}", limit));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ChatListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ChatListData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(|c| Conversation {
                chat_id: c.chat_id.unwrap_or_default(),
                name: c.name.unwrap_or_default(),
                chat_type: match c.chat_type.as_deref() {
                    Some("p2p") => ConversationType::Direct,
                    Some("group") => ConversationType::Group,
                    _ => ConversationType::Unknown,
                },
                last_message_time: None,
                message_count: None,
                unread_count: None,
                extra: serde_json::Value::Null,
            }).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }
}
