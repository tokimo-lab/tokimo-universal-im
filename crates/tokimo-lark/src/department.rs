use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    DepartmentService, ImResult, ImError,
    DepartmentDetail, User, Page, Department,
    ListDepartmentsRequest, ListDepartmentMembersRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct DeptListData {
    #[serde(default)]
    items: Vec<LarkDept>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct DeptData {
    department: Option<LarkDept>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkDept {
    department_id: Option<String>,
    open_department_id: Option<String>,
    name: Option<String>,
    parent_department_id: Option<String>,
    order: Option<String>,
    member_count: Option<u32>,
    has_child: Option<bool>,
}

impl From<LarkDept> for DepartmentDetail {
    fn from(d: LarkDept) -> Self {
        DepartmentDetail {
            id: d.open_department_id.or(d.department_id).unwrap_or_default(),
            name: d.name.unwrap_or_default(),
            parent_id: d.parent_department_id,
            order: d.order.and_then(|s| s.parse().ok()),
            member_count: d.member_count,
            has_children: d.has_child,
        }
    }
}

#[derive(Deserialize)]
struct UserListData {
    #[serde(default)]
    items: Vec<LarkUser>,
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
impl DepartmentService for LarkClient {
    async fn list_departments(&self, req: ListDepartmentsRequest) -> ImResult<Page<DepartmentDetail>> {
        let parent_id = req.parent_id.as_deref().unwrap_or("0");
        let mut path = format!(
            "/open-apis/contact/v3/departments/{}/children?page_size={}",
            parent_id,
            req.limit.unwrap_or(50),
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<DeptListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(DeptListData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_department(&self, department_id: &str) -> ImResult<DepartmentDetail> {
        let path = format!("/open-apis/contact/v3/departments/{}", department_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<DeptData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let dept = data.data.and_then(|d| d.department).ok_or_else(|| ImError::NotFound {
            resource: department_id.into(),
        })?;
        Ok(dept.into())
    }

    async fn list_department_members(&self, req: ListDepartmentMembersRequest) -> ImResult<Page<User>> {
        let mut path = format!(
            "/open-apis/contact/v3/users/find_by_department?department_id={}&page_size={}",
            req.department_id,
            req.limit.unwrap_or(50),
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<UserListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(UserListData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }
}
