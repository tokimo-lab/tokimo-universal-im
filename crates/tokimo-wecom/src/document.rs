use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    DocumentService, ImResult, ImError,
    Document, DocumentType, Page,
    CreateDocumentRequest, UpdateDocumentRequest, SearchDocumentRequest,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct CreateDocResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    url: Option<String>,
    docid: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GetDocResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    task_id: Option<String>,
    task_done: Option<bool>,
    content: Option<String>,
}

#[async_trait]
impl DocumentService for WeComClient {
    async fn create_document(&self, req: CreateDocumentRequest) -> ImResult<Document> {
        let doc_type_int = match req.doc_type {
            DocumentType::Document => 3,
            DocumentType::Spreadsheet | DocumentType::Smartsheet => 10,
            _ => 3,
        };
        let body = serde_json::json!({
            "doc_type": doc_type_int,
            "doc_name": req.title,
        });
        let resp = self.post("/cgi-bin/doc/create", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: CreateDocResp = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.errcode.unwrap_or(-1),
                message: data.errmsg.unwrap_or(text),
            });
        }
        let doc_id = data.docid.unwrap_or_default();
        // If initial content is provided, write it
        if let Some(ref content) = req.content {
            let _ = self.update_document(UpdateDocumentRequest {
                doc_id: doc_id.clone(),
                content: content.clone(),
            }).await;
        }
        Ok(Document {
            id: doc_id,
            title: req.title,
            doc_type: req.doc_type,
            url: data.url,
            content: req.content,
            extra: serde_json::Value::Null,
        })
    }

    async fn get_document(&self, doc_id: &str) -> ImResult<Document> {
        // WeCom document retrieval is async; first request gets task_id, then poll
        let body = serde_json::json!({ "docid": doc_id, "type": 2 });
        let resp = self.post("/cgi-bin/doc/get_content", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let mut data: GetDocResp = serde_json::from_str(&text)?;

        // Poll until task completes (max 10 attempts)
        if let Some(task_id) = data.task_id.clone() {
            for _ in 0..10 {
                if data.task_done.unwrap_or(false) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let poll_body = serde_json::json!({ "docid": doc_id, "type": 2, "task_id": task_id });
                let poll_resp = self.post("/cgi-bin/doc/get_content", &poll_body).await?;
                let poll_text = poll_resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
                data = serde_json::from_str(&poll_text)?;
            }
        }

        Ok(Document {
            id: doc_id.to_string(),
            title: String::new(),
            doc_type: DocumentType::Document,
            url: None,
            content: data.content,
            extra: serde_json::Value::Null,
        })
    }

    async fn update_document(&self, req: UpdateDocumentRequest) -> ImResult<()> {
        let body = serde_json::json!({
            "docid": req.doc_id,
            "content": req.content,
            "content_type": 1, // markdown
        });
        let resp = self.post("/cgi-bin/doc/edit_content", &body).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        Ok(())
    }

    async fn search_documents(&self, _req: SearchDocumentRequest) -> ImResult<Page<Document>> {
        Err(ImError::NotSupported {
            feature: "search_documents".into(),
            platform: "wecom".into(),
        })
    }
}
