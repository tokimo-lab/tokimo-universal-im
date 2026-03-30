use async_trait::async_trait;
use tokimo_core::{
    GroupService, ImResult, ImError,
    GroupChat, GroupMember, Page,
    CreateGroupRequest, ModifyMembersRequest, SearchGroupRequest,
};
use crate::client::WeComClient;

#[async_trait]
impl GroupService for WeComClient {
    async fn create_group(&self, _req: CreateGroupRequest) -> ImResult<GroupChat> {
        Err(ImError::NotSupported {
            feature: "create_group".into(),
            platform: "wecom".into(),
        })
    }

    async fn search_groups(&self, _req: SearchGroupRequest) -> ImResult<Page<GroupChat>> {
        // WeCom's get_msg_chat_list returns recent chats, not a search
        Err(ImError::NotSupported {
            feature: "search_groups (use list_messages chat_list instead)".into(),
            platform: "wecom".into(),
        })
    }

    async fn get_group(&self, _chat_id: &str) -> ImResult<GroupChat> {
        Err(ImError::NotSupported {
            feature: "get_group".into(),
            platform: "wecom".into(),
        })
    }

    async fn get_members(&self, _chat_id: &str, _cursor: Option<&str>) -> ImResult<Page<GroupMember>> {
        Err(ImError::NotSupported {
            feature: "get_members".into(),
            platform: "wecom".into(),
        })
    }

    async fn add_members(&self, _req: ModifyMembersRequest) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "add_members".into(),
            platform: "wecom".into(),
        })
    }

    async fn remove_members(&self, _req: ModifyMembersRequest) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "remove_members".into(),
            platform: "wecom".into(),
        })
    }
}
