use serde::{Deserialize, Serialize};

/// A message pin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePin {
    /// Pin ID.
    pub pin_id: String,
    /// Pinned message ID.
    pub message_id: String,
    /// Chat ID where the message is pinned.
    pub chat_id: String,
    /// User who pinned the message.
    pub operator_id: Option<String>,
    /// When the message was pinned.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A group announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAnnouncement {
    /// Announcement content (may be rich text/doc token).
    pub content: String,
    /// Revision / version.
    pub revision: Option<String>,
    /// Who set the announcement.
    pub operator_id: Option<String>,
    /// When it was last updated.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to set a group announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAnnouncementRequest {
    /// Chat/group ID.
    pub chat_id: String,
    /// Announcement content.
    pub content: String,
}

/// DING notification request (DingTalk-specific but defined generically).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingNotificationRequest {
    /// Target user IDs.
    pub user_ids: Vec<String>,
    /// Notification content.
    pub content: String,
}

/// A department with member info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentDetail {
    /// Department ID.
    pub id: String,
    /// Department name.
    pub name: String,
    /// Parent department ID.
    pub parent_id: Option<String>,
    /// Order within parent.
    pub order: Option<i64>,
    /// Number of members.
    pub member_count: Option<u32>,
    /// Whether this department has sub-departments.
    pub has_children: Option<bool>,
}

/// Request to list departments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDepartmentsRequest {
    /// Parent department ID (None for root departments).
    pub parent_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Request to list department members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDepartmentMembersRequest {
    /// Department ID.
    pub department_id: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
