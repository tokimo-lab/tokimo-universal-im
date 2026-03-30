use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{MeetingRoom, Page, SearchRoomRequest, BookRoomRequest};

/// Meeting room search and booking.
#[async_trait]
pub trait MeetingRoomService: Send + Sync {
    /// Search available meeting rooms.
    async fn search_rooms(&self, req: SearchRoomRequest) -> ImResult<Page<MeetingRoom>>;

    /// Get details for a specific room.
    async fn get_room(&self, room_id: &str) -> ImResult<MeetingRoom>;

    /// Book a meeting room for a calendar event.
    async fn book_room(&self, req: BookRoomRequest) -> ImResult<()>;

    /// Cancel a room booking.
    async fn cancel_room(&self, room_id: &str, event_id: &str) -> ImResult<()>;
}
