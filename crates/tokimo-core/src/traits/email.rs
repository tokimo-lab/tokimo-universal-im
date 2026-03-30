use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    Email, Mailbox, Page,
    SendEmailRequest, ListEmailsRequest,
};

/// Email service.
#[async_trait]
pub trait EmailService: Send + Sync {
    /// Send an email.
    async fn send_email(&self, req: SendEmailRequest) -> ImResult<Email>;

    /// List emails in a mailbox/folder.
    async fn list_emails(&self, req: ListEmailsRequest) -> ImResult<Page<Email>>;

    /// Get a single email by ID.
    async fn get_email(&self, email_id: &str) -> ImResult<Email>;

    /// List mailboxes/folders.
    async fn list_mailboxes(&self) -> ImResult<Vec<Mailbox>>;

    /// Mark an email as read.
    async fn mark_as_read(&self, email_id: &str) -> ImResult<()>;

    /// Delete an email.
    async fn delete_email(&self, email_id: &str) -> ImResult<()>;
}
