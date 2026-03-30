use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    ReportService, ImResult, ImError,
    Report, ReportTemplate, ReportStatistics, Page,
    ListReportsRequest, CreateReportRequest,
};
use crate::client::DingTalkClient;

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtTemplate {
    id: Option<String>,
    name: Option<String>,
    fields: Option<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtTemplateListResponse {
    #[serde(default)]
    templates: Vec<DtTemplate>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtReport {
    #[serde(rename = "reportId")]
    report_id: Option<String>,
    #[serde(rename = "templateName")]
    template_name: Option<String>,
    #[serde(rename = "creatorId")]
    creator_id: Option<String>,
    #[serde(rename = "creatorName")]
    creator_name: Option<String>,
    content: Option<serde_json::Value>,
    #[serde(rename = "createTime")]
    create_time: Option<String>,
    #[serde(rename = "modifiedTime")]
    modified_time: Option<String>,
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

impl From<DtReport> for Report {
    fn from(r: DtReport) -> Self {
        Report {
            id: r.report_id.unwrap_or_default(),
            template_name: r.template_name,
            creator_id: r.creator_id.unwrap_or_default(),
            creator_name: r.creator_name,
            content: r.content.unwrap_or(serde_json::Value::Null),
            created_at: r.create_time.as_deref().and_then(parse_dt_time),
            modified_at: r.modified_time.as_deref().and_then(parse_dt_time),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtReportListResponse {
    #[serde(default)]
    list: Vec<DtReport>,
    #[serde(rename = "hasMore")]
    has_more: Option<bool>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtStatistics {
    #[serde(rename = "templateName")]
    template_name: Option<String>,
    #[serde(rename = "totalSubmitted")]
    total_submitted: Option<u32>,
    #[serde(rename = "totalNotSubmitted")]
    total_not_submitted: Option<u32>,
    #[serde(default, rename = "submittedUsers")]
    submitted_users: Vec<String>,
}

#[async_trait]
impl ReportService for DingTalkClient {
    async fn list_templates(&self) -> ImResult<Vec<ReportTemplate>> {
        let resp = self.get("/v1.0/report/templates").await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtTemplateListResponse = serde_json::from_str(&text)?;
        Ok(data.templates.into_iter().map(|t| ReportTemplate {
            id: t.id.unwrap_or_default(),
            name: t.name.unwrap_or_default(),
            fields: t.fields.unwrap_or(serde_json::Value::Null),
        }).collect())
    }

    async fn get_template(&self, template_name: &str) -> ImResult<ReportTemplate> {
        let path = format!("/v1.0/report/templates/{}", template_name);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let t: DtTemplate = serde_json::from_str(&text)?;
        Ok(ReportTemplate {
            id: t.id.unwrap_or_default(),
            name: t.name.unwrap_or_default(),
            fields: t.fields.unwrap_or(serde_json::Value::Null),
        })
    }

    async fn create_report(&self, req: CreateReportRequest) -> ImResult<Report> {
        let body = serde_json::json!({
            "templateId": req.template_id,
            "content": req.content,
            "toUserIds": req.to_user_ids,
        });
        let resp = self.post("/v1.0/report/reports", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let r: DtReport = serde_json::from_str(&text)?;
        Ok(r.into())
    }

    async fn list_reports(&self, req: ListReportsRequest) -> ImResult<Page<Report>> {
        let mut body = serde_json::Map::new();
        if let Some(ref name) = req.template_name {
            body.insert("templateName".into(), serde_json::json!(name));
        }
        if let Some(ref cid) = req.creator_id {
            body.insert("creatorId".into(), serde_json::json!(cid));
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
            "/v1.0/report/reports/query",
            &serde_json::Value::Object(body),
        ).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtReportListResponse = serde_json::from_str(&text)?;
        Ok(Page {
            items: data.list.into_iter().map(Into::into).collect(),
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next_cursor,
        })
    }

    async fn get_report(&self, report_id: &str) -> ImResult<Report> {
        let path = format!("/v1.0/report/reports/{}", report_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let r: DtReport = serde_json::from_str(&text)?;
        Ok(r.into())
    }

    async fn get_statistics(&self, report_id: &str) -> ImResult<ReportStatistics> {
        let path = format!("/v1.0/report/reports/{}/statistics", report_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let s: DtStatistics = serde_json::from_str(&text)?;
        Ok(ReportStatistics {
            template_name: s.template_name.unwrap_or_default(),
            total_submitted: s.total_submitted.unwrap_or(0),
            total_not_submitted: s.total_not_submitted.unwrap_or(0),
            submitted_users: s.submitted_users,
        })
    }
}
