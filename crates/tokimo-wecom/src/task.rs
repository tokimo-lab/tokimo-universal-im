use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    TaskService, ImResult, ImError,
    Task, TaskStatus, TaskPriority, Page,
    CreateTaskRequest, UpdateTaskRequest, ListTasksRequest,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
#[allow(dead_code)]
struct WcTodoIndex {
    todo_id: Option<String>,
    todo_status: Option<i32>,
    user_status: Option<i32>,
    creator_id: Option<String>,
    remind_time: Option<String>,
    create_time: Option<String>,
    update_time: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct WcTodoDetail {
    todo_id: Option<String>,
    content: Option<String>,
    todo_status: Option<i32>,
    user_status: Option<i32>,
    creator_id: Option<String>,
    remind_time: Option<String>,
    create_time: Option<String>,
    update_time: Option<String>,
}

fn parse_wc_dt(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
}

impl From<WcTodoDetail> for Task {
    fn from(t: WcTodoDetail) -> Self {
        Task {
            id: t.todo_id.unwrap_or_default(),
            title: t.content.unwrap_or_default(),
            description: None,
            status: match t.todo_status.unwrap_or(1) {
                0 => TaskStatus::Done,
                2 => TaskStatus::Deleted,
                _ => TaskStatus::Pending,
            },
            priority: TaskPriority::Normal,
            due_time: t.remind_time.as_deref().and_then(parse_wc_dt),
            assignees: vec![],
            creator_id: t.creator_id,
            created_at: t.create_time.as_deref().and_then(parse_wc_dt),
            updated_at: t.update_time.as_deref().and_then(parse_wc_dt),
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl TaskService for WeComClient {
    async fn create_task(&self, req: CreateTaskRequest) -> ImResult<Task> {
        let body = serde_json::json!({
            "content": req.title,
            "remind_time": req.due_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        });
        let resp = self.post("/cgi-bin/oa/todo/create", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let todo_id = val.get("todo_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Ok(Task {
            id: todo_id,
            title: req.title,
            description: req.description,
            status: TaskStatus::Pending,
            priority: req.priority,
            due_time: req.due_time,
            assignees: req.assignee_ids,
            creator_id: None,
            created_at: Some(chrono::Utc::now()),
            updated_at: None,
            extra: serde_json::Value::Null,
        })
    }

    async fn list_tasks(&self, req: ListTasksRequest) -> ImResult<Page<Task>> {
        let body = serde_json::json!({
            "limit": req.limit.unwrap_or(20),
            "cursor": req.cursor.unwrap_or_default(),
        });
        let resp = self.post("/cgi-bin/oa/todo/get_list", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let indices: Vec<WcTodoIndex> = serde_json::from_value(
            val.get("index_list").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        let has_more = val.get("has_more").and_then(|v| v.as_bool()).unwrap_or(false);
        let next_cursor = val.get("next_cursor").and_then(|v| v.as_str()).map(String::from);

        // Fetch details
        let ids: Vec<String> = indices.iter().filter_map(|i| i.todo_id.clone()).collect();
        if ids.is_empty() {
            return Ok(Page { items: vec![], has_more: false, next_cursor: None });
        }
        let detail_body = serde_json::json!({ "todo_id_list": ids });
        let detail_resp = self.post("/cgi-bin/oa/todo/get_detail", &detail_body).await?;
        let detail_text = detail_resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let detail_val: serde_json::Value = serde_json::from_str(&detail_text)?;
        let details: Vec<WcTodoDetail> = serde_json::from_value(
            detail_val.get("data_list").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        Ok(Page {
            items: details.into_iter().map(Into::into).collect(),
            has_more,
            next_cursor,
        })
    }

    async fn get_task(&self, task_id: &str) -> ImResult<Task> {
        let body = serde_json::json!({ "todo_id_list": [task_id] });
        let resp = self.post("/cgi-bin/oa/todo/get_detail", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let details: Vec<WcTodoDetail> = serde_json::from_value(
            val.get("data_list").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        details.into_iter().next().map(Into::into).ok_or_else(|| ImError::NotFound {
            resource: format!("todo {}", task_id),
        })
    }

    async fn update_task(&self, req: UpdateTaskRequest) -> ImResult<Task> {
        let mut body = serde_json::Map::new();
        body.insert("todo_id".into(), serde_json::json!(req.task_id));
        if let Some(ref title) = req.title { body.insert("content".into(), serde_json::json!(title)); }
        if let Some(ref s) = req.status {
            let v = match s {
                TaskStatus::Done => 0,
                TaskStatus::Deleted => 2,
                _ => 1,
            };
            body.insert("todo_status".into(), serde_json::json!(v));
        }
        if let Some(ref due) = req.due_time {
            body.insert("remind_time".into(), serde_json::json!(due.format("%Y-%m-%d %H:%M:%S").to_string()));
        }

        let resp = self.post("/cgi-bin/oa/todo/update", &serde_json::Value::Object(body)).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        self.get_task(&req.task_id).await
    }

    async fn delete_task(&self, task_id: &str) -> ImResult<()> {
        let body = serde_json::json!({ "todo_id": task_id });
        let resp = self.post("/cgi-bin/oa/todo/delete", &body).await?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }
}
