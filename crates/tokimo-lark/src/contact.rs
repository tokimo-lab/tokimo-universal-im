use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    ContactService, ImResult, ImError,
    User, Page, SearchUserRequest, Department,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct UserInfoData {
    user_id: Option<String>,
    name: Option<String>,
    email: Option<String>,
    mobile: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct SearchData {
    #[serde(default)]
    users: Vec<LarkUser>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct LarkUser {
    user_id: Option<String>,
    open_id: Option<String>,
    name: Option<String>,
    email: Option<String>,
    mobile: Option<String>,
    avatar: Option<AvatarInfo>,
    #[serde(default)]
    department_ids: Vec<String>,
}

#[derive(Deserialize)]
struct AvatarInfo {
    avatar_72: Option<String>,
}

impl From<LarkUser> for User {
    fn from(u: LarkUser) -> Self {
        User {
            id: u.open_id.or(u.user_id).unwrap_or_default(),
            name: u.name.unwrap_or_default(),
            email: u.email,
            phone: u.mobile,
            avatar: u.avatar.and_then(|a| a.avatar_72),
            departments: u.department_ids.into_iter().map(|id| Department {
                id,
                name: String::new(),
                parent_id: None,
            }).collect(),
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl ContactService for LarkClient {
    async fn get_self(&self) -> ImResult<User> {
        let resp = self.get("/open-apis/authen/v1/user_info").await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<UserInfoData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let u = data.data.ok_or_else(|| ImError::Internal("empty user data".into()))?;
        Ok(User {
            id: u.user_id.unwrap_or_default(),
            name: u.name.unwrap_or_default(),
            email: u.email,
            phone: u.mobile,
            avatar: u.avatar_url,
            departments: vec![],
            extra: serde_json::Value::Null,
        })
    }

    async fn search_users(&self, req: SearchUserRequest) -> ImResult<Page<User>> {
        let body = serde_json::json!({
            "query": req.keyword,
            "page_size": req.limit.unwrap_or(20),
            "page_token": req.cursor.unwrap_or_default(),
        });
        let resp = self.post("/open-apis/search/v1/user", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SearchData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let search = data.data.unwrap_or(SearchData { users: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: search.users.into_iter().map(Into::into).collect(),
            has_more: search.has_more.unwrap_or(false),
            next_cursor: search.page_token,
        })
    }

    async fn get_users(&self, user_ids: &[String]) -> ImResult<Vec<User>> {
        let mut users = vec![];
        for id in user_ids {
            let path = format!("/open-apis/contact/v3/users/{}?user_id_type=open_id", id);
            let resp = self.get(&path).await?;
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            let data: LarkResp<serde_json::Value> = serde_json::from_str(&text)?;
            if let Some(user_val) = data.data.and_then(|d| d.get("user").cloned()) {
                let u: LarkUser = serde_json::from_value(user_val)?;
                users.push(u.into());
            }
        }
        Ok(users)
    }
}
