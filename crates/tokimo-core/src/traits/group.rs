use async_trait::async_trait;
use crate::error::{ImResult, ImError};
use crate::types::{
    GroupChat, GroupMember, Page, CreateGroupRequest,
    ModifyMembersRequest, SearchGroupRequest, GroupAnnouncement,
    SetAnnouncementRequest,
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

    /// Get the group announcement.
    async fn get_announcement(&self, chat_id: &str) -> ImResult<GroupAnnouncement> {
        let _ = chat_id;
        Err(ImError::NotSupported { feature: "get_announcement".into(), platform: "unknown".into() })
    }

    /// Set/update the group announcement.
    async fn set_announcement(&self, req: SetAnnouncementRequest) -> ImResult<()> {
        let _ = req;
        Err(ImError::NotSupported { feature: "set_announcement".into(), platform: "unknown".into() })
    }

    /// Add a bot to a group.
    async fn add_bot(&self, chat_id: &str, bot_id: &str) -> ImResult<()> {
        let _ = (chat_id, bot_id);
        Err(ImError::NotSupported { feature: "add_bot".into(), platform: "unknown".into() })
    }

    /// Remove a bot from a group.
    async fn remove_bot(&self, chat_id: &str, bot_id: &str) -> ImResult<()> {
        let _ = (chat_id, bot_id);
        Err(ImError::NotSupported { feature: "remove_bot".into(), platform: "unknown".into() })
    }
}
