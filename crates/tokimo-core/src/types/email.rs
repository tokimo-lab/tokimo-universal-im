use serde::{Deserialize, Serialize};

/// An email message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// Email ID / message-id.
    pub id: String,
    /// Subject.
    pub subject: String,
    /// From address.
    pub from: EmailAddress,
    /// To addresses.
    #[serde(default)]
    pub to: Vec<EmailAddress>,
    /// CC addresses.
    #[serde(default)]
    pub cc: Vec<EmailAddress>,
    /// BCC addresses.
    #[serde(default)]
    pub bcc: Vec<EmailAddress>,
    /// Body content (HTML or plain text).
    pub body: EmailBody,
    /// Whether this email has been read.
    pub is_read: Option<bool>,
    /// Timestamp.
    pub date: Option<chrono::DateTime<chrono::Utc>>,
    /// Attachment metadata.
    #[serde(default)]
    pub attachments: Vec<EmailAttachment>,
}

/// An email address with optional display name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    /// Email address.
    pub address: String,
    /// Display name.
    pub name: Option<String>,
}

/// Email body content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailBody {
    /// Content type: "text/plain" or "text/html".
    pub content_type: String,
    /// Body content.
    pub content: String,
}

/// Email attachment metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    /// Attachment ID.
    pub id: String,
    /// File name.
    pub filename: String,
    /// MIME type.
    pub content_type: Option<String>,
    /// Size in bytes.
    pub size: Option<u64>,
}

/// Request to send an email.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailRequest {
    /// Subject.
    pub subject: String,
    /// To addresses.
    pub to: Vec<EmailAddress>,
    /// CC addresses.
    #[serde(default)]
    pub cc: Vec<EmailAddress>,
    /// BCC addresses.
    #[serde(default)]
    pub bcc: Vec<EmailAddress>,
    /// Body.
    pub body: EmailBody,
}

/// Request to list emails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEmailsRequest {
    /// Mailbox / folder to list from (e.g., "INBOX", "SENT").
    pub mailbox: Option<String>,
    /// Search query.
    pub query: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// A mailbox / folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    /// Mailbox ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Total message count.
    pub total_count: Option<u32>,
    /// Unread message count.
    pub unread_count: Option<u32>,
}
