use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{WebhookMessageRequest, WebhookMessageResponse};

/// Webhook-based message sending (custom bot webhooks).
#[async_trait]
pub trait WebhookService: Send + Sync {
    /// Send a message via webhook URL.
    async fn send_webhook(&self, req: WebhookMessageRequest) -> ImResult<WebhookMessageResponse>;
}
