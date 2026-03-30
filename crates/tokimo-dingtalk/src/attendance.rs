use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    AttendanceService, ImResult, ImError,
    AttendanceRecord, AttendanceShift, AttendanceSummary, CheckType, AttendanceResult, Page,
    ListAttendanceRequest,
};
use crate::client::DingTalkClient;

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtAttendanceRecord {
    id: Option<String>,
    #[serde(rename = "userId")]
    user_id: Option<String>,
    #[serde(rename = "userName")]
    user_name: Option<String>,
    #[serde(rename = "checkType")]
    check_type: Option<String>,
    #[serde(rename = "userCheckTime")]
    user_check_time: Option<String>,
    #[serde(rename = "timeResult")]
    time_result: Option<String>,
    #[serde(rename = "locationResult")]
    location_result: Option<String>,
}

fn parse_check_type(s: &str) -> CheckType {
    match s {
        "OnDuty" | "on_duty" => CheckType::CheckIn,
        _ => CheckType::CheckOut,
    }
}

fn parse_attendance_result(s: &str) -> AttendanceResult {
    match s {
        "Normal" | "normal" => AttendanceResult::Normal,
        "Late" | "late" => AttendanceResult::Late,
        "Early" | "early" => AttendanceResult::EarlyLeave,
        "Absent" | "absent" | "SeriousLate" => AttendanceResult::Absent,
        "NotSigned" | "not_signed" => AttendanceResult::NotSigned,
        _ => AttendanceResult::Normal,
    }
}

fn parse_dt_time(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|| {
            s.parse::<i64>().ok().and_then(|ms| {
                chrono::DateTime::from_timestamp_millis(ms)
            })
        })
}

impl From<DtAttendanceRecord> for AttendanceRecord {
    fn from(r: DtAttendanceRecord) -> Self {
        AttendanceRecord {
            id: r.id.unwrap_or_default(),
            user_id: r.user_id.unwrap_or_default(),
            user_name: r.user_name,
            check_type: r.check_type.as_deref().map(parse_check_type).unwrap_or(CheckType::CheckIn),
            check_time: r.user_check_time.as_deref().and_then(parse_dt_time),
            result: r.time_result.as_deref().map(parse_attendance_result).unwrap_or(AttendanceResult::Normal),
            location: r.location_result,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtAttendanceListResponse {
    #[serde(default)]
    list: Vec<DtAttendanceRecord>,
    #[serde(rename = "hasMore")]
    has_more: Option<bool>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtShift {
    id: Option<String>,
    name: Option<String>,
    #[serde(default)]
    sections: serde_json::Value,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtShiftListResponse {
    #[serde(default)]
    list: Vec<DtShift>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtSummary {
    #[serde(rename = "userId")]
    user_id: Option<String>,
    #[serde(rename = "workDays")]
    work_days: Option<u32>,
    #[serde(rename = "lateCount")]
    late_count: Option<u32>,
    #[serde(rename = "earlyLeaveCount")]
    early_leave_count: Option<u32>,
    #[serde(rename = "absentCount")]
    absent_count: Option<u32>,
    #[serde(rename = "overtimeHours")]
    overtime_hours: Option<f64>,
}

#[async_trait]
impl AttendanceService for DingTalkClient {
    async fn list_records(&self, req: ListAttendanceRequest) -> ImResult<Page<AttendanceRecord>> {
        let body = serde_json::json!({
            "userIds": req.user_ids,
            "startDate": req.start_date.format("%Y-%m-%d").to_string(),
            "endDate": req.end_date.format("%Y-%m-%d").to_string(),
            "cursor": req.cursor.as_deref().unwrap_or("0"),
            "size": req.limit.unwrap_or(50)
        });
        let resp = self.post("/v1.0/attendance/records/query", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtAttendanceListResponse = serde_json::from_str(&text)?;
        Ok(Page {
            items: data.list.into_iter().map(Into::into).collect(),
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next_cursor,
        })
    }

    async fn list_shifts(
        &self,
        user_ids: &[String],
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> ImResult<Vec<AttendanceShift>> {
        let body = serde_json::json!({
            "userIds": user_ids,
            "startDate": start_date.format("%Y-%m-%d").to_string(),
            "endDate": end_date.format("%Y-%m-%d").to_string()
        });
        let resp = self.post("/v1.0/attendance/shifts/query", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtShiftListResponse = serde_json::from_str(&text)?;
        Ok(data.list.into_iter().map(|s| AttendanceShift {
            id: s.id.unwrap_or_default(),
            name: s.name.unwrap_or_default(),
            schedule: s.sections,
        }).collect())
    }

    async fn get_summary(
        &self,
        user_id: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> ImResult<AttendanceSummary> {
        let body = serde_json::json!({
            "userId": user_id,
            "startDate": start_date.format("%Y-%m-%d").to_string(),
            "endDate": end_date.format("%Y-%m-%d").to_string()
        });
        let resp = self.post("/v1.0/attendance/records/summary", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let s: DtSummary = serde_json::from_str(&text)?;
        Ok(AttendanceSummary {
            user_id: s.user_id.unwrap_or_else(|| user_id.to_string()),
            work_days: s.work_days.unwrap_or(0),
            late_count: s.late_count.unwrap_or(0),
            early_leave_count: s.early_leave_count.unwrap_or(0),
            absent_count: s.absent_count.unwrap_or(0),
            overtime_hours: s.overtime_hours.unwrap_or(0.0),
        })
    }
}
