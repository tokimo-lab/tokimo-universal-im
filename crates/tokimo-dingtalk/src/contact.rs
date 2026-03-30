use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    ContactService, ImResult, ImError,
    User, Page, SearchUserRequest, Department,
};
use crate::client::DingTalkClient;

#[derive(Deserialize)]
struct DtUser {
    #[serde(rename = "userId")]
    user_id: Option<String>,
    name: Option<String>,
    email: Option<String>,
    mobile: Option<String>,
    #[serde(rename = "orgAuthEmail")]
    org_auth_email: Option<String>,
    #[serde(default)]
    depts: Vec<DtDept>,
}

#[derive(Deserialize)]
struct DtDept {
    #[serde(rename = "deptId")]
    dept_id: Option<String>,
    #[serde(rename = "deptName")]
    dept_name: Option<String>,
}

impl From<DtUser> for User {
    fn from(u: DtUser) -> Self {
        User {
            id: u.user_id.unwrap_or_default(),
            name: u.name.unwrap_or_default(),
            email: u.email.or(u.org_auth_email),
            phone: u.mobile,
            avatar: None,
            departments: u.depts.into_iter().map(|d| Department {
                id: d.dept_id.unwrap_or_default(),
                name: d.dept_name.unwrap_or_default(),
                parent_id: None,
            }).collect(),
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl ContactService for DingTalkClient {
    async fn get_self(&self) -> ImResult<User> {
        let resp = self.get("/v1.0/contact/users/me").await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let u: DtUser = serde_json::from_str(&text)?;
        Ok(u.into())
    }

    async fn search_users(&self, req: SearchUserRequest) -> ImResult<Page<User>> {
        let body = serde_json::json!({ "keyword": req.keyword });
        let resp = self.post("/v1.0/contact/users/search", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let users: Vec<DtUser> = serde_json::from_value(
            val.get("list").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        Ok(Page {
            items: users.into_iter().map(Into::into).collect(),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_users(&self, user_ids: &[String]) -> ImResult<Vec<User>> {
        let body = serde_json::json!({ "userIds": user_ids });
        let resp = self.post("/v1.0/contact/users/get", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let users: Vec<DtUser> = serde_json::from_value(
            val.get("list").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        Ok(users.into_iter().map(Into::into).collect())
    }
}
