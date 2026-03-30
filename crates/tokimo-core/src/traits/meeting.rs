use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    Meeting, Page,
    CreateMeetingRequest, ListMeetingsRequest, UpdateMeetingMembersRequest,
};

/// Meeting / video conference operations.
#[async_trait]
pub trait MeetingService: Send + Sync {
    /// Create a new meeting.
    async fn create_meeting(&self, req: CreateMeetingRequest) -> ImResult<Meeting>;

    /// List meetings within a time range.
    async fn list_meetings(&self, req: ListMeetingsRequest) -> ImResult<Page<Meeting>>;

    /// Get a single meeting by ID.
    async fn get_meeting(&self, meeting_id: &str) -> ImResult<Meeting>;

    /// Cancel a meeting.
    async fn cancel_meeting(&self, meeting_id: &str) -> ImResult<()>;

    /// Update meeting members (full replacement).
    async fn update_meeting_members(&self, req: UpdateMeetingMembersRequest) -> ImResult<()>;
}
