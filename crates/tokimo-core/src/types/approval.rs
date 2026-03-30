use serde::{Deserialize, Serialize};

/// An approval workflow instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalInstance {
    /// Instance ID.
    pub id: String,
    /// Process/template code.
    pub process_code: String,
    /// Title of the approval.
    pub title: String,
    /// Current status.
    pub status: ApprovalStatus,
    /// Initiator user ID.
    pub initiator_id: String,
    /// Form data as key-value pairs.
    #[serde(default)]
    pub form_data: serde_json::Value,
    /// Creation time.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completion time.
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Approval status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
    Deleted,
}

/// Request to create an approval instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApprovalRequest {
    /// Process/template code.
    pub process_code: String,
    /// Initiator user ID.
    pub initiator_id: String,
    /// Form values.
    pub form_data: serde_json::Value,
    /// Approver user IDs.
    #[serde(default)]
    pub approvers: Vec<String>,
    /// CC user IDs.
    #[serde(default)]
    pub cc_users: Vec<String>,
}

/// Request to list approval instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListApprovalRequest {
    /// Process/template code to filter by.
    pub process_code: Option<String>,
    /// Status filter.
    pub status: Option<ApprovalStatus>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Approval action request (approve / reject).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalActionRequest {
    /// Instance ID.
    pub instance_id: String,
    /// Action type.
    pub action: ApprovalAction,
    /// Comment / reason.
    pub comment: Option<String>,
}

/// The action to take on an approval.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalAction {
    Approve,
    Reject,
}
