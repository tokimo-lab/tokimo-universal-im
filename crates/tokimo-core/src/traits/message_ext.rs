use async_trait::async_trait;
use crate::error::{ImResult, ImError};
use crate::types::{
    SendMessageResponse, ReplyMessageRequest, ForwardMessageRequest,
    MessageReaction, AddReactionRequest, MessageReadStatus,
    Message, BatchGetMessagesRequest, MessagePin, Page,
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

    /// Pin a message in a chat.
    async fn pin_message(&self, message_id: &str) -> ImResult<MessagePin> {
        let _ = message_id;
        Err(ImError::NotSupported { feature: "pin_message".into(), platform: "unknown".into() })
    }

    /// Unpin a message.
    async fn unpin_message(&self, message_id: &str) -> ImResult<()> {
        let _ = message_id;
        Err(ImError::NotSupported { feature: "unpin_message".into(), platform: "unknown".into() })
    }

    /// List pinned messages in a chat.
    async fn list_pins(&self, chat_id: &str) -> ImResult<Page<MessagePin>> {
        let _ = chat_id;
        Err(ImError::NotSupported { feature: "list_pins".into(), platform: "unknown".into() })
    }
}
