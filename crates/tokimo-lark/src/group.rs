use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    GroupService, ImResult, ImError,
    GroupChat, GroupMember, MemberRole, Page,
    CreateGroupRequest, ModifyMembersRequest, SearchGroupRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct CreateChatData {
    chat_id: Option<String>,
}

#[derive(Deserialize)]
struct ChatListData {
    #[serde(default)]
    items: Vec<LarkChat>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct LarkChat {
    chat_id: Option<String>,
    name: Option<String>,
    owner_id: Option<String>,
    description: Option<String>,
    member_count: Option<u32>,
}

#[derive(Deserialize)]
struct MemberListData {
    #[serde(default)]
    items: Vec<LarkMember>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct LarkMember {
    member_id: Option<String>,
    name: Option<String>,
    #[allow(dead_code)]
    member_id_type: Option<String>,
}

impl From<LarkChat> for GroupChat {
    fn from(c: LarkChat) -> Self {
        GroupChat {
            id: c.chat_id.unwrap_or_default(),
            name: c.name.unwrap_or_default(),
            owner_id: c.owner_id,
            member_count: c.member_count,
            description: c.description,
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl GroupService for LarkClient {
    async fn create_group(&self, req: CreateGroupRequest) -> ImResult<GroupChat> {
        let body = serde_json::json!({
            "name": req.name,
            "description": req.description,
            "user_id_list": req.member_ids,
        });
        let resp = self.post("/open-apis/im/v1/chats?user_id_type=open_id", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<CreateChatData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let chat_id = data.data.and_then(|d| d.chat_id).unwrap_or_default();
        Ok(GroupChat {
            id: chat_id,
            name: req.name,
            owner_id: None,
            member_count: Some(req.member_ids.len() as u32),
            description: req.description,
            extra: serde_json::Value::Null,
        })
    }

    async fn search_groups(&self, req: SearchGroupRequest) -> ImResult<Page<GroupChat>> {
        let body = serde_json::json!({
            "query": req.keyword,
            "page_size": req.limit.unwrap_or(20),
            "page_token": req.cursor.unwrap_or_default(),
        });
        let resp = self.post("/open-apis/im/v2/chats/search?user_id_type=open_id", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ChatListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ChatListData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_group(&self, chat_id: &str) -> ImResult<GroupChat> {
        let path = format!("/open-apis/im/v1/chats/{}?user_id_type=open_id", chat_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<LarkChat> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let chat = data.data.ok_or_else(|| ImError::NotFound { resource: chat_id.into() })?;
        Ok(chat.into())
    }

    async fn get_members(&self, chat_id: &str, cursor: Option<&str>) -> ImResult<Page<GroupMember>> {
        let mut path = format!("/open-apis/im/v1/chats/{}/members?member_id_type=open_id", chat_id);
        if let Some(c) = cursor {
            path.push_str(&format!("&page_token={}", c));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<MemberListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(MemberListData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(|m| GroupMember {
                user_id: m.member_id.unwrap_or_default(),
                name: m.name,
                role: MemberRole::Member,
            }).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn add_members(&self, req: ModifyMembersRequest) -> ImResult<()> {
        let body = serde_json::json!({
            "id_list": req.user_ids,
        });
        let path = format!("/open-apis/im/v1/chats/{}/members?member_id_type=open_id", req.chat_id);
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<serde_json::Value> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(())
    }

    async fn remove_members(&self, req: ModifyMembersRequest) -> ImResult<()> {
        let _body = serde_json::json!({
            "id_list": req.user_ids,
        });
        let path = format!("/open-apis/im/v1/chats/{}/members?member_id_type=open_id", req.chat_id);
        let resp = self.delete(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<serde_json::Value> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(())
    }
}
