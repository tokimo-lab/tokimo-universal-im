use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    AttendanceRecord, AttendanceShift, AttendanceSummary, Page,
    ListAttendanceRequest,
};

/// Attendance / check-in management.
#[async_trait]
pub trait AttendanceService: Send + Sync {
    /// Get attendance records for users in a date range.
    async fn list_records(&self, req: ListAttendanceRequest) -> ImResult<Page<AttendanceRecord>>;

    /// Get shift schedules for users.
    async fn list_shifts(&self, user_ids: &[String], start_date: chrono::NaiveDate, end_date: chrono::NaiveDate) -> ImResult<Vec<AttendanceShift>>;

    /// Get attendance summary for a user.
    async fn get_summary(&self, user_id: &str, start_date: chrono::NaiveDate, end_date: chrono::NaiveDate) -> ImResult<AttendanceSummary>;
}
