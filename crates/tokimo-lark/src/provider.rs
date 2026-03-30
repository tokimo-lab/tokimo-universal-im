use tokimo_core::{
    Platform, ImProvider,
    AuthService, MessagingService, ContactService, GroupService, CalendarService, TaskService,
    MeetingService, ChatListService, MediaService, MessageExtService, DocumentService,
    EventService, DepartmentService, MeetingRoomService, ApprovalService, AttendanceService,
    DataTableService, WikiService, EmailService,
};
use crate::client::{LarkClient, LarkRegion};

/// Lark/Feishu (飞书) platform provider.
pub struct LarkProvider {
    client: LarkClient,
}

impl LarkProvider {
    pub fn new(app_id: impl Into<String>, app_secret: impl Into<String>, region: LarkRegion) -> Self {
        Self {
            client: LarkClient::new(app_id, app_secret, region),
        }
    }

    /// Create a provider using Feishu (China) endpoints.
    pub fn feishu(app_id: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self::new(app_id, app_secret, LarkRegion::Feishu)
    }

    /// Create a provider using Lark (International) endpoints.
    pub fn lark(app_id: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self::new(app_id, app_secret, LarkRegion::Lark)
    }

    pub fn client(&self) -> &LarkClient {
        &self.client
    }
}

impl ImProvider for LarkProvider {
    fn platform(&self) -> Platform {
        Platform::Lark
    }

    fn auth(&self) -> &dyn AuthService {
        &self.client
    }

    fn messaging(&self) -> Option<&dyn MessagingService> {
        Some(&self.client)
    }

    fn message_ext(&self) -> Option<&dyn MessageExtService> {
        Some(&self.client)
    }

    fn contact(&self) -> Option<&dyn ContactService> {
        Some(&self.client)
    }

    fn group(&self) -> Option<&dyn GroupService> {
        Some(&self.client)
    }

    fn chat_list(&self) -> Option<&dyn ChatListService> {
        Some(&self.client)
    }

    fn calendar(&self) -> Option<&dyn CalendarService> {
        Some(&self.client)
    }

    fn task(&self) -> Option<&dyn TaskService> {
        Some(&self.client)
    }

    fn meeting(&self) -> Option<&dyn MeetingService> {
        Some(&self.client)
    }

    fn media(&self) -> Option<&dyn MediaService> {
        Some(&self.client)
    }

    fn document(&self) -> Option<&dyn DocumentService> {
        Some(&self.client)
    }

    fn event(&self) -> Option<&dyn EventService> {
        Some(&self.client)
    }

    fn department(&self) -> Option<&dyn DepartmentService> {
        Some(&self.client)
    }

    fn meeting_room(&self) -> Option<&dyn MeetingRoomService> {
        Some(&self.client)
    }

    fn approval(&self) -> Option<&dyn ApprovalService> {
        Some(&self.client)
    }

    fn attendance(&self) -> Option<&dyn AttendanceService> {
        Some(&self.client)
    }

    fn data_table(&self) -> Option<&dyn DataTableService> {
        Some(&self.client)
    }

    fn wiki(&self) -> Option<&dyn WikiService> {
        Some(&self.client)
    }

    fn email(&self) -> Option<&dyn EmailService> {
        Some(&self.client)
    }
}
