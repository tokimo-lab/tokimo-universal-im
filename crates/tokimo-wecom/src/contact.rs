use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    ContactService, ImResult, ImError,
    User, Page, SearchUserRequest,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct UserListResponse {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    userlist: Vec<WcUser>,
}

#[derive(Deserialize)]
struct WcUser {
    userid: Option<String>,
    name: Option<String>,
    alias: Option<String>,
}

impl From<WcUser> for User {
    fn from(u: WcUser) -> Self {
        User {
            id: u.userid.unwrap_or_default(),
            name: u.name.unwrap_or_default(),
            email: None,
            phone: None,
            avatar: None,
            departments: vec![],
            extra: u.alias.map(|a| serde_json::json!({"alias": a})).unwrap_or(serde_json::Value::Null),
        }
    }
}

#[async_trait]
impl ContactService for WeComClient {
    async fn get_self(&self) -> ImResult<User> {
        // WeCom bot API doesn't have a direct "get self" — return placeholder
        Err(ImError::NotSupported {
            feature: "get_self (use contact list instead)".into(),
            platform: "wecom".into(),
        })
    }

    async fn search_users(&self, _req: SearchUserRequest) -> ImResult<Page<User>> {
        // WeCom returns all visible contacts; client-side filtering needed
        let body = serde_json::json!({});
        let resp = self.post("/cgi-bin/user/list", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: UserListResponse = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.errcode.unwrap_or(-1),
                message: data.errmsg.unwrap_or(text),
            });
        }
        Ok(Page {
            items: data.userlist.into_iter().map(Into::into).collect(),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_users(&self, _user_ids: &[String]) -> ImResult<Vec<User>> {
        // WeCom doesn't support batch user detail lookup via bot API
        Err(ImError::NotSupported {
            feature: "batch get_users".into(),
            platform: "wecom".into(),
        })
    }
}
