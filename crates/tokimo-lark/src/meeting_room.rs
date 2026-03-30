use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MeetingRoomService, ImResult, ImError,
    MeetingRoom, Page, SearchRoomRequest, BookRoomRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct RoomListData {
    #[serde(default)]
    rooms: Vec<LarkRoom>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct RoomData {
    room: Option<LarkRoom>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkRoom {
    room_id: Option<String>,
    name: Option<String>,
    capacity: Option<u32>,
    description: Option<String>,
}

impl From<LarkRoom> for MeetingRoom {
    fn from(r: LarkRoom) -> Self {
        MeetingRoom {
            id: r.room_id.unwrap_or_default(),
            name: r.name.unwrap_or_default(),
            capacity: r.capacity,
            location: r.description,
            has_video: None,
            is_available: None,
        }
    }
}

#[async_trait]
impl MeetingRoomService for LarkClient {
    async fn search_rooms(&self, req: SearchRoomRequest) -> ImResult<Page<MeetingRoom>> {
        let body = serde_json::json!({
            "query": req.keyword.unwrap_or_default(),
            "page_size": req.limit.unwrap_or(20),
            "page_token": req.cursor.unwrap_or_default(),
        });
        let resp = self.post("/open-apis/vc/v1/rooms/search", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<RoomListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(RoomListData { rooms: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.rooms.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_room(&self, room_id: &str) -> ImResult<MeetingRoom> {
        let path = format!("/open-apis/vc/v1/rooms/{}", room_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<RoomData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let room = data.data.and_then(|d| d.room).ok_or_else(|| ImError::NotFound {
            resource: room_id.into(),
        })?;
        Ok(room.into())
    }

    async fn book_room(&self, _req: BookRoomRequest) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "book_room (use calendar event creation with room_id in Lark)".into(),
            platform: "lark".into(),
        })
    }

    async fn cancel_room(&self, _room_id: &str, _event_id: &str) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "cancel_room (cancel the associated calendar event in Lark)".into(),
            platform: "lark".into(),
        })
    }
}
