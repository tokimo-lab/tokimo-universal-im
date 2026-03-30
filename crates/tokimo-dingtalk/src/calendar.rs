use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    CalendarService, ImResult, ImError,
    CalendarEvent, Page, BusySlot,
    CreateEventRequest, UpdateEventRequest, ListEventsRequest, FreeBusyRequest,
};
use crate::client::DingTalkClient;

#[derive(Deserialize)]
struct DtEvent {
    #[serde(rename = "eventId")]
    event_id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    start: Option<DtTime>,
    end: Option<DtTime>,
    location: Option<DtLocation>,
}

#[derive(Deserialize)]
struct DtTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
}

#[derive(Deserialize)]
struct DtLocation {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

fn parse_dt(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

impl From<DtEvent> for CalendarEvent {
    fn from(e: DtEvent) -> Self {
        CalendarEvent {
            id: e.event_id.unwrap_or_default(),
            title: e.title.unwrap_or_default(),
            description: e.description,
            start_time: e.start.and_then(|t| t.date_time).map(|s| parse_dt(&s)).unwrap_or_else(chrono::Utc::now),
            end_time: e.end.and_then(|t| t.date_time).map(|s| parse_dt(&s)).unwrap_or_else(chrono::Utc::now),
            location: e.location.and_then(|l| l.display_name),
            is_all_day: false,
            attendees: vec![],
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl CalendarService for DingTalkClient {
    async fn create_event(&self, req: CreateEventRequest) -> ImResult<CalendarEvent> {
        let body = serde_json::json!({
            "title": req.title,
            "description": req.description,
            "start": { "dateTime": req.start_time.to_rfc3339() },
            "end": { "dateTime": req.end_time.to_rfc3339() },
            "location": req.location.map(|l| serde_json::json!({"displayName": l})),
        });
        let resp = self.post("/v1.0/calendar/users/me/calendars/primary/events", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let e: DtEvent = serde_json::from_str(&text)?;
        Ok(e.into())
    }

    async fn list_events(&self, req: ListEventsRequest) -> ImResult<Page<CalendarEvent>> {
        let path = format!(
            "/v1.0/calendar/users/me/calendars/primary/events?timeMin={}&timeMax={}",
            req.start_time.to_rfc3339(),
            req.end_time.to_rfc3339()
        );
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let events: Vec<DtEvent> = serde_json::from_value(
            val.get("items").cloned().unwrap_or(serde_json::Value::Array(vec![]))
        )?;
        Ok(Page {
            items: events.into_iter().map(Into::into).collect(),
            has_more: false,
            next_cursor: None,
        })
    }

    async fn get_event(&self, event_id: &str) -> ImResult<CalendarEvent> {
        let path = format!("/v1.0/calendar/users/me/calendars/primary/events/{}", event_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let e: DtEvent = serde_json::from_str(&text)?;
        Ok(e.into())
    }

    async fn update_event(&self, req: UpdateEventRequest) -> ImResult<CalendarEvent> {
        let mut body = serde_json::Map::new();
        if let Some(ref title) = req.title { body.insert("title".into(), serde_json::json!(title)); }
        if let Some(ref desc) = req.description { body.insert("description".into(), serde_json::json!(desc)); }
        if let Some(ref st) = req.start_time { body.insert("start".into(), serde_json::json!({"dateTime": st.to_rfc3339()})); }
        if let Some(ref et) = req.end_time { body.insert("end".into(), serde_json::json!({"dateTime": et.to_rfc3339()})); }

        let path = format!("/v1.0/calendar/users/me/calendars/primary/events/{}", req.event_id);
        let resp = self.post(&path, &serde_json::Value::Object(body)).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let e: DtEvent = serde_json::from_str(&text)?;
        Ok(e.into())
    }

    async fn delete_event(&self, event_id: &str) -> ImResult<()> {
        let path = format!("/v1.0/calendar/users/me/calendars/primary/events/{}", event_id);
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!("{}{}", self.base_url, path);
        let resp = self.http
            .delete(&url)
            .header("x-acs-dingtalk-access-token", &token)
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }

    async fn get_free_busy(&self, req: FreeBusyRequest) -> ImResult<Vec<BusySlot>> {
        let body = serde_json::json!({
            "userIds": req.user_ids,
            "timeMin": req.start_time.to_rfc3339(),
            "timeMax": req.end_time.to_rfc3339(),
        });
        let resp = self.post("/v1.0/calendar/users/me/calendars/primary/freeBusy/query", &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        // Parse response into BusySlots
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let mut slots = vec![];
        if let Some(arr) = val.as_array() {
            for item in arr {
                if let (Some(uid), Some(st), Some(et)) = (
                    item.get("userId").and_then(|v| v.as_str()),
                    item.get("startTime").and_then(|v| v.as_str()),
                    item.get("endTime").and_then(|v| v.as_str()),
                ) {
                    slots.push(BusySlot {
                        user_id: uid.to_string(),
                        start_time: parse_dt(st),
                        end_time: parse_dt(et),
                        subject: item.get("subject").and_then(|v| v.as_str()).map(String::from),
                    });
                }
            }
        }
        Ok(slots)
    }
}
