use serde::{Deserialize, Serialize};

/// A conversation / chat entry in the chat list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Platform-specific chat ID.
    pub chat_id: String,
    /// Display name of the conversation.
    pub name: String,
    /// Type of conversation.
    pub chat_type: ConversationType,
    /// Last message time (millisecond timestamp).
    pub last_message_time: Option<i64>,
    /// Number of messages (if available).
    pub message_count: Option<u32>,
    /// Unread count (if available).
    pub unread_count: Option<u32>,
    /// Platform-specific extra data.
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// Type of conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConversationType {
    /// Direct / single chat.
    Direct,
    /// Group chat.
    Group,
    /// Bot / service chat.
    Bot,
    /// Unknown type.
    Unknown,
}

/// Request to list conversations / chat list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListConversationsRequest {
    /// Start time filter.
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// End time filter.
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
