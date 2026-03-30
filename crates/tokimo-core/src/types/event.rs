use serde::{Deserialize, Serialize};

/// An event received from the IM platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImEvent {
    /// Unique event ID.
    pub id: String,
    /// Event type (e.g., "message.receive_v1", "message_created").
    pub event_type: String,
    /// Timestamp when the event occurred.
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Raw event payload.
    pub payload: serde_json::Value,
}

/// Callback registration request (for webhook-based event subscription).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterCallbackRequest {
    /// Callback URL to receive events.
    pub callback_url: String,
    /// Event types to subscribe to.
    pub event_types: Vec<String>,
    /// Verification token for callback validation.
    pub token: Option<String>,
    /// AES key for encrypting callback data.
    pub aes_key: Option<String>,
}

/// Event subscription info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub id: String,
    pub callback_url: Option<String>,
    pub event_types: Vec<String>,
    pub status: EventSubscriptionStatus,
}

/// Status of an event subscription.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventSubscriptionStatus {
    Active,
    Inactive,
    Failed,
}
