use tokimo_core::{
    Platform, ImProvider,
    AuthService, MessagingService, ContactService, GroupService, CalendarService, TaskService,
};
use crate::client::DingTalkClient;

/// DingTalk platform provider.
///
/// Wraps [`DingTalkClient`] and exposes all supported service traits.
pub struct DingTalkProvider {
    client: DingTalkClient,
}

impl DingTalkProvider {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            client: DingTalkClient::new(client_id, client_secret),
        }
    }

    /// Get direct access to the underlying client.
    pub fn client(&self) -> &DingTalkClient {
        &self.client
    }
}

impl ImProvider for DingTalkProvider {
    fn platform(&self) -> Platform {
        Platform::DingTalk
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
