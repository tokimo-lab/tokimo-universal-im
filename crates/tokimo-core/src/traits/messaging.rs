use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    Message, Page, SendMessageRequest, SendMessageResponse,
    ListMessagesRequest, RecallMessageRequest,
};

/// Core messaging operations — send, receive, recall.
#[async_trait]
pub trait MessagingService: Send + Sync {
    /// Send a message to a user or group.
    async fn send_message(&self, req: SendMessageRequest) -> ImResult<SendMessageResponse>;

    /// List / fetch messages from a chat.
    async fn list_messages(&self, req: ListMessagesRequest) -> ImResult<Page<Message>>;

    /// Recall (delete / unsend) a message.
    async fn recall_message(&self, req: RecallMessageRequest) -> ImResult<()>;
}
