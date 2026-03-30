use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    TaskService, ImResult, ImError,
    Task, TaskStatus, TaskPriority, Page,
    CreateTaskRequest, UpdateTaskRequest, ListTasksRequest,
};
use crate::client::DingTalkClient;

fn priority_to_dt(p: &TaskPriority) -> u32 {
    match p {
        TaskPriority::Low => 10,
        TaskPriority::Normal => 20,
        TaskPriority::High => 30,
        TaskPriority::Urgent => 40,
    }
}

fn dt_to_priority(v: u32) -> TaskPriority {
    match v {
        10 => TaskPriority::Low,
        30 => TaskPriority::High,
        40 => TaskPriority::Urgent,
        _ => TaskPriority::Normal,
    }
}

#[derive(Deserialize)]
struct DtTask {
    #[serde(rename = "todoTaskId")]
    todo_task_id: Option<String>,
    #[serde(rename = "subject")]
    subject: Option<String>,
    priority: Option<u32>,
    done: Option<bool>,
    #[serde(rename = "dueTime")]
    due_time: Option<String>,
    #[serde(default, rename = "executorIds")]
    executor_ids: Vec<String>,
}

impl From<DtTask> for Task {
    fn from(t: DtTask) -> Self {
        Task {
            id: t.todo_task_id.unwrap_or_default(),
            title: t.subject.unwrap_or_default(),
            description: None,
            status: if t.done.unwrap_or(false) { TaskStatus::Done } else { TaskStatus::Pending },
            priority: dt_to_priority(t.priority.unwrap_or(20)),
            due_time: t.due_time.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            assignees: t.executor_ids,
            creator_id: None,
            created_at: None,
            updated_at: None,
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl TaskService for DingTalkClient {
    async fn create_task(&self, req: CreateTaskRequest) -> ImResult<Task> {
        let body = serde_json::json!({
            "subject": req.title,
            "priority": priority_to_dt(&req.priority),
            "executorIds": req.assignee_ids,
            "dueTime": req.due_time.map(|t| t.to_rfc3339()),
            "description": req.description,
        });
        let resp = self.post("/v1.0/todo/users/me/tasks", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let t: DtTask = serde_json::from_str(&text)?;
        Ok(t.into())
    }

    async fn list_tasks(&self, req: ListTasksRequest) -> ImResult<Page<Task>> {
        let done = match req.status {
            Some(TaskStatus::Done) => "true",
            Some(TaskStatus::Pending) => "false",
            _ => "false",
        };
        let path = format!(
            "/v1.0/todo/users/me/tasks?isDone={}&nextToken={}&maxResults={}",
            done,
            req.cursor.as_deref().unwrap_or(""),
            req.limit.unwrap_or(20),
        );
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let tasks: Vec<DtTask> = serde_json::from_value(
            val.get("todoCards").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        let next = val.get("nextToken").and_then(|v| v.as_str()).map(String::from);
        Ok(Page {
            has_more: next.is_some(),
            items: tasks.into_iter().map(Into::into).collect(),
            next_cursor: next,
        })
    }

    async fn get_task(&self, task_id: &str) -> ImResult<Task> {
        let path = format!("/v1.0/todo/users/me/tasks/{}", task_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let t: DtTask = serde_json::from_str(&text)?;
        Ok(t.into())
    }

    async fn update_task(&self, req: UpdateTaskRequest) -> ImResult<Task> {
        let mut body = serde_json::Map::new();
        if let Some(ref title) = req.title { body.insert("subject".into(), serde_json::json!(title)); }
        if let Some(ref p) = req.priority { body.insert("priority".into(), serde_json::json!(priority_to_dt(p))); }
        if let Some(ref s) = req.status {
            body.insert("done".into(), serde_json::json!(*s == TaskStatus::Done));
        }
        if let Some(ref due) = req.due_time { body.insert("dueTime".into(), serde_json::json!(due.to_rfc3339())); }

        let path = format!("/v1.0/todo/users/me/tasks/{}", req.task_id);
        let resp = self.post(&path, &serde_json::Value::Object(body)).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let t: DtTask = serde_json::from_str(&text)?;
        Ok(t.into())
    }

    async fn delete_task(&self, task_id: &str) -> ImResult<()> {
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!("{}/v1.0/todo/users/me/tasks/{}", self.base_url, task_id);
        let resp = self.http
            .delete(&url)
            .header("x-acs-dingtalk-access-token", &token)
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }
}
