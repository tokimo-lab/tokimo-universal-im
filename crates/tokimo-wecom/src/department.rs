use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    DepartmentService, ImResult, ImError,
    DepartmentDetail, User, Department, Page,
    ListDepartmentsRequest, ListDepartmentMembersRequest,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct DeptListResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    department: Vec<WcDepartment>,
}

#[derive(Deserialize)]
struct DeptGetResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    department: Option<WcDepartment>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct WcDepartment {
    id: Option<i64>,
    name: Option<String>,
    parentid: Option<i64>,
    order: Option<i64>,
    department_leader: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct MemberListResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    userlist: Vec<WcUser>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct WcUser {
    userid: Option<String>,
    name: Option<String>,
    email: Option<String>,
    mobile: Option<String>,
    avatar: Option<String>,
    #[serde(default)]
    department: Vec<i64>,
    status: Option<i32>,
}

impl From<WcDepartment> for DepartmentDetail {
    fn from(d: WcDepartment) -> Self {
        DepartmentDetail {
            id: d.id.map(|v| v.to_string()).unwrap_or_default(),
            name: d.name.unwrap_or_default(),
            parent_id: d.parentid.map(|v| v.to_string()),
            order: d.order,
            member_count: None,
            has_children: None,
        }
    }
}

impl From<WcUser> for User {
    fn from(u: WcUser) -> Self {
        User {
            id: u.userid.unwrap_or_default(),
            name: u.name.unwrap_or_default(),
            email: u.email,
            phone: u.mobile,
            avatar: u.avatar,
            departments: u.department.into_iter().map(|d| Department {
                id: d.to_string(),
                name: String::new(),
                parent_id: None,
            }).collect(),
            extra: serde_json::Value::Null,
        }
    }
}

fn check_errcode(errcode: Option<i64>, errmsg: Option<String>, raw: String) -> ImResult<()> {
    if errcode.unwrap_or(0) != 0 {
        return Err(ImError::Platform {
            code: errcode.unwrap_or(-1),
            message: errmsg.unwrap_or(raw),
        });
    }
    Ok(())
}

#[async_trait]
impl DepartmentService for WeComClient {
    async fn list_departments(&self, req: ListDepartmentsRequest) -> ImResult<Page<DepartmentDetail>> {
        let parent_id = req.parent_id.as_deref().unwrap_or("0");
        let path = format!("/cgi-bin/department/list?id={}", parent_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: DeptListResp = serde_json::from_str(&text)?;
        check_errcode(data.errcode, data.errmsg, text)?;

        let items: Vec<DepartmentDetail> = data.department.into_iter().map(Into::into).collect();
        Ok(Page {
            items,
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_department(&self, department_id: &str) -> ImResult<DepartmentDetail> {
        let path = format!("/cgi-bin/department/get?id={}", department_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: DeptGetResp = serde_json::from_str(&text)?;
        check_errcode(data.errcode, data.errmsg.clone(), text)?;

        data.department
            .map(Into::into)
            .ok_or_else(|| ImError::NotFound {
                resource: format!("department {}", department_id),
            })
    }

    async fn list_department_members(&self, req: ListDepartmentMembersRequest) -> ImResult<Page<User>> {
        let path = format!(
            "/cgi-bin/user/list?department_id={}&fetch_child=0",
            req.department_id
        );
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: MemberListResp = serde_json::from_str(&text)?;
        check_errcode(data.errcode, data.errmsg, text)?;

        let items: Vec<User> = data.userlist.into_iter().map(Into::into).collect();
        Ok(Page {
            items,
            has_more: false,
            next_cursor: None,
        })
    }
}
