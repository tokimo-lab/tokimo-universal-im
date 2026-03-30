use serde::{Deserialize, Serialize};

/// A report / daily / weekly submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Report ID.
    pub id: String,
    /// Template name / type.
    pub template_name: Option<String>,
    /// Creator user ID.
    pub creator_id: String,
    /// Creator name.
    pub creator_name: Option<String>,
    /// Report content (key-value form fields).
    pub content: serde_json::Value,
    /// Creation time.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Modified time.
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Report template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    /// Template fields schema.
    pub fields: serde_json::Value,
}

/// Request to list reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListReportsRequest {
    /// Template name to filter by.
    pub template_name: Option<String>,
    /// Creator user ID filter.
    pub creator_id: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Request to create a report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportRequest {
    /// Template ID.
    pub template_id: String,
    /// Report content as form data.
    pub content: serde_json::Value,
    /// Recipient user IDs.
    #[serde(default)]
    pub to_user_ids: Vec<String>,
}

/// Report statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStatistics {
    /// Template name.
    pub template_name: String,
    /// Total submitted.
    pub total_submitted: u32,
    /// Total not submitted.
    pub total_not_submitted: u32,
    /// Submitted user IDs.
    #[serde(default)]
    pub submitted_users: Vec<String>,
}
