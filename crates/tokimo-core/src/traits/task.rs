use crate::error::ImResult;
use crate::types::{CreateTaskRequest, ListTasksRequest, Page, Task, UpdateTaskRequest};
use async_trait::async_trait;

/// Task / to-do operations.
#[async_trait]
pub trait TaskService: Send + Sync {
    /// Create a task.
    async fn create_task(&self, req: CreateTaskRequest) -> ImResult<Task>;

    /// List tasks with optional filters.
    async fn list_tasks(&self, req: ListTasksRequest) -> ImResult<Page<Task>>;

    /// Get a single task by ID.
    async fn get_task(&self, task_id: &str) -> ImResult<Task>;

    /// Update an existing task (title, priority, status, etc.).
    async fn update_task(&self, req: UpdateTaskRequest) -> ImResult<Task>;

    /// Delete a task.
    async fn delete_task(&self, task_id: &str) -> ImResult<()>;
}
