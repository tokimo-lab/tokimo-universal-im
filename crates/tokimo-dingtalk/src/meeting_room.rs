use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MeetingRoomService, ImResult, ImError,
    MeetingRoom, Page, SearchRoomRequest, BookRoomRequest,
};
use crate::client::DingTalkClient;

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtRoom {
    #[serde(rename = "roomId")]
    room_id: Option<String>,
    title: Option<String>,
    capacity: Option<u32>,
    #[serde(rename = "roomLocation")]
    room_location: Option<DtRoomLocation>,
    #[serde(rename = "roomStatus")]
    room_status: Option<u32>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtRoomLocation {
    #[serde(rename = "buildingName")]
    building_name: Option<String>,
    #[serde(rename = "floorName")]
    floor_name: Option<String>,
}

impl From<DtRoom> for MeetingRoom {
    fn from(r: DtRoom) -> Self {
        let location = r.room_location.map(|loc| {
            let mut parts = Vec::new();
            if let Some(b) = loc.building_name { parts.push(b); }
            if let Some(f) = loc.floor_name { parts.push(f); }
            parts.join(" ")
        });
        MeetingRoom {
            id: r.room_id.unwrap_or_default(),
            name: r.title.unwrap_or_default(),
            capacity: r.capacity,
            location,
            has_video: None,
            is_available: r.room_status.map(|s| s == 0),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtRoomListResponse {
    #[serde(default)]
    rooms: Vec<DtRoom>,
    #[serde(rename = "hasMore")]
    has_more: Option<bool>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[async_trait]
impl MeetingRoomService for DingTalkClient {
    async fn search_rooms(&self, req: SearchRoomRequest) -> ImResult<Page<MeetingRoom>> {
        let mut body = serde_json::Map::new();
        if let Some(ref kw) = req.keyword {
            body.insert("keyword".into(), serde_json::json!(kw));
        }
        if let Some(ref st) = req.start_time {
            body.insert("startTime".into(), serde_json::json!(st.to_rfc3339()));
        }
        if let Some(ref et) = req.end_time {
            body.insert("endTime".into(), serde_json::json!(et.to_rfc3339()));
        }
        if let Some(ref cursor) = req.cursor {
            body.insert("nextCursor".into(), serde_json::json!(cursor));
        }
        if let Some(limit) = req.limit {
            body.insert("maxResults".into(), serde_json::json!(limit));
        }

        let resp = self.post(
            "/v1.0/calendar/rooms/search",
            &serde_json::Value::Object(body),
        ).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtRoomListResponse = serde_json::from_str(&text)?;
        Ok(Page {
            items: data.rooms.into_iter().map(Into::into).collect(),
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next_cursor,
        })
    }

    async fn get_room(&self, room_id: &str) -> ImResult<MeetingRoom> {
        let path = format!("/v1.0/calendar/rooms/{}", room_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let r: DtRoom = serde_json::from_str(&text)?;
        Ok(r.into())
    }

    async fn book_room(&self, req: BookRoomRequest) -> ImResult<()> {
        let body = serde_json::json!({
            "roomId": req.room_id
        });
        let path = format!("/v1.0/calendar/events/{}/rooms", req.event_id);
        let resp = self.post(&path, &body).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        Ok(())
    }

    async fn cancel_room(&self, room_id: &str, event_id: &str) -> ImResult<()> {
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!(
            "{}/v1.0/calendar/events/{}/rooms/{}",
            self.base_url, event_id, room_id
        );
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
}
