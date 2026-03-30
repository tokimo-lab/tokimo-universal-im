use tokimo_core::{
    Platform, ImProvider,
    AuthService, MessagingService, ContactService, GroupService, CalendarService, TaskService,
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

    fn contact(&self) -> Option<&dyn ContactService> {
        Some(&self.client)
    }

    fn group(&self) -> Option<&dyn GroupService> {
        Some(&self.client)
    }

    fn calendar(&self) -> Option<&dyn CalendarService> {
        Some(&self.client)
    }

    fn task(&self) -> Option<&dyn TaskService> {
        Some(&self.client)
    }
}
