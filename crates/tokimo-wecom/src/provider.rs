use tokimo_core::{
    Platform, ImProvider,
    AuthService, MessagingService, ContactService, GroupService, CalendarService, TaskService,
    MeetingService, ChatListService, MediaService, MessageExtService, DocumentService,
};
use crate::client::WeComClient;

/// WeCom (企业微信) platform provider.
pub struct WeComProvider {
    client: WeComClient,
}

impl WeComProvider {
    pub fn new(corp_id: impl Into<String>, corp_secret: impl Into<String>) -> Self {
        Self {
            client: WeComClient::new(corp_id, corp_secret),
        }
    }

    pub fn client(&self) -> &WeComClient {
        &self.client
    }
}

impl ImProvider for WeComProvider {
    fn platform(&self) -> Platform {
        Platform::WeCom
    }

    fn auth(&self) -> &dyn AuthService {
        &self.client
    }

    fn messaging(&self) -> Option<&dyn MessagingService> {
        Some(&self.client)
    }

    fn message_ext(&self) -> Option<&dyn MessageExtService> {
        None
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
}
