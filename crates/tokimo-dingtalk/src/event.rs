use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    EventService, ImResult, ImError,
    ImEvent, RegisterCallbackRequest, EventSubscription, EventSubscriptionStatus,
};
use crate::client::DingTalkClient;

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtSubscription {
    #[serde(rename = "subscriptionId")]
    subscription_id: Option<String>,
    #[serde(rename = "callbackUrl")]
    callback_url: Option<String>,
    #[serde(default, rename = "eventTypes")]
    event_types: Vec<String>,
    status: Option<String>,
}

impl From<DtSubscription> for EventSubscription {
    fn from(s: DtSubscription) -> Self {
        let status = match s.status.as_deref() {
            Some("active") | Some("ACTIVE") => EventSubscriptionStatus::Active,
            Some("failed") | Some("FAILED") => EventSubscriptionStatus::Failed,
            _ => EventSubscriptionStatus::Inactive,
        };
        EventSubscription {
            id: s.subscription_id.unwrap_or_default(),
            callback_url: s.callback_url,
            event_types: s.event_types,
            status,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtSubscriptionListResponse {
    #[serde(default)]
    subscriptions: Vec<DtSubscription>,
}

/// Known DingTalk event types.
const DINGTALK_EVENT_TYPES: &[&str] = &[
    "user_add_org",
    "user_modify_org",
    "user_leave_org",
    "user_active_org",
    "org_dept_create",
    "org_dept_modify",
    "org_dept_remove",
    "chat_add_member",
    "chat_remove_member",
    "chat_update_owner",
    "chat_update_title",
    "chat_disband",
    "bpms_task_change",
    "bpms_instance_change",
    "attendance_check_record",
    "attendance_schedule_change",
];

#[async_trait]
impl EventService for DingTalkClient {
    async fn register_callback(&self, req: RegisterCallbackRequest) -> ImResult<EventSubscription> {
        let mut body = serde_json::Map::new();
        body.insert("callbackUrl".into(), serde_json::json!(req.callback_url));
        body.insert("eventTypes".into(), serde_json::json!(req.event_types));
        if let Some(ref token) = req.token {
            body.insert("token".into(), serde_json::json!(token));
        }
        if let Some(ref aes_key) = req.aes_key {
            body.insert("aesKey".into(), serde_json::json!(aes_key));
        }

        let resp = self.post(
            "/v1.0/events/callbacks/register",
            &serde_json::Value::Object(body),
        ).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let sub: DtSubscription = serde_json::from_str(&text)?;
        Ok(sub.into())
    }

    async fn list_subscriptions(&self) -> ImResult<Vec<EventSubscription>> {
        let resp = self.get("/v1.0/events/callbacks").await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtSubscriptionListResponse = serde_json::from_str(&text)?;
        Ok(data.subscriptions.into_iter().map(Into::into).collect())
    }

    async fn delete_subscription(&self, subscription_id: &str) -> ImResult<()> {
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!(
            "{}/v1.0/events/callbacks/{}",
            self.base_url, subscription_id
        );
        let resp = self.http
            .delete(&url)
            .header("x-acs-dingtalk-access-token", &token)
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }

    async fn list_event_types(&self) -> ImResult<Vec<String>> {
        Ok(DINGTALK_EVENT_TYPES.iter().map(|s| s.to_string()).collect())
    }

    async fn poll_events(&self) -> ImResult<Vec<ImEvent>> {
        Err(ImError::NotSupported {
            feature: "poll_events".into(),
            platform: "dingtalk".into(),
        })
    }
}
