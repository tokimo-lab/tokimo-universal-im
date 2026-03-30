use serde::{Deserialize, Serialize};

/// A task / to-do item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Platform-specific task ID.
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_time: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub assignees: Vec<String>,
    pub creator_id: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// Task completion status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Deleted,
}

/// Task priority level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Request to create a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub due_time: Option<chrono::DateTime<chrono::Utc>>,
    pub assignee_ids: Vec<String>,
}

/// Request to update a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub due_time: Option<chrono::DateTime<chrono::Utc>>,
    pub status: Option<TaskStatus>,
}

/// Request to list tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksRequest {
    pub status: Option<TaskStatus>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
