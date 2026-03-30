use serde::{Deserialize, Serialize};

/// An attendance record (check-in / check-out).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceRecord {
    /// Record ID.
    pub id: String,
    /// User ID.
    pub user_id: String,
    /// User name.
    pub user_name: Option<String>,
    /// Check type.
    pub check_type: CheckType,
    /// Check time.
    pub check_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Check result.
    pub result: AttendanceResult,
    /// Location description.
    pub location: Option<String>,
}

/// Type of attendance check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckType {
    CheckIn,
    CheckOut,
}

/// Result of an attendance check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttendanceResult {
    Normal,
    Late,
    EarlyLeave,
    Absent,
    NotSigned,
}

/// Request to query attendance records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAttendanceRequest {
    /// User IDs to query.
    #[serde(default)]
    pub user_ids: Vec<String>,
    /// Start date.
    pub start_date: chrono::NaiveDate,
    /// End date.
    pub end_date: chrono::NaiveDate,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// An attendance shift / schedule rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceShift {
    pub id: String,
    pub name: String,
    /// Shift times as JSON.
    pub schedule: serde_json::Value,
}

/// Attendance summary for a user over a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceSummary {
    pub user_id: String,
    pub work_days: u32,
    pub late_count: u32,
    pub early_leave_count: u32,
    pub absent_count: u32,
    pub overtime_hours: f64,
}
