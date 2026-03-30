use serde::{Deserialize, Serialize};

/// A group chat / conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChat {
    /// Platform-specific chat/conversation ID.
    pub id: String,
    /// Group name.
    pub name: String,
    /// Owner user ID.
    pub owner_id: Option<String>,
    /// Number of members.
    pub member_count: Option<u32>,
    /// Description text.
    pub description: Option<String>,
    /// Platform-specific extra data.
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// Member of a group chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub user_id: String,
    pub name: Option<String>,
    pub role: MemberRole,
}

/// Role within a group chat.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

/// Request to create a new group chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub member_ids: Vec<String>,
    pub description: Option<String>,
}

/// Request to add or remove members from a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyMembersRequest {
    pub chat_id: String,
    pub user_ids: Vec<String>,
}

/// Request to search for groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchGroupRequest {
    pub keyword: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
