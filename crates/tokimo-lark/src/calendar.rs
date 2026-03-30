use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    CalendarService, ImResult, ImError,
    CalendarEvent, EventAttendee, AttendeeStatus, Page, BusySlot,
    CreateEventRequest, UpdateEventRequest, ListEventsRequest, FreeBusyRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct EventData {
    event: Option<LarkEvent>,
}

#[derive(Deserialize)]
struct ListEventData {
    #[serde(default)]
    items: Vec<LarkEvent>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct LarkEvent {
    event_id: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    start_time: Option<TimeInfo>,
    end_time: Option<TimeInfo>,
    location: Option<LocationInfo>,
    #[serde(default)]
    attendees: Vec<LarkAttendee>,
}

#[derive(Deserialize)]
struct TimeInfo {
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct LocationInfo {
    name: Option<String>,
}

#[derive(Deserialize)]
struct LarkAttendee {
    user_id: Option<String>,
    display_name: Option<String>,
    status: Option<String>,
}

fn ts_str_to_dt(s: &str) -> chrono::DateTime<chrono::Utc> {
    let ts: i64 = s.parse().unwrap_or(0);
    chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(|| chrono::Utc::now())
}

impl From<LarkEvent> for CalendarEvent {
    fn from(e: LarkEvent) -> Self {
        CalendarEvent {
            id: e.event_id.unwrap_or_default(),
            title: e.summary.unwrap_or_default(),
            description: e.description,
            start_time: e.start_time.and_then(|t| t.timestamp).map(|s| ts_str_to_dt(&s)).unwrap_or_else(chrono::Utc::now),
            end_time: e.end_time.and_then(|t| t.timestamp).map(|s| ts_str_to_dt(&s)).unwrap_or_else(chrono::Utc::now),
            location: e.location.and_then(|l| l.name),
            is_all_day: false,
            attendees: e.attendees.into_iter().map(|a| EventAttendee {
                user_id: a.user_id.unwrap_or_default(),
                name: a.display_name,
                status: match a.status.as_deref() {
                    Some("accept") => AttendeeStatus::Accepted,
                    Some("decline") => AttendeeStatus::Declined,
                    Some("tentative") => AttendeeStatus::Tentative,
                    _ => AttendeeStatus::Unknown,
                },
            }).collect(),
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl CalendarService for LarkClient {
    async fn create_event(&self, req: CreateEventRequest) -> ImResult<CalendarEvent> {
        let body = serde_json::json!({
            "summary": req.title,
            "description": req.description,
            "start_time": { "timestamp": req.start_time.timestamp().to_string() },
            "end_time": { "timestamp": req.end_time.timestamp().to_string() },
            "location": req.location.map(|l| serde_json::json!({"name": l})),
            "attendees": req.attendee_ids.iter().map(|id| serde_json::json!({"user_id": id})).collect::<Vec<_>>(),
        });
        let resp = self.post("/open-apis/calendar/v4/calendars/primary/events", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<EventData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let event = data.data.and_then(|d| d.event).ok_or_else(|| ImError::Internal("no event".into()))?;
        Ok(event.into())
    }

    async fn list_events(&self, req: ListEventsRequest) -> ImResult<Page<CalendarEvent>> {
        let mut path = format!(
            "/open-apis/calendar/v4/calendars/primary/events?start_time={}&end_time={}",
            req.start_time.timestamp(),
            req.end_time.timestamp()
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        if let Some(limit) = req.limit {
            path.push_str(&format!("&page_size={}", limit));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ListEventData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ListEventData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_event(&self, event_id: &str) -> ImResult<CalendarEvent> {
        let path = format!("/open-apis/calendar/v4/calendars/primary/events/{}", event_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<EventData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let event = data.data.and_then(|d| d.event).ok_or_else(|| ImError::NotFound { resource: event_id.into() })?;
        Ok(event.into())
    }

    async fn update_event(&self, req: UpdateEventRequest) -> ImResult<CalendarEvent> {
        let mut body = serde_json::Map::new();
        if let Some(ref t) = req.title { body.insert("summary".into(), serde_json::json!(t)); }
        if let Some(ref d) = req.description { body.insert("description".into(), serde_json::json!(d)); }
        if let Some(ref st) = req.start_time { body.insert("start_time".into(), serde_json::json!({"timestamp": st.timestamp().to_string()})); }
        if let Some(ref et) = req.end_time { body.insert("end_time".into(), serde_json::json!({"timestamp": et.timestamp().to_string()})); }
        if let Some(ref l) = req.location { body.insert("location".into(), serde_json::json!({"name": l})); }

        let path = format!("/open-apis/calendar/v4/calendars/primary/events/{}", req.event_id);
        let resp = self.put(&path, &serde_json::Value::Object(body)).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<EventData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let event = data.data.and_then(|d| d.event).ok_or_else(|| ImError::Internal("no event".into()))?;
        Ok(event.into())
    }

    async fn delete_event(&self, event_id: &str) -> ImResult<()> {
        let path = format!("/open-apis/calendar/v4/calendars/primary/events/{}", event_id);
        let resp = self.delete(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<serde_json::Value> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(())
    }

    async fn get_free_busy(&self, req: FreeBusyRequest) -> ImResult<Vec<BusySlot>> {
        let body = serde_json::json!({
            "time_min": req.start_time.timestamp().to_string(),
            "time_max": req.end_time.timestamp().to_string(),
            "user_id_list": req.user_ids,
        });
        let resp = self.post("/open-apis/calendar/v4/calendars/primary/freebusy/query", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let mut slots = vec![];
        if let Some(arr) = val.get("data").and_then(|d| d.get("freebusy_list")).and_then(|v| v.as_array()) {
            for item in arr {
                let uid = item.get("user_id").and_then(|v| v.as_str()).unwrap_or_default();
                if let Some(busy) = item.get("time_ranges").and_then(|v| v.as_array()) {
                    for slot in busy {
                        let st = slot.get("start_time").and_then(|v| v.as_str()).unwrap_or("0");
                        let et = slot.get("end_time").and_then(|v| v.as_str()).unwrap_or("0");
                        slots.push(BusySlot {
                            user_id: uid.to_string(),
                            start_time: ts_str_to_dt(st),
                            end_time: ts_str_to_dt(et),
                            subject: None,
                        });
                    }
                }
            }
        }
        Ok(slots)
    }
}
