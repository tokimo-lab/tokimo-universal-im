use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    TaskService, ImResult, ImError,
    Task, TaskStatus, TaskPriority, Page,
    CreateTaskRequest, UpdateTaskRequest, ListTasksRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct TaskData {
    task: Option<LarkTask>,
}

#[derive(Deserialize)]
struct ListTaskData {
    #[serde(default)]
    items: Vec<LarkTask>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct LarkTask {
    id: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    completed_at: Option<String>,
    due: Option<DueInfo>,
    #[serde(default)]
    members: Vec<MemberInfo>,
    creator: Option<CreatorInfo>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Deserialize)]
struct DueInfo {
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct MemberInfo {
    id: Option<String>,
}

#[derive(Deserialize)]
struct CreatorInfo {
    id: Option<String>,
}

fn ts_opt(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let ts: i64 = s.parse().ok()?;
    chrono::DateTime::from_timestamp(ts, 0)
}

impl From<LarkTask> for Task {
    fn from(t: LarkTask) -> Self {
        Task {
            id: t.id.unwrap_or_default(),
            title: t.summary.unwrap_or_default(),
            description: t.description,
            status: if t.completed_at.is_some() { TaskStatus::Done } else { TaskStatus::Pending },
            priority: TaskPriority::Normal, // Lark tasks don't have priority via REST
            due_time: t.due.and_then(|d| d.timestamp).and_then(|s| ts_opt(&s)),
            assignees: t.members.into_iter().filter_map(|m| m.id).collect(),
            creator_id: t.creator.and_then(|c| c.id),
            created_at: t.created_at.and_then(|s| ts_opt(&s)),
            updated_at: t.updated_at.and_then(|s| ts_opt(&s)),
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl TaskService for LarkClient {
    async fn create_task(&self, req: CreateTaskRequest) -> ImResult<Task> {
        let body = serde_json::json!({
            "summary": req.title,
            "description": req.description,
            "due": req.due_time.map(|t| serde_json::json!({"timestamp": t.timestamp().to_string()})),
            "members": req.assignee_ids.iter().map(|id| serde_json::json!({"id": id, "role": "assignee"})).collect::<Vec<_>>(),
        });
        let resp = self.post("/open-apis/task/v2/tasks", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<TaskData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let task = data.data.and_then(|d| d.task).ok_or_else(|| ImError::Internal("no task".into()))?;
        Ok(task.into())
    }

    async fn list_tasks(&self, req: ListTasksRequest) -> ImResult<Page<Task>> {
        let mut path = "/open-apis/task/v2/tasks".to_string();
        let mut params = vec![];
        if let Some(ref cursor) = req.cursor {
            params.push(format!("page_token={}", cursor));
        }
        if let Some(limit) = req.limit {
            params.push(format!("page_size={}", limit));
        }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ListTaskData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ListTaskData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_task(&self, task_id: &str) -> ImResult<Task> {
        let path = format!("/open-apis/task/v2/tasks/{}", task_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<TaskData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let task = data.data.and_then(|d| d.task).ok_or_else(|| ImError::NotFound { resource: task_id.into() })?;
        Ok(task.into())
    }

    async fn update_task(&self, req: UpdateTaskRequest) -> ImResult<Task> {
        let mut body = serde_json::Map::new();
        if let Some(ref t) = req.title { body.insert("summary".into(), serde_json::json!(t)); }
        if let Some(ref d) = req.description { body.insert("description".into(), serde_json::json!(d)); }
        if let Some(ref due) = req.due_time { body.insert("due".into(), serde_json::json!({"timestamp": due.timestamp().to_string()})); }
        if let Some(ref s) = req.status {
            if *s == TaskStatus::Done {
                body.insert("completed_at".into(), serde_json::json!(chrono::Utc::now().timestamp().to_string()));
            }
        }

        let path = format!("/open-apis/task/v2/tasks/{}", req.task_id);
        let resp = self.put(&path, &serde_json::Value::Object(body)).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<TaskData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let task = data.data.and_then(|d| d.task).ok_or_else(|| ImError::Internal("no task".into()))?;
        Ok(task.into())
    }

    async fn delete_task(&self, task_id: &str) -> ImResult<()> {
        let path = format!("/open-apis/task/v2/tasks/{}", task_id);
        let resp = self.delete(&path).await?;
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
