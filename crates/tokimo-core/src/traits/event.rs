use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{ImEvent, RegisterCallbackRequest, EventSubscription};

/// Real-time event subscription (WebSocket / Webhook callbacks).
#[async_trait]
pub trait EventService: Send + Sync {
    /// Register a callback URL for receiving events (webhook mode).
    async fn register_callback(&self, req: RegisterCallbackRequest) -> ImResult<EventSubscription>;

    /// List active event subscriptions.
    async fn list_subscriptions(&self) -> ImResult<Vec<EventSubscription>>;

    /// Delete an event subscription.
    async fn delete_subscription(&self, subscription_id: &str) -> ImResult<()>;

    /// Get available event types that can be subscribed to.
    async fn list_event_types(&self) -> ImResult<Vec<String>>;

    /// Poll for events (for platforms that support polling; returns empty vec if none).
    /// For WebSocket-based platforms (Lark), this may return NotSupported — use
    /// the platform-specific WebSocket client instead.
    async fn poll_events(&self) -> ImResult<Vec<ImEvent>>;
}
