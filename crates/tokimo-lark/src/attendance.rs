use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    AttendanceService, ImResult, ImError,
    AttendanceRecord, AttendanceShift, AttendanceSummary, Page,
    CheckType, AttendanceResult, ListAttendanceRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct StatsData {
    #[serde(default)]
    user_datas: Vec<UserStatData>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct UserStatData {
    user_id: Option<String>,
    name: Option<String>,
    #[serde(default)]
    datas: Vec<StatItem>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct StatItem {
    code: Option<String>,
    value: Option<String>,
    title: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ShiftListData {
    #[serde(default)]
    shift_list: Vec<LarkShift>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkShift {
    shift_id: Option<String>,
    shift_name: Option<String>,
    #[serde(default)]
    punch_time_rules: serde_json::Value,
}

#[derive(Deserialize)]
struct SummaryData {
    #[serde(default)]
    user_datas: Vec<UserStatData>,
}

#[async_trait]
impl AttendanceService for LarkClient {
    async fn list_records(&self, req: ListAttendanceRequest) -> ImResult<Page<AttendanceRecord>> {
        let body = serde_json::json!({
            "user_ids": req.user_ids,
            "start_date": req.start_date.format("%Y-%m-%d").to_string(),
            "end_date": req.end_date.format("%Y-%m-%d").to_string(),
            "page_token": req.cursor.as_deref().unwrap_or(""),
            "page_size": req.limit.unwrap_or(50),
        });
        let resp = self.post("/open-apis/attendance/v1/user_stats_datas/query", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<StatsData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let stats = data.data.unwrap_or(StatsData {
            user_datas: vec![], page_token: None, has_more: None,
        });
        let mut records = Vec::new();
        for user_data in &stats.user_datas {
            let user_id = user_data.user_id.clone().unwrap_or_default();
            let user_name = user_data.name.clone();
            for (idx, item) in user_data.datas.iter().enumerate() {
                records.push(AttendanceRecord {
                    id: format!("{}_{}", user_id, idx),
                    user_id: user_id.clone(),
                    user_name: user_name.clone(),
                    check_type: CheckType::CheckIn,
                    check_time: None,
                    result: AttendanceResult::Normal,
                    location: item.value.clone(),
                });
            }
        }
        Ok(Page {
            items: records,
            has_more: stats.has_more.unwrap_or(false),
            next_cursor: stats.page_token,
        })
    }

    async fn list_shifts(
        &self,
        _user_ids: &[String],
        _start_date: chrono::NaiveDate,
        _end_date: chrono::NaiveDate,
    ) -> ImResult<Vec<AttendanceShift>> {
        let resp = self.get("/open-apis/attendance/v1/shifts?page_size=50").await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ShiftListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ShiftListData {
            shift_list: vec![], page_token: None, has_more: None,
        });
        Ok(list.shift_list.into_iter().map(|s| AttendanceShift {
            id: s.shift_id.unwrap_or_default(),
            name: s.shift_name.unwrap_or_default(),
            schedule: s.punch_time_rules,
        }).collect())
    }

    async fn get_summary(
        &self,
        user_id: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> ImResult<AttendanceSummary> {
        let body = serde_json::json!({
            "user_ids": [user_id],
            "start_date": start_date.format("%Y-%m-%d").to_string(),
            "end_date": end_date.format("%Y-%m-%d").to_string(),
        });
        let resp = self.post("/open-apis/attendance/v1/user_stats_views/query", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SummaryData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        // Parse summary from stats view data
        let summary_data = data.data.unwrap_or(SummaryData { user_datas: vec![] });
        let user_data = summary_data.user_datas.into_iter().next();

        let mut work_days: u32 = 0;
        let mut late_count: u32 = 0;
        let mut early_leave_count: u32 = 0;
        let mut absent_count: u32 = 0;
        let mut overtime_hours: f64 = 0.0;

        if let Some(ud) = &user_data {
            for item in &ud.datas {
                let code = item.code.as_deref().unwrap_or("");
                let val = item.value.as_deref().unwrap_or("0");
                match code {
                    "work_days" => work_days = val.parse().unwrap_or(0),
                    "late_count" => late_count = val.parse().unwrap_or(0),
                    "early_leave_count" => early_leave_count = val.parse().unwrap_or(0),
                    "absent_count" => absent_count = val.parse().unwrap_or(0),
                    "overtime_hours" => overtime_hours = val.parse().unwrap_or(0.0),
                    _ => {}
                }
            }
        }

        Ok(AttendanceSummary {
            user_id: user_id.to_string(),
            work_days,
            late_count,
            early_leave_count,
            absent_count,
            overtime_hours,
        })
    }
}
