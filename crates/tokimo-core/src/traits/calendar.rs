use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    CalendarEvent, Page, BusySlot,
    CreateEventRequest, UpdateEventRequest, ListEventsRequest, FreeBusyRequest,
};

/// Calendar / schedule operations.
#[async_trait]
pub trait CalendarService: Send + Sync {
    /// Create a calendar event.
    async fn create_event(&self, req: CreateEventRequest) -> ImResult<CalendarEvent>;

    /// List events within a time range.
    async fn list_events(&self, req: ListEventsRequest) -> ImResult<Page<CalendarEvent>>;

    /// Get a single event by ID.
    async fn get_event(&self, event_id: &str) -> ImResult<CalendarEvent>;

    /// Update an existing event.
    async fn update_event(&self, req: UpdateEventRequest) -> ImResult<CalendarEvent>;

    /// Delete an event.
    async fn delete_event(&self, event_id: &str) -> ImResult<()>;

    /// Query free/busy status for users.
    async fn get_free_busy(&self, req: FreeBusyRequest) -> ImResult<Vec<BusySlot>>;
}
