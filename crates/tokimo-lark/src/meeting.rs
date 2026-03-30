use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MeetingService, ImResult, ImError,
    Meeting, MeetingStatus, MeetingType, MeetingAttendee, Page,
    CreateMeetingRequest, ListMeetingsRequest, UpdateMeetingMembersRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct MeetingData {
    meeting: Option<LarkMeeting>,
}

#[derive(Deserialize)]
struct SearchData {
    #[serde(default)]
    meeting_list: Vec<LarkMeeting>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct LarkMeeting {
    id: Option<String>,
    topic: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    meeting_no: Option<String>,
    url: Option<String>,
    #[serde(default)]
    participants: Vec<LarkParticipant>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkParticipant {
    id: Option<String>,
    user_type: Option<i32>,
}

fn ts_str(s: &str) -> chrono::DateTime<chrono::Utc> {
    let ts: i64 = s.parse().unwrap_or(0);
    chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(|| chrono::Utc::now())
}

impl From<LarkMeeting> for Meeting {
    fn from(m: LarkMeeting) -> Self {
        let start = m.start_time.as_deref().map(ts_str).unwrap_or_else(chrono::Utc::now);
        let end = m.end_time.as_deref().map(ts_str).unwrap_or_else(chrono::Utc::now);
        let dur = (end - start).num_seconds().max(0) as u64;
        Meeting {
            id: m.id.unwrap_or_default(),
            title: m.topic.unwrap_or_default(),
            description: None,
            start_time: start,
            duration_secs: dur,
            location: None,
            meeting_code: m.meeting_no,
            meeting_link: m.url,
            status: MeetingStatus::Pending,
            meeting_type: MeetingType::Once,
            creator_id: None,
            attendees: m.participants.into_iter().map(|p| MeetingAttendee {
                user_id: p.id.unwrap_or_default(),
                name: None,
                joined: false,
                cumulative_time_secs: None,
            }).collect(),
            settings: None,
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl MeetingService for LarkClient {
    async fn create_meeting(&self, _req: CreateMeetingRequest) -> ImResult<Meeting> {
        // Lark VC API requires special permissions; provide search instead
        Err(ImError::NotSupported {
            feature: "create_meeting (use Lark calendar events with video conference)".into(),
            platform: "lark".into(),
        })
    }

    async fn list_meetings(&self, req: ListMeetingsRequest) -> ImResult<Page<Meeting>> {
        let mut body = serde_json::Map::new();
        if let Some(ref st) = req.start_time {
            body.insert("start_time".into(), serde_json::json!(st.timestamp().to_string()));
        }
        if let Some(ref et) = req.end_time {
            body.insert("end_time".into(), serde_json::json!(et.timestamp().to_string()));
        }
        if let Some(ref cursor) = req.cursor {
            body.insert("page_token".into(), serde_json::json!(cursor));
        }
        if let Some(limit) = req.limit {
            body.insert("page_size".into(), serde_json::json!(limit));
        }
        let resp = self.post("/open-apis/vc/v1/meetings/search", &serde_json::Value::Object(body)).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SearchData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let search = data.data.unwrap_or(SearchData { meeting_list: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: search.meeting_list.into_iter().map(Into::into).collect(),
            has_more: search.has_more.unwrap_or(false),
            next_cursor: search.page_token,
        })
    }

    async fn get_meeting(&self, meeting_id: &str) -> ImResult<Meeting> {
        let path = format!("/open-apis/vc/v1/meetings/{}", meeting_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<MeetingData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let meeting = data.data.and_then(|d| d.meeting).ok_or_else(|| ImError::NotFound {
            resource: meeting_id.into(),
        })?;
        Ok(meeting.into())
    }

    async fn cancel_meeting(&self, _meeting_id: &str) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "cancel_meeting".into(),
            platform: "lark".into(),
        })
    }

    async fn update_meeting_members(&self, _req: UpdateMeetingMembersRequest) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "update_meeting_members".into(),
            platform: "lark".into(),
        })
    }
}
