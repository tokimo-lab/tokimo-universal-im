use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    CalendarService, ImResult, ImError,
    CalendarEvent, EventAttendee, AttendeeStatus, Page, BusySlot,
    CreateEventRequest, UpdateEventRequest, ListEventsRequest, FreeBusyRequest,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct ScheduleIdListResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    schedule_id_list: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ScheduleDetailResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    schedule: Vec<WcSchedule>,
}

#[derive(Deserialize)]
struct WcSchedule {
    schedule_id: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    location: Option<String>,
    is_whole_day: Option<i32>,
    #[serde(default)]
    attendees: Vec<WcAttendee>,
}

#[derive(Deserialize)]
struct WcAttendee {
    userid: Option<String>,
    response_status: Option<i32>,
}

fn ts_to_dt(ts: i64) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(|| chrono::Utc::now())
}

impl From<WcSchedule> for CalendarEvent {
    fn from(s: WcSchedule) -> Self {
        CalendarEvent {
            id: s.schedule_id.unwrap_or_default(),
            title: s.summary.unwrap_or_default(),
            description: s.description,
            start_time: s.start_time.map(ts_to_dt).unwrap_or_else(chrono::Utc::now),
            end_time: s.end_time.map(ts_to_dt).unwrap_or_else(chrono::Utc::now),
            location: s.location,
            is_all_day: s.is_whole_day.unwrap_or(0) == 1,
            attendees: s.attendees.into_iter().map(|a| EventAttendee {
                user_id: a.userid.unwrap_or_default(),
                name: None,
                status: match a.response_status.unwrap_or(0) {
                    1 => AttendeeStatus::Accepted,
                    2 => AttendeeStatus::Declined,
                    3 => AttendeeStatus::Tentative,
                    _ => AttendeeStatus::Unknown,
                },
            }).collect(),
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl CalendarService for WeComClient {
    async fn create_event(&self, req: CreateEventRequest) -> ImResult<CalendarEvent> {
        let body = serde_json::json!({
            "schedule": {
                "summary": req.title,
                "description": req.description,
                "start_time": req.start_time.timestamp().to_string(),
                "end_time": req.end_time.timestamp().to_string(),
                "location": req.location,
                "is_whole_day": if req.is_all_day { 1 } else { 0 },
                "attendees": req.attendee_ids.iter().map(|id| serde_json::json!({"userid": id})).collect::<Vec<_>>(),
            }
        });
        let resp = self.post("/cgi-bin/oa/schedule/add", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let schedule_id = val.get("schedule_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        Ok(CalendarEvent {
            id: schedule_id,
            title: req.title,
            description: req.description,
            start_time: req.start_time,
            end_time: req.end_time,
            location: req.location,
            is_all_day: req.is_all_day,
            attendees: vec![],
            extra: serde_json::Value::Null,
        })
    }

    async fn list_events(&self, req: ListEventsRequest) -> ImResult<Page<CalendarEvent>> {
        let body = serde_json::json!({
            "start_time": req.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            "end_time": req.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        let resp = self.post("/cgi-bin/oa/schedule/get_by_range", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: ScheduleIdListResp = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.errcode.unwrap_or(-1),
                message: data.errmsg.unwrap_or(text),
            });
        }
        // Need to fetch details for each schedule
        if data.schedule_id_list.is_empty() {
            return Ok(Page { items: vec![], has_more: false, next_cursor: None });
        }
        let detail_body = serde_json::json!({
            "schedule_id_list": data.schedule_id_list,
        });
        let detail_resp = self.post("/cgi-bin/oa/schedule/get", &detail_body).await?;
        let detail_text = detail_resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let details: ScheduleDetailResp = serde_json::from_str(&detail_text)?;
        Ok(Page {
            items: details.schedule.into_iter().map(Into::into).collect(),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_event(&self, event_id: &str) -> ImResult<CalendarEvent> {
        let body = serde_json::json!({
            "schedule_id_list": [event_id],
        });
        let resp = self.post("/cgi-bin/oa/schedule/get", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: ScheduleDetailResp = serde_json::from_str(&text)?;
        data.schedule.into_iter().next().map(Into::into).ok_or_else(|| ImError::NotFound {
            resource: format!("schedule {}", event_id),
        })
    }

    async fn update_event(&self, req: UpdateEventRequest) -> ImResult<CalendarEvent> {
        let mut sched = serde_json::Map::new();
        sched.insert("schedule_id".into(), serde_json::json!(req.event_id));
        if let Some(ref t) = req.title { sched.insert("summary".into(), serde_json::json!(t)); }
        if let Some(ref d) = req.description { sched.insert("description".into(), serde_json::json!(d)); }
        if let Some(ref st) = req.start_time { sched.insert("start_time".into(), serde_json::json!(st.timestamp().to_string())); }
        if let Some(ref et) = req.end_time { sched.insert("end_time".into(), serde_json::json!(et.timestamp().to_string())); }
        if let Some(ref l) = req.location { sched.insert("location".into(), serde_json::json!(l)); }

        let body = serde_json::json!({ "schedule": sched });
        let resp = self.post("/cgi-bin/oa/schedule/update", &body).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        // Fetch updated event
        self.get_event(&req.event_id).await
    }

    async fn delete_event(&self, event_id: &str) -> ImResult<()> {
        let body = serde_json::json!({ "schedule_id": event_id });
        let resp = self.post("/cgi-bin/oa/schedule/del", &body).await?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }

    async fn get_free_busy(&self, req: FreeBusyRequest) -> ImResult<Vec<BusySlot>> {
        let body = serde_json::json!({
            "check_user_list": req.user_ids,
            "start_time": req.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            "end_time": req.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        let resp = self.post("/cgi-bin/oa/schedule/check_availability", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let mut slots = vec![];
        if let Some(arr) = val.get("user_busy_list").and_then(|v| v.as_array()) {
            for item in arr {
                let uid = item.get("userid").and_then(|v| v.as_str()).unwrap_or_default();
                if let Some(busy) = item.get("busy_slots").and_then(|v| v.as_array()) {
                    for slot in busy {
                        let st = slot.get("start_time").and_then(|v| v.as_i64()).unwrap_or(0);
                        let et = slot.get("end_time").and_then(|v| v.as_i64()).unwrap_or(0);
                        slots.push(BusySlot {
                            user_id: uid.to_string(),
                            start_time: ts_to_dt(st),
                            end_time: ts_to_dt(et),
                            subject: slot.get("subject").and_then(|v| v.as_str()).map(String::from),
                        });
                    }
                }
            }
        }
        Ok(slots)
    }
}
