use serde::{Deserialize, Serialize};

/// A meeting / video conference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    /// Platform-specific meeting ID.
    pub id: String,
    /// Meeting title.
    pub title: String,
    /// Meeting description.
    pub description: Option<String>,
    /// Meeting start time.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Duration in seconds.
    pub duration_secs: u64,
    /// Meeting location or link.
    pub location: Option<String>,
    /// Meeting join code (e.g., "xxx-xxx-xxx").
    pub meeting_code: Option<String>,
    /// Meeting join link.
    pub meeting_link: Option<String>,
    /// Meeting status.
    pub status: MeetingStatus,
    /// Meeting type.
    pub meeting_type: MeetingType,
    /// Creator user ID.
    pub creator_id: Option<String>,
    /// Attendees.
    #[serde(default)]
    pub attendees: Vec<MeetingAttendee>,
    /// Meeting settings.
    pub settings: Option<MeetingSettings>,
    /// Platform-specific extra data.
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// Meeting status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeetingStatus {
    Pending,
    Ongoing,
    Ended,
    Cancelled,
    Expired,
}

/// Meeting type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeetingType {
    Once,
    Recurring,
    Webinar,
    Other,
}

/// Meeting attendee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingAttendee {
    pub user_id: String,
    pub name: Option<String>,
    /// Whether the attendee has joined.
    pub joined: bool,
    /// Total time spent in the meeting (seconds).
    pub cumulative_time_secs: Option<u64>,
}

/// Meeting settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSettings {
    pub password: Option<String>,
    pub enable_waiting_room: Option<bool>,
    pub allow_enter_before_host: Option<bool>,
    pub mute_on_entry: Option<bool>,
    pub allow_external_user: Option<bool>,
}

/// Request to create a meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMeetingRequest {
    pub title: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Duration in seconds.
    pub duration_secs: u64,
    pub description: Option<String>,
    pub location: Option<String>,
    pub invitee_ids: Vec<String>,
    pub settings: Option<MeetingSettings>,
}

/// Request to list meetings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMeetingsRequest {
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Request to update meeting members (full replacement).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMeetingMembersRequest {
    pub meeting_id: String,
    pub invitee_ids: Vec<String>,
}
