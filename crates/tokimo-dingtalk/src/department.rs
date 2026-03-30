use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    DepartmentService, ImResult, ImError,
    DepartmentDetail, User, Page,
    ListDepartmentsRequest, ListDepartmentMembersRequest,
};
use crate::client::DingTalkClient;

// ── DingTalk response types ──

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtDepartment {
    #[serde(rename = "deptId")]
    dept_id: Option<i64>,
    name: Option<String>,
    #[serde(rename = "parentId")]
    parent_id: Option<i64>,
    order: Option<i64>,
    #[serde(rename = "memberCount")]
    member_count: Option<u32>,
    #[serde(rename = "hasSubDept")]
    has_sub_dept: Option<bool>,
}

impl From<DtDepartment> for DepartmentDetail {
    fn from(d: DtDepartment) -> Self {
        DepartmentDetail {
            id: d.dept_id.map(|id| id.to_string()).unwrap_or_default(),
            name: d.name.unwrap_or_default(),
            parent_id: d.parent_id.map(|id| id.to_string()),
            order: d.order,
            member_count: d.member_count,
            has_children: d.has_sub_dept,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtSubDeptResponse {
    #[serde(default, rename = "deptIdList")]
    dept_id_list: Vec<i64>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtUserListResponse {
    #[serde(default)]
    list: Vec<DtDeptUser>,
    #[serde(rename = "hasMore")]
    has_more: Option<bool>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtDeptUser {
    #[serde(rename = "userId")]
    user_id: Option<String>,
    name: Option<String>,
    email: Option<String>,
    mobile: Option<String>,
    #[serde(rename = "orgAuthEmail")]
    org_auth_email: Option<String>,
}

impl From<DtDeptUser> for User {
    fn from(u: DtDeptUser) -> Self {
        User {
            id: u.user_id.unwrap_or_default(),
            name: u.name.unwrap_or_default(),
            email: u.email.or(u.org_auth_email),
            phone: u.mobile,
            avatar: None,
            departments: vec![],
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl DepartmentService for DingTalkClient {
    async fn list_departments(&self, req: ListDepartmentsRequest) -> ImResult<Page<DepartmentDetail>> {
        let parent_id = req.parent_id.as_deref().unwrap_or("1");
        let path = format!(
            "/v1.0/contact/departments/{}/listSubDepartmentIds",
            parent_id
        );
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }

        let sub_resp: DtSubDeptResponse = serde_json::from_str(&text)?;

        // Fetch details for each sub-department
        let mut items = Vec::new();
        for dept_id in sub_resp.dept_id_list {
            match self.get_department(&dept_id.to_string()).await {
                Ok(detail) => items.push(detail),
                Err(_) => {
                    // If individual fetch fails, add minimal info
                    items.push(DepartmentDetail {
                        id: dept_id.to_string(),
                        name: String::new(),
                        parent_id: Some(parent_id.to_string()),
                        order: None,
                        member_count: None,
                        has_children: None,
                    });
                }
            }
        }

        Ok(Page {
            items,
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_department(&self, department_id: &str) -> ImResult<DepartmentDetail> {
        let path = format!("/v1.0/contact/departments/{}", department_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let d: DtDepartment = serde_json::from_str(&text)?;
        Ok(d.into())
    }

    async fn list_department_members(&self, req: ListDepartmentMembersRequest) -> ImResult<Page<User>> {
        let cursor = req.cursor.as_deref().unwrap_or("0");
        let size = req.limit.unwrap_or(100);
        let body = serde_json::json!({
            "departmentIds": [req.department_id],
            "cursor": cursor,
            "size": size
        });
        let resp = self.post("/v1.0/contact/users/listByDepartmentIds", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtUserListResponse = serde_json::from_str(&text)?;
        Ok(Page {
            items: data.list.into_iter().map(Into::into).collect(),
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next_cursor,
        })
    }
}
