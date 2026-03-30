use serde::{Deserialize, Serialize};

/// A calendar event / schedule item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// Platform-specific event ID.
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub location: Option<String>,
    pub is_all_day: bool,
    #[serde(default)]
    pub attendees: Vec<EventAttendee>,
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// An attendee of a calendar event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAttendee {
    pub user_id: String,
    pub name: Option<String>,
    pub status: AttendeeStatus,
}

/// RSVP status for a calendar event attendee.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttendeeStatus {
    Accepted,
    Declined,
    Tentative,
    Unknown,
}

/// Request to create a calendar event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub location: Option<String>,
    pub is_all_day: bool,
    pub attendee_ids: Vec<String>,
}

/// Request to update a calendar event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEventRequest {
    pub event_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub location: Option<String>,
}

/// Request to list calendar events within a time range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEventsRequest {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// A free/busy time slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusySlot {
    pub user_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub subject: Option<String>,
}

/// Request to check free/busy status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeBusyRequest {
    pub user_ids: Vec<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
}
