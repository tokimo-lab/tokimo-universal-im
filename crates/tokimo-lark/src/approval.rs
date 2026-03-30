use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    ApprovalService, ImResult, ImError,
    ApprovalInstance, ApprovalStatus, ApprovalAction, Page,
    CreateApprovalRequest, ListApprovalRequest, ApprovalActionRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct CreateData {
    instance_code: Option<String>,
}

#[derive(Deserialize)]
struct ListData {
    #[serde(default)]
    instance_code_list: Vec<String>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct InstanceData {
    approval_code: Option<String>,
    approval_name: Option<String>,
    instance_code: Option<String>,
    status: Option<String>,
    user_id: Option<String>,
    open_id: Option<String>,
    form: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
}

fn parse_status(s: &str) -> ApprovalStatus {
    match s {
        "PENDING" => ApprovalStatus::Pending,
        "APPROVED" => ApprovalStatus::Approved,
        "REJECTED" => ApprovalStatus::Rejected,
        "CANCELED" | "CANCELLED" => ApprovalStatus::Cancelled,
        "DELETED" => ApprovalStatus::Deleted,
        _ => ApprovalStatus::Pending,
    }
}

fn ts_opt(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let ms: i64 = s.parse().ok()?;
    chrono::DateTime::from_timestamp_millis(ms)
}

#[async_trait]
impl ApprovalService for LarkClient {
    async fn create_approval(&self, req: CreateApprovalRequest) -> ImResult<ApprovalInstance> {
        let body = serde_json::json!({
            "approval_code": req.process_code,
            "open_id": req.initiator_id,
            "form": req.form_data.to_string(),
            "approvers": req.approvers.iter().map(|id| serde_json::json!({"type": "AND", "open_id": id})).collect::<Vec<_>>(),
            "cc_list": req.cc_users.iter().map(|id| serde_json::json!({"open_id": id})).collect::<Vec<_>>(),
        });
        let resp = self.post("/open-apis/approval/v4/instances", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<CreateData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let instance_code = data.data.and_then(|d| d.instance_code).unwrap_or_default();
        Ok(ApprovalInstance {
            id: instance_code,
            process_code: req.process_code,
            title: String::new(),
            status: ApprovalStatus::Pending,
            initiator_id: req.initiator_id,
            form_data: req.form_data,
            created_at: Some(chrono::Utc::now()),
            finished_at: None,
        })
    }

    async fn list_approvals(&self, req: ListApprovalRequest) -> ImResult<Page<ApprovalInstance>> {
        let approval_code = req.process_code.as_deref().unwrap_or_default();
        let mut path = format!(
            "/open-apis/approval/v4/instances?approval_code={}&page_size={}",
            approval_code,
            req.limit.unwrap_or(20),
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ListData {
            instance_code_list: vec![], page_token: None, has_more: None,
        });
        // The list endpoint only returns instance codes; create stubs.
        let items: Vec<ApprovalInstance> = list.instance_code_list.into_iter().map(|code| {
            ApprovalInstance {
                id: code,
                process_code: approval_code.to_string(),
                title: String::new(),
                status: ApprovalStatus::Pending,
                initiator_id: String::new(),
                form_data: serde_json::Value::Null,
                created_at: None,
                finished_at: None,
            }
        }).collect();
        Ok(Page {
            items,
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_approval(&self, instance_id: &str) -> ImResult<ApprovalInstance> {
        let path = format!("/open-apis/approval/v4/instances/{}", instance_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<InstanceData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let inst = data.data.ok_or_else(|| ImError::NotFound {
            resource: instance_id.into(),
        })?;
        let form_data = inst.form.as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(serde_json::Value::Null);
        Ok(ApprovalInstance {
            id: inst.instance_code.unwrap_or_else(|| instance_id.to_string()),
            process_code: inst.approval_code.unwrap_or_default(),
            title: inst.approval_name.unwrap_or_default(),
            status: inst.status.as_deref().map(parse_status).unwrap_or(ApprovalStatus::Pending),
            initiator_id: inst.open_id.or(inst.user_id).unwrap_or_default(),
            form_data,
            created_at: inst.start_time.as_deref().and_then(ts_opt),
            finished_at: inst.end_time.as_deref().and_then(ts_opt),
        })
    }

    async fn action_approval(&self, req: ApprovalActionRequest) -> ImResult<()> {
        let action_path = match req.action {
            ApprovalAction::Approve => "approve",
            ApprovalAction::Reject => "reject",
        };
        let body = serde_json::json!({
            "approval_code": "",
            "instance_code": req.instance_id,
            "user_id": "",
            "comment": req.comment.unwrap_or_default(),
        });
        let path = format!("/open-apis/approval/v4/instances/{}/{}", req.instance_id, action_path);
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
}
