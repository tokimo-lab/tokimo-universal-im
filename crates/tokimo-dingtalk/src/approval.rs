use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    ApprovalService, ImResult, ImError,
    ApprovalInstance, ApprovalStatus, ApprovalAction, Page,
    CreateApprovalRequest, ListApprovalRequest, ApprovalActionRequest,
};
use crate::client::DingTalkClient;

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtApprovalInstance {
    #[serde(rename = "instanceId")]
    instance_id: Option<String>,
    #[serde(rename = "processCode")]
    process_code: Option<String>,
    title: Option<String>,
    status: Option<String>,
    #[serde(rename = "originatorUserId")]
    originator_user_id: Option<String>,
    #[serde(rename = "formComponentValues")]
    form_component_values: Option<serde_json::Value>,
    #[serde(rename = "createTime")]
    create_time: Option<String>,
    #[serde(rename = "finishTime")]
    finish_time: Option<String>,
}

fn parse_status(s: &str) -> ApprovalStatus {
    match s {
        "COMPLETED" | "APPROVED" => ApprovalStatus::Approved,
        "REFUSED" | "REJECTED" => ApprovalStatus::Rejected,
        "TERMINATED" | "CANCELLED" => ApprovalStatus::Cancelled,
        "DELETED" => ApprovalStatus::Deleted,
        _ => ApprovalStatus::Pending,
    }
}

fn parse_dt_time(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|| {
            s.parse::<i64>().ok().and_then(|ms| {
                chrono::DateTime::from_timestamp_millis(ms)
            })
        })
}

impl From<DtApprovalInstance> for ApprovalInstance {
    fn from(a: DtApprovalInstance) -> Self {
        ApprovalInstance {
            id: a.instance_id.unwrap_or_default(),
            process_code: a.process_code.unwrap_or_default(),
            title: a.title.unwrap_or_default(),
            status: a.status.as_deref().map(parse_status).unwrap_or(ApprovalStatus::Pending),
            initiator_id: a.originator_user_id.unwrap_or_default(),
            form_data: a.form_component_values.unwrap_or(serde_json::Value::Null),
            created_at: a.create_time.as_deref().and_then(parse_dt_time),
            finished_at: a.finish_time.as_deref().and_then(parse_dt_time),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtApprovalListResponse {
    #[serde(default)]
    list: Vec<DtApprovalInstance>,
    #[serde(rename = "hasMore")]
    has_more: Option<bool>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtCreateApprovalResponse {
    #[serde(rename = "instanceId")]
    instance_id: Option<String>,
}

#[async_trait]
impl ApprovalService for DingTalkClient {
    async fn create_approval(&self, req: CreateApprovalRequest) -> ImResult<ApprovalInstance> {
        let body = serde_json::json!({
            "processCode": req.process_code,
            "originatorUserId": req.initiator_id,
            "formComponentValues": req.form_data,
            "approvers": req.approvers,
            "ccList": req.cc_users,
        });
        let resp = self.post("/v1.0/workflow/processInstances", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let created: DtCreateApprovalResponse = serde_json::from_str(&text)?;
        let instance_id = created.instance_id.unwrap_or_default();

        // Fetch the full instance details
        self.get_approval(&instance_id).await
    }

    async fn list_approvals(&self, req: ListApprovalRequest) -> ImResult<Page<ApprovalInstance>> {
        let mut body = serde_json::Map::new();
        if let Some(ref code) = req.process_code {
            body.insert("processCode".into(), serde_json::json!(code));
        }
        if let Some(ref st) = req.start_time {
            body.insert("startTime".into(), serde_json::json!(st.timestamp_millis()));
        }
        if let Some(ref et) = req.end_time {
            body.insert("endTime".into(), serde_json::json!(et.timestamp_millis()));
        }
        if let Some(ref cursor) = req.cursor {
            body.insert("nextCursor".into(), serde_json::json!(cursor));
        }
        if let Some(limit) = req.limit {
            body.insert("maxResults".into(), serde_json::json!(limit));
        }

        let resp = self.post(
            "/v1.0/workflow/processInstances/query",
            &serde_json::Value::Object(body),
        ).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtApprovalListResponse = serde_json::from_str(&text)?;
        Ok(Page {
            items: data.list.into_iter().map(Into::into).collect(),
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next_cursor,
        })
    }

    async fn get_approval(&self, instance_id: &str) -> ImResult<ApprovalInstance> {
        let path = format!("/v1.0/workflow/processInstances/{}", instance_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let a: DtApprovalInstance = serde_json::from_str(&text)?;
        Ok(a.into())
    }

    async fn action_approval(&self, req: ApprovalActionRequest) -> ImResult<()> {
        let action_path = match req.action {
            ApprovalAction::Approve => "approve",
            ApprovalAction::Reject => "reject",
        };
        let path = format!(
            "/v1.0/workflow/processInstances/{}/{}",
            req.instance_id, action_path
        );
        let mut body = serde_json::Map::new();
        if let Some(ref comment) = req.comment {
            body.insert("comment".into(), serde_json::json!(comment));
        }

        let resp = self.post(&path, &serde_json::Value::Object(body)).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        Ok(())
    }
}
