use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    ApprovalInstance, Page, CreateApprovalRequest,
    ListApprovalRequest, ApprovalActionRequest,
};

/// Approval / OA workflow management.
#[async_trait]
pub trait ApprovalService: Send + Sync {
    /// Create a new approval instance.
    async fn create_approval(&self, req: CreateApprovalRequest) -> ImResult<ApprovalInstance>;

    /// List approval instances.
    async fn list_approvals(&self, req: ListApprovalRequest) -> ImResult<Page<ApprovalInstance>>;

    /// Get a single approval instance.
    async fn get_approval(&self, instance_id: &str) -> ImResult<ApprovalInstance>;

    /// Take action on an approval (approve / reject).
    async fn action_approval(&self, req: ApprovalActionRequest) -> ImResult<()>;
}
