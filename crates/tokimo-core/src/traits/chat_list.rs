use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{Conversation, Page, ListConversationsRequest};

/// Chat list / conversation list operations.
///
/// Retrieve the list of recent conversations, which is essential for
/// pulling message history (you need the chat IDs).
#[async_trait]
pub trait ChatListService: Send + Sync {
    /// List recent conversations.
    async fn list_conversations(&self, req: ListConversationsRequest) -> ImResult<Page<Conversation>>;
}
