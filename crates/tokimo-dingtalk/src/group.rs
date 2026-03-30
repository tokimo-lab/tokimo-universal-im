use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    GroupService, ImResult, ImError,
    GroupChat, GroupMember, MemberRole, Page,
    CreateGroupRequest, ModifyMembersRequest, SearchGroupRequest,
};
use crate::client::DingTalkClient;

#[derive(Deserialize)]
struct DtGroup {
    #[serde(rename = "openConversationId")]
    open_conversation_id: Option<String>,
    name: Option<String>,
    #[serde(rename = "ownerUserId")]
    owner_user_id: Option<String>,
    #[serde(rename = "memberCount")]
    member_count: Option<u32>,
}

impl From<DtGroup> for GroupChat {
    fn from(g: DtGroup) -> Self {
        GroupChat {
            id: g.open_conversation_id.unwrap_or_default(),
            name: g.name.unwrap_or_default(),
            owner_id: g.owner_user_id,
            member_count: g.member_count,
            description: None,
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl GroupService for DingTalkClient {
    async fn create_group(&self, req: CreateGroupRequest) -> ImResult<GroupChat> {
        let body = serde_json::json!({
            "name": req.name,
            "userIds": req.member_ids,
        });
        let resp = self.post("/v1.0/im/interconnections/groups", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let g: DtGroup = serde_json::from_str(&text)?;
        Ok(g.into())
    }

    async fn search_groups(&self, req: SearchGroupRequest) -> ImResult<Page<GroupChat>> {
        let body = serde_json::json!({
            "query": req.keyword,
            "cursor": req.cursor.unwrap_or_default(),
        });
        let resp = self.post("/v1.0/im/interconnections/groups/search", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let groups: Vec<DtGroup> = serde_json::from_value(
            val.get("records").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        let next_cursor = val.get("nextCursor").and_then(|v| v.as_str()).map(String::from);
        let has_more = next_cursor.is_some();
        Ok(Page {
            items: groups.into_iter().map(Into::into).collect(),
            has_more,
            next_cursor,
        })
    }

    async fn get_group(&self, chat_id: &str) -> ImResult<GroupChat> {
        let resp = self
            .get(&format!("/v1.0/im/interconnections/groups/{}", chat_id))
            .await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let g: DtGroup = serde_json::from_str(&text)?;
        Ok(g.into())
    }

    async fn get_members(&self, chat_id: &str, cursor: Option<&str>) -> ImResult<Page<GroupMember>> {
        let body = serde_json::json!({
            "openConversationId": chat_id,
            "cursor": cursor.unwrap_or(""),
        });
        let resp = self.post("/v1.0/im/interconnections/groups/members", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let members: Vec<serde_json::Value> = serde_json::from_value(
            val.get("memberUserIds").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        let items: Vec<GroupMember> = members.iter().filter_map(|m| {
            m.as_str().map(|id| GroupMember {
                user_id: id.to_string(),
                name: None,
                role: MemberRole::Member,
            })
        }).collect();
        let next_cursor = val.get("nextCursor").and_then(|v| v.as_str()).map(String::from);
        Ok(Page {
            has_more: next_cursor.is_some(),
            items,
            next_cursor,
        })
    }

    async fn add_members(&self, req: ModifyMembersRequest) -> ImResult<()> {
        let body = serde_json::json!({
            "openConversationId": req.chat_id,
            "userIds": req.user_ids,
        });
        let resp = self.post("/v1.0/im/interconnections/groups/members/add", &body).await?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }

    async fn remove_members(&self, req: ModifyMembersRequest) -> ImResult<()> {
        let body = serde_json::json!({
            "openConversationId": req.chat_id,
            "userIds": req.user_ids,
        });
        let resp = self.post("/v1.0/im/interconnections/groups/members/remove", &body).await?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }
}
