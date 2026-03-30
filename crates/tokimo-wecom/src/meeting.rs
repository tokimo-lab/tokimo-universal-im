use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MeetingService, ImResult, ImError,
    Meeting, MeetingStatus, MeetingType, MeetingAttendee, MeetingSettings, Page,
    CreateMeetingRequest, ListMeetingsRequest, UpdateMeetingMembersRequest,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct CreateResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    meetingid: Option<String>,
    meeting_code: Option<String>,
    meeting_link: Option<String>,
}

#[derive(Deserialize)]
struct ListResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    meetingid_list: Vec<String>,
    next_cursor: Option<String>,
}

#[derive(Deserialize)]
struct MeetingInfo {
    meetingid: Option<String>,
    title: Option<String>,
    description: Option<String>,
    meeting_start_datetime: Option<String>,
    meeting_duration: Option<u64>,
    meeting_code: Option<String>,
    meeting_link: Option<String>,
    status: Option<i32>,
    meeting_type: Option<i32>,
    creator_userid: Option<String>,
    admin_userid: Option<String>,
    attendees: Option<AttendeesInfo>,
    settings: Option<WcSettings>,
}

#[derive(Deserialize)]
struct AttendeesInfo {
    #[serde(default)]
    member: Vec<WcMeetingMember>,
}

#[derive(Deserialize)]
struct WcMeetingMember {
    userid: Option<String>,
    status: Option<i32>,
    cumulative_time: Option<u64>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct WcSettings {
    need_password: Option<bool>,
    password: Option<String>,
    enable_waiting_room: Option<bool>,
    allow_enter_before_host: Option<bool>,
    enable_enter_mute: Option<i32>,
    allow_external_user: Option<bool>,
}

fn parse_wc_meeting_dt(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| chrono::Utc::now())
}

impl From<MeetingInfo> for Meeting {
    fn from(m: MeetingInfo) -> Self {
        Meeting {
            id: m.meetingid.unwrap_or_default(),
            title: m.title.unwrap_or_default(),
            description: m.description,
            start_time: m.meeting_start_datetime.as_deref().map(parse_wc_meeting_dt).unwrap_or_else(chrono::Utc::now),
            duration_secs: m.meeting_duration.unwrap_or(0),
            location: None,
            meeting_code: m.meeting_code,
            meeting_link: m.meeting_link,
            status: match m.status.unwrap_or(0) {
                1 => MeetingStatus::Pending,
                2 => MeetingStatus::Ongoing,
                3 => MeetingStatus::Ended,
                4 => MeetingStatus::Cancelled,
                5 => MeetingStatus::Expired,
                _ => MeetingStatus::Pending,
            },
            meeting_type: match m.meeting_type.unwrap_or(0) {
                0 => MeetingType::Once,
                1 => MeetingType::Recurring,
                6 => MeetingType::Webinar,
                _ => MeetingType::Other,
            },
            creator_id: m.creator_userid.or(m.admin_userid),
            attendees: m.attendees.map(|a| a.member.into_iter().map(|mm| MeetingAttendee {
                user_id: mm.userid.unwrap_or_default(),
                name: None,
                joined: mm.status.unwrap_or(0) == 1,
                cumulative_time_secs: mm.cumulative_time,
            }).collect()).unwrap_or_default(),
            settings: m.settings.map(|s| MeetingSettings {
                password: s.password,
                enable_waiting_room: s.enable_waiting_room,
                allow_enter_before_host: s.allow_enter_before_host,
                mute_on_entry: s.enable_enter_mute.map(|v| v == 1),
                allow_external_user: s.allow_external_user,
            }),
            extra: serde_json::Value::Null,
        }
    }
}

#[async_trait]
impl MeetingService for WeComClient {
    async fn create_meeting(&self, req: CreateMeetingRequest) -> ImResult<Meeting> {
        let start = req.start_time.format("%Y-%m-%d %H:%M").to_string();
        let body = serde_json::json!({
            "title": req.title,
            "meeting_start_datetime": start,
            "meeting_duration": req.duration_secs,
            "description": req.description,
            "location": req.location,
            "invitees": { "userid": req.invitee_ids },
            "settings": req.settings.as_ref().map(|s| serde_json::json!({
                "password": s.password,
                "enable_waiting_room": s.enable_waiting_room,
                "allow_enter_before_host": s.allow_enter_before_host,
                "enable_enter_mute": s.mute_on_entry.map(|v| if v { 1 } else { 0 }),
                "allow_external_user": s.allow_external_user,
            })),
        });
        let resp = self.post("/cgi-bin/meeting/create", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: CreateResp = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform { code: data.errcode.unwrap_or(-1), message: data.errmsg.unwrap_or(text) });
        }
        Ok(Meeting {
            id: data.meetingid.unwrap_or_default(),
            title: req.title,
            description: req.description,
            start_time: req.start_time,
            duration_secs: req.duration_secs,
            location: req.location,
            meeting_code: data.meeting_code,
            meeting_link: data.meeting_link,
            status: MeetingStatus::Pending,
            meeting_type: MeetingType::Once,
            creator_id: None,
            attendees: vec![],
            settings: req.settings,
            extra: serde_json::Value::Null,
        })
    }

    async fn list_meetings(&self, req: ListMeetingsRequest) -> ImResult<Page<Meeting>> {
        let body = serde_json::json!({
            "begin_datetime": req.start_time.map(|t| t.format("%Y-%m-%d %H:%M").to_string()),
            "end_datetime": req.end_time.map(|t| t.format("%Y-%m-%d %H:%M").to_string()),
            "cursor": req.cursor,
            "limit": req.limit.unwrap_or(20),
        });
        let resp = self.post("/cgi-bin/meeting/list_user_meetings", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: ListResp = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform { code: data.errcode.unwrap_or(-1), message: data.errmsg.unwrap_or(text) });
        }
        // Fetch details for each meeting
        let mut meetings = vec![];
        for mid in &data.meetingid_list {
            if let Ok(m) = self.get_meeting(mid).await {
                meetings.push(m);
            }
        }
        Ok(Page {
            items: meetings,
            has_more: data.next_cursor.is_some(),
            next_cursor: data.next_cursor,
        })
    }

    async fn get_meeting(&self, meeting_id: &str) -> ImResult<Meeting> {
        let body = serde_json::json!({ "meetingid": meeting_id });
        let resp = self.post("/cgi-bin/meeting/get_info", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let info: MeetingInfo = serde_json::from_str(&text)?;
        Ok(info.into())
    }

    async fn cancel_meeting(&self, meeting_id: &str) -> ImResult<()> {
        let body = serde_json::json!({ "meetingid": meeting_id });
        let resp = self.post("/cgi-bin/meeting/cancel", &body).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        Ok(())
    }

    async fn update_meeting_members(&self, req: UpdateMeetingMembersRequest) -> ImResult<()> {
        let body = serde_json::json!({
            "meetingid": req.meeting_id,
            "invitees": req.invitee_ids.iter().map(|id| serde_json::json!({"userid": id})).collect::<Vec<_>>(),
        });
        let resp = self.post("/cgi-bin/meeting/set_invite_members", &body).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        Ok(())
    }
}
