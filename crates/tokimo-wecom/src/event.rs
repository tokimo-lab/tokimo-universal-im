use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    EventService, ImResult, ImError,
    ImEvent, RegisterCallbackRequest, EventSubscription, EventSubscriptionStatus,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct CallbackResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GetCallbackResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    url: Option<String>,
    token: Option<String>,
    #[allow(dead_code)]
    encodingaeskey: Option<String>,
    #[serde(default)]
    event: Vec<String>,
}

fn check_errcode(errcode: Option<i64>, errmsg: Option<String>, raw: String) -> ImResult<()> {
    if errcode.unwrap_or(0) != 0 {
        return Err(ImError::Platform {
            code: errcode.unwrap_or(-1),
            message: errmsg.unwrap_or(raw),
        });
    }
    Ok(())
}

/// Known WeCom event types that can be subscribed to.
const WECOM_EVENT_TYPES: &[&str] = &[
    "change_contact",
    "change_external_contact",
    "change_external_chat",
    "msg_audit_notify",
    "subscribe",
    "unsubscribe",
    "enter_agent",
    "location",
    "batch_job_result",
    "click",
    "view",
    "scancode_push",
    "scancode_waitmsg",
    "pic_sysphoto",
    "pic_photo_or_album",
    "pic_weixin",
    "location_select",
    "open_approval_change",
    "share_agent_change",
    "template_card_event",
    "sys_approval_change",
    "living_status_change",
    "msgaudit_notify",
];

#[async_trait]
impl EventService for WeComClient {
    async fn register_callback(&self, req: RegisterCallbackRequest) -> ImResult<EventSubscription> {
        let token = req.token.as_deref().unwrap_or("");
        let aes_key = req.aes_key.as_deref().unwrap_or("");

        // Try to create first; if already exists (errcode 301002), update instead
        let body = serde_json::json!({
            "url": req.callback_url,
            "token": token,
            "encodingaeskey": aes_key,
            "event": req.event_types,
        });

        let resp = self.post("/cgi-bin/callback/create", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: CallbackResp = serde_json::from_str(&text)?;

        let errcode = data.errcode.unwrap_or(0);
        if errcode == 301002 {
            // Callback already exists, update it
            let resp = self.post("/cgi-bin/callback/update", &body).await?;
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            let data: CallbackResp = serde_json::from_str(&text)?;
            check_errcode(data.errcode, data.errmsg, text)?;
        } else {
            check_errcode(Some(errcode), data.errmsg, text)?;
        }

        Ok(EventSubscription {
            id: "wecom-callback".to_string(),
            callback_url: Some(req.callback_url),
            event_types: req.event_types,
            status: EventSubscriptionStatus::Active,
        })
    }

    async fn list_subscriptions(&self) -> ImResult<Vec<EventSubscription>> {
        let resp = self.get("/cgi-bin/callback/get").await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: GetCallbackResp = serde_json::from_str(&text)?;

        let errcode = data.errcode.unwrap_or(0);
        if errcode == 301003 {
            // No callback configured
            return Ok(vec![]);
        }
        check_errcode(Some(errcode), data.errmsg, text)?;

        Ok(vec![EventSubscription {
            id: "wecom-callback".to_string(),
            callback_url: data.url,
            event_types: data.event,
            status: EventSubscriptionStatus::Active,
        }])
    }

    async fn delete_subscription(&self, _subscription_id: &str) -> ImResult<()> {
        let resp = self.get("/cgi-bin/callback/delete").await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: CallbackResp = serde_json::from_str(&text)?;
        check_errcode(data.errcode, data.errmsg, text)?;
        Ok(())
    }

    async fn list_event_types(&self) -> ImResult<Vec<String>> {
        Ok(WECOM_EVENT_TYPES.iter().map(|s| s.to_string()).collect())
    }

    async fn poll_events(&self) -> ImResult<Vec<ImEvent>> {
        Err(ImError::NotSupported {
            feature: "poll_events (WeCom uses callback-based event delivery)".into(),
            platform: "wecom".into(),
        })
    }
}
