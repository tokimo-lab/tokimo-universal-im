use serde::{Deserialize, Serialize};

/// A meeting room / conference room resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRoom {
    /// Room ID.
    pub id: String,
    /// Room name.
    pub name: String,
    /// Capacity.
    pub capacity: Option<u32>,
    /// Building / floor location.
    pub location: Option<String>,
    /// Whether the room has video equipment.
    pub has_video: Option<bool>,
    /// Whether the room is currently available.
    pub is_available: Option<bool>,
}

/// Request to search for meeting rooms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRoomRequest {
    /// Search keyword.
    pub keyword: Option<String>,
    /// Start time for availability check.
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// End time for availability check.
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Request to book a meeting room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookRoomRequest {
    /// Room ID.
    pub room_id: String,
    /// Associated calendar event ID.
    pub event_id: String,
}
