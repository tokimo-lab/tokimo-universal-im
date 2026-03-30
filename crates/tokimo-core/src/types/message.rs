use serde::{Deserialize, Serialize};

/// Unified message type that works across all platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Platform-specific message ID.
    pub id: String,
    /// The chat/conversation this message belongs to.
    pub chat_id: String,
    /// Who sent the message.
    pub sender: MessageSender,
    /// The content of the message.
    pub content: MessageContent,
    /// When the message was sent (millisecond timestamp).
    pub timestamp: i64,
    /// Platform-specific raw data for advanced usage.
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// Identifies who sent a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSender {
    /// Platform-specific user/bot ID.
    pub id: String,
    /// Display name (if available).
    pub name: Option<String>,
    /// Whether the sender is a bot.
    pub is_bot: bool,
}

/// Unified message content across platforms.
///
/// Each variant maps to a specific message type. Platforms that don't support
/// a given variant will return [`ImError::NotSupported`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageContent {
    /// Plain text message.
    Text(TextContent),
    /// Markdown / rich text message.
    Markdown(MarkdownContent),
    /// Image message.
    Image(ImageContent),
    /// File / attachment message.
    File(FileContent),
    /// Audio / voice message.
    Audio(AudioContent),
    /// Video message.
    Video(VideoContent),
    /// Interactive card (platform-specific JSON).
    Card(serde_json::Value),
    /// Unknown or platform-specific message type with raw JSON.
    Unknown {
        msg_type: String,
        raw: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    #[serde(default)]
    pub mentions: Vec<Mention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownContent {
    pub title: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    /// Platform-specific media key (e.g., `image_key`, `media_id`).
    pub media_key: String,
    pub name: Option<String>,
    /// Direct download URL if available.
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub media_key: String,
    pub name: Option<String>,
    pub size: Option<u64>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioContent {
    pub media_key: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    pub media_key: String,
    pub name: Option<String>,
    pub cover_key: Option<String>,
}

/// An @-mention in a text message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mention {
    /// User ID being mentioned, or "all" for @everyone.
    pub user_id: String,
    /// Display name for the mention.
    pub name: Option<String>,
}

/// Parameters for sending a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub target: super::ChatTarget,
    pub content: MessageContent,
    /// Bot/robot code (required for DingTalk).
    pub bot_id: Option<String>,
    /// Idempotency key to prevent duplicate sends.
    pub idempotency_key: Option<String>,
}

/// Result of a message send operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: String,
    /// Platform-specific extra data (e.g., processQueryKey for DingTalk).
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// Parameters for listing messages in a chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessagesRequest {
    pub chat_id: String,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Parameters for recalling a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallMessageRequest {
    pub message_id: String,
    /// Bot/robot code (required for DingTalk).
    pub bot_id: Option<String>,
    /// Chat/group ID (required for some platforms).
    pub chat_id: Option<String>,
}
