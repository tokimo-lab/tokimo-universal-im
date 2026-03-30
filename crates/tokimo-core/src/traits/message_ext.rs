use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    SendMessageResponse, ReplyMessageRequest, ForwardMessageRequest,
    MessageReaction, AddReactionRequest, MessageReadStatus,
    Message, BatchGetMessagesRequest,
};

/// Extended messaging operations beyond basic send/receive/recall.
///
/// Not all platforms support all of these. Check for `None` returns.
#[async_trait]
pub trait MessageExtService: Send + Sync {
    /// Reply to a specific message (creates a threaded reply).
    async fn reply_message(&self, req: ReplyMessageRequest) -> ImResult<SendMessageResponse>;

    /// Forward a message to another chat.
    async fn forward_message(&self, req: ForwardMessageRequest) -> ImResult<SendMessageResponse>;

    /// Add an emoji reaction to a message.
    async fn add_reaction(&self, req: AddReactionRequest) -> ImResult<MessageReaction>;

    /// Remove a reaction from a message.
    async fn remove_reaction(&self, message_id: &str, reaction_id: &str) -> ImResult<()>;

    /// List reactions on a message.
    async fn list_reactions(&self, message_id: &str) -> ImResult<Vec<MessageReaction>>;

    /// Get read status for a message.
    async fn get_read_status(&self, message_id: &str) -> ImResult<MessageReadStatus>;

    /// Batch-fetch messages by their IDs.
    async fn batch_get_messages(&self, req: BatchGetMessagesRequest) -> ImResult<Vec<Message>>;
}
