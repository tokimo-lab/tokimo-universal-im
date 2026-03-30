use serde::{Deserialize, Serialize};

/// Webhook message request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessageRequest {
    /// Webhook URL or webhook key/token.
    pub webhook_url: String,
    /// Message content to send.
    pub content: super::MessageContent,
    /// Optional signing secret for request verification (DingTalk/WeCom).
    pub secret: Option<String>,
    /// Optional @mention user IDs.
    #[serde(default)]
    pub at_user_ids: Vec<String>,
    /// Whether to @all.
    #[serde(default)]
    pub at_all: bool,
}

/// Webhook message response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessageResponse {
    pub success: bool,
    pub error_code: Option<i64>,
    pub error_message: Option<String>,
    pub extra: serde_json::Value,
}
