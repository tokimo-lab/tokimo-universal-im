use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    EmailService, ImResult, ImError,
    Email, EmailAddress, EmailBody, Mailbox, Page,
    SendEmailRequest, ListEmailsRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

// ── Message types ──

#[derive(Deserialize)]
struct CreateMsgData {
    message_id: Option<String>,
}

#[derive(Deserialize)]
struct MsgListData {
    #[serde(default)]
    items: Vec<LarkEmail>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct MsgData {
    message: Option<LarkEmail>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkEmail {
    message_id: Option<String>,
    subject: Option<String>,
    from: Option<LarkEmailAddr>,
    #[serde(default)]
    to: Vec<LarkEmailAddr>,
    #[serde(default)]
    cc: Vec<LarkEmailAddr>,
    #[serde(default)]
    bcc: Vec<LarkEmailAddr>,
    body: Option<LarkEmailBody>,
    is_read: Option<bool>,
    date: Option<String>,
}

#[derive(Deserialize)]
struct LarkEmailAddr {
    mail_address: Option<String>,
    name: Option<String>,
}

impl From<LarkEmailAddr> for EmailAddress {
    fn from(a: LarkEmailAddr) -> Self {
        EmailAddress {
            address: a.mail_address.unwrap_or_default(),
            name: a.name,
        }
    }
}

#[derive(Deserialize)]
struct LarkEmailBody {
    content_type: Option<String>,
    content: Option<String>,
}

impl From<LarkEmail> for Email {
    fn from(e: LarkEmail) -> Self {
        Email {
            id: e.message_id.unwrap_or_default(),
            subject: e.subject.unwrap_or_default(),
            from: e.from.map(Into::into).unwrap_or(EmailAddress { address: String::new(), name: None }),
            to: e.to.into_iter().map(Into::into).collect(),
            cc: e.cc.into_iter().map(Into::into).collect(),
            bcc: e.bcc.into_iter().map(Into::into).collect(),
            body: e.body.map(|b| EmailBody {
                content_type: b.content_type.unwrap_or_else(|| "text/html".into()),
                content: b.content.unwrap_or_default(),
            }).unwrap_or(EmailBody { content_type: "text/plain".into(), content: String::new() }),
            is_read: e.is_read,
            date: e.date.as_deref().and_then(|s| {
                let ts: i64 = s.parse().ok()?;
                chrono::DateTime::from_timestamp(ts, 0)
            }),
            attachments: vec![],
        }
    }
}

// ── Folder types ──

#[derive(Deserialize)]
struct FolderListData {
    #[serde(default)]
    items: Vec<LarkFolder>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkFolder {
    folder_id: Option<String>,
    name: Option<String>,
    total_count: Option<u32>,
    unread_count: Option<u32>,
}

#[async_trait]
impl EmailService for LarkClient {
    async fn send_email(&self, req: SendEmailRequest) -> ImResult<Email> {
        // Step 1: Create draft
        let to_addrs: Vec<serde_json::Value> = req.to.iter().map(|a| {
            serde_json::json!({
                "mail_address": a.address,
                "name": a.name,
            })
        }).collect();
        let cc_addrs: Vec<serde_json::Value> = req.cc.iter().map(|a| {
            serde_json::json!({
                "mail_address": a.address,
                "name": a.name,
            })
        }).collect();
        let bcc_addrs: Vec<serde_json::Value> = req.bcc.iter().map(|a| {
            serde_json::json!({
                "mail_address": a.address,
                "name": a.name,
            })
        }).collect();
        let body = serde_json::json!({
            "subject": req.subject,
            "to": to_addrs,
            "cc": cc_addrs,
            "bcc": bcc_addrs,
            "body": {
                "content_type": req.body.content_type,
                "content": req.body.content,
            },
        });
        let resp = self.post("/open-apis/mail/v1/user_mailboxes/me/messages", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<CreateMsgData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let message_id = data.data.and_then(|d| d.message_id).unwrap_or_default();

        // Step 2: Send the draft
        let send_path = format!(
            "/open-apis/mail/v1/user_mailboxes/me/messages/{}/send",
            message_id,
        );
        let send_resp = self.post(&send_path, &serde_json::json!({})).await?;
        let send_text = send_resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let send_data: LarkResp<serde_json::Value> = serde_json::from_str(&send_text)?;
        if send_data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: send_data.code.unwrap_or(-1),
                message: send_data.msg.unwrap_or(send_text),
            });
        }

        Ok(Email {
            id: message_id,
            subject: req.subject,
            from: EmailAddress { address: String::new(), name: None },
            to: req.to,
            cc: req.cc,
            bcc: req.bcc,
            body: req.body,
            is_read: Some(true),
            date: Some(chrono::Utc::now()),
            attachments: vec![],
        })
    }

    async fn list_emails(&self, req: ListEmailsRequest) -> ImResult<Page<Email>> {
        let mut path = format!(
            "/open-apis/mail/v1/user_mailboxes/me/messages?page_size={}",
            req.limit.unwrap_or(20),
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<MsgListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(MsgListData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_email(&self, email_id: &str) -> ImResult<Email> {
        let path = format!("/open-apis/mail/v1/user_mailboxes/me/messages/{}", email_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<MsgData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let email = data.data.and_then(|d| d.message).ok_or_else(|| ImError::NotFound {
            resource: email_id.into(),
        })?;
        Ok(email.into())
    }

    async fn list_mailboxes(&self) -> ImResult<Vec<Mailbox>> {
        let resp = self.get("/open-apis/mail/v1/user_mailboxes/me/folders").await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<FolderListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(FolderListData { items: vec![] });
        Ok(list.items.into_iter().map(|f| Mailbox {
            id: f.folder_id.unwrap_or_default(),
            name: f.name.unwrap_or_default(),
            total_count: f.total_count,
            unread_count: f.unread_count,
        }).collect())
    }

    async fn mark_as_read(&self, email_id: &str) -> ImResult<()> {
        let body = serde_json::json!({ "is_read": true });
        let path = format!("/open-apis/mail/v1/user_mailboxes/me/messages/{}", email_id);
        let resp = self.patch(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<serde_json::Value> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(())
    }

    async fn delete_email(&self, email_id: &str) -> ImResult<()> {
        let path = format!("/open-apis/mail/v1/user_mailboxes/me/messages/{}", email_id);
        let resp = self.delete(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<serde_json::Value> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(())
    }
}
