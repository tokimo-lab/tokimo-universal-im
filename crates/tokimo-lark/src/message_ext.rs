use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MessageExtService, ImResult, ImError,
    Message, SendMessageResponse,
    ReplyMessageRequest, ForwardMessageRequest,
    MessageReaction, AddReactionRequest, MessageReadStatus, ReadUser,
    BatchGetMessagesRequest,
};
use crate::client::LarkClient;
use crate::messaging::build_content;

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
struct ReactionData {
    reaction_id: Option<String>,
    #[allow(dead_code)]
    reaction_type: Option<ReactionType>,
    operator: Option<Operator>,
    action_time: Option<String>,
}

#[derive(Deserialize)]
struct ReactionType {
    emoji_type: Option<String>,
}

#[derive(Deserialize)]
struct Operator {
    operator_id: Option<String>,
}

#[derive(Deserialize)]
struct ReactionListData {
    #[serde(default)]
    items: Vec<ReactionItem>,
}

#[derive(Deserialize)]
struct ReactionItem {
    reaction_id: Option<String>,
    reaction_type: Option<ReactionType>,
    operator: Option<Operator>,
    action_time: Option<String>,
}

#[derive(Deserialize)]
struct ReadData {
    #[serde(default)]
    items: Vec<ReadItem>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ReadItem {
    user_id_type: Option<String>,
    user_id: Option<String>,
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct BatchData {
    #[serde(default)]
    items: Vec<serde_json::Value>,
}

#[async_trait]
impl MessageExtService for LarkClient {
    async fn reply_message(&self, req: ReplyMessageRequest) -> ImResult<SendMessageResponse> {
        let (msg_type, content) = build_content(&req.content)?;
        let body = serde_json::json!({
            "msg_type": msg_type,
            "content": content,
        });
        let path = format!("/open-apis/im/v1/messages/{}/reply", req.reply_to_message_id);
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SendData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(SendMessageResponse {
            message_id: data.data.and_then(|d| d.message_id).unwrap_or_default(),
            extra: serde_json::Value::Null,
        })
    }

    async fn forward_message(&self, req: ForwardMessageRequest) -> ImResult<SendMessageResponse> {
        let receive_id = match &req.target {
            tokimo_core::ChatTarget::User(id) => id.as_str(),
            tokimo_core::ChatTarget::Group(id) => id.as_str(),
        };
        let receive_id_type = match &req.target {
            tokimo_core::ChatTarget::User(_) => "open_id",
            tokimo_core::ChatTarget::Group(_) => "chat_id",
        };
        let body = serde_json::json!({
            "receive_id": receive_id,
        });
        let path = format!(
            "/open-apis/im/v1/messages/{}/forward?receive_id_type={}",
            req.message_id, receive_id_type
        );
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SendData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(SendMessageResponse {
            message_id: data.data.and_then(|d| d.message_id).unwrap_or_default(),
            extra: serde_json::Value::Null,
        })
    }

    async fn add_reaction(&self, req: AddReactionRequest) -> ImResult<MessageReaction> {
        let body = serde_json::json!({
            "reaction_type": { "emoji_type": req.emoji_type },
        });
        let path = format!("/open-apis/im/v1/messages/{}/reactions", req.message_id);
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ReactionData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let rd = data.data.unwrap_or(ReactionData {
            reaction_id: None, reaction_type: None, operator: None, action_time: None,
        });
        Ok(MessageReaction {
            reaction_id: rd.reaction_id.unwrap_or_default(),
            message_id: req.message_id,
            emoji_type: req.emoji_type,
            user_id: rd.operator.and_then(|o| o.operator_id).unwrap_or_default(),
            timestamp: rd.action_time.and_then(|s| s.parse().ok()),
        })
    }

    async fn remove_reaction(&self, message_id: &str, reaction_id: &str) -> ImResult<()> {
        let path = format!("/open-apis/im/v1/messages/{}/reactions/{}", message_id, reaction_id);
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

    async fn list_reactions(&self, message_id: &str) -> ImResult<Vec<MessageReaction>> {
        let path = format!("/open-apis/im/v1/messages/{}/reactions", message_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ReactionListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let items = data.data.unwrap_or(ReactionListData { items: vec![] });
        Ok(items.items.into_iter().map(|r| MessageReaction {
            reaction_id: r.reaction_id.unwrap_or_default(),
            message_id: message_id.to_string(),
            emoji_type: r.reaction_type.and_then(|rt| rt.emoji_type).unwrap_or_default(),
            user_id: r.operator.and_then(|o| o.operator_id).unwrap_or_default(),
            timestamp: r.action_time.and_then(|s| s.parse().ok()),
        }).collect())
    }

    async fn get_read_status(&self, message_id: &str) -> ImResult<MessageReadStatus> {
        let path = format!("/open-apis/im/v1/messages/{}/read_users?user_id_type=open_id", message_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ReadData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let items = data.data.unwrap_or(ReadData { items: vec![] });
        let read_count = items.items.len() as u32;
        Ok(MessageReadStatus {
            message_id: message_id.to_string(),
            read_users: items.items.into_iter().map(|r| ReadUser {
                user_id: r.user_id.unwrap_or_default(),
                read_at: r.timestamp.and_then(|s| s.parse().ok()),
            }).collect(),
            total_count: 0, // Not available from this endpoint
            read_count,
        })
    }

    async fn batch_get_messages(&self, req: BatchGetMessagesRequest) -> ImResult<Vec<Message>> {
        let resp = self.get(&format!(
            "/open-apis/im/v1/messages/mget?{}",
            req.message_ids.iter().map(|id| format!("message_ids={}", id)).collect::<Vec<_>>().join("&")
        )).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<BatchData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        // Parse each item as a LarkMessage
        let items = data.data.unwrap_or(BatchData { items: vec![] });
        let mut messages = vec![];
        for item in items.items {
            if let Ok(msg) = serde_json::from_value::<super::messaging::LarkMessage>(item) {
                messages.push(msg.into());
            }
        }
        Ok(messages)
    }
}
