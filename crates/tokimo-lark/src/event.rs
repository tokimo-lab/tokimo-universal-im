use async_trait::async_trait;
use tokimo_core::{
    EventService, ImResult, ImError,
    ImEvent, RegisterCallbackRequest, EventSubscription,
};
use crate::client::LarkClient;

#[async_trait]
impl EventService for LarkClient {
    async fn register_callback(&self, _req: RegisterCallbackRequest) -> ImResult<EventSubscription> {
        Err(ImError::NotSupported {
            feature: "register_callback (configure callback URL in Lark developer console)".into(),
            platform: "lark".into(),
        })
    }

    async fn list_subscriptions(&self) -> ImResult<Vec<EventSubscription>> {
        Err(ImError::NotSupported {
            feature: "list_subscriptions (manage subscriptions in Lark developer console)".into(),
            platform: "lark".into(),
        })
    }

    async fn delete_subscription(&self, _subscription_id: &str) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "delete_subscription (manage subscriptions in Lark developer console)".into(),
            platform: "lark".into(),
        })
    }

    async fn list_event_types(&self) -> ImResult<Vec<String>> {
        Ok(vec![
            "im.message.receive_v1".into(),
            "im.message.message_read_v1".into(),
            "im.chat.disbanded_v1".into(),
            "im.chat.updated_v1".into(),
            "im.chat.member.bot.added_v1".into(),
            "im.chat.member.bot.deleted_v1".into(),
            "im.chat.member.user.added_v1".into(),
            "im.chat.member.user.deleted_v1".into(),
            "contact.user.created_v3".into(),
            "contact.user.deleted_v3".into(),
            "contact.user.updated_v3".into(),
            "contact.department.created_v3".into(),
            "contact.department.deleted_v3".into(),
            "contact.department.updated_v3".into(),
            "calendar.calendar.acl.created_v4".into(),
            "calendar.calendar.event.changed_v4".into(),
            "approval.instance.status_updated".into(),
            "drive.file.permission_member_added_v1".into(),
            "vc.meeting.meeting_started_v1".into(),
            "vc.meeting.meeting_ended_v1".into(),
        ])
    }

    async fn poll_events(&self) -> ImResult<Vec<ImEvent>> {
        Err(ImError::NotSupported {
            feature: "poll_events (use Lark WebSocket client for real-time events)".into(),
            platform: "lark".into(),
        })
    }
}
