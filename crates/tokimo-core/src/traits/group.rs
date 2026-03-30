use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    GroupChat, GroupMember, Page, CreateGroupRequest,
    ModifyMembersRequest, SearchGroupRequest,
};

/// Group chat management operations.
#[async_trait]
pub trait GroupService: Send + Sync {
    /// Create a new group chat.
    async fn create_group(&self, req: CreateGroupRequest) -> ImResult<GroupChat>;

    /// Search for groups by keyword.
    async fn search_groups(&self, req: SearchGroupRequest) -> ImResult<Page<GroupChat>>;

    /// Get detailed info for a group chat.
    async fn get_group(&self, chat_id: &str) -> ImResult<GroupChat>;

    /// List members of a group chat.
    async fn get_members(&self, chat_id: &str, cursor: Option<&str>) -> ImResult<Page<GroupMember>>;

    /// Add members to a group chat.
    async fn add_members(&self, req: ModifyMembersRequest) -> ImResult<()>;

    /// Remove members from a group chat.
    async fn remove_members(&self, req: ModifyMembersRequest) -> ImResult<()>;
}
