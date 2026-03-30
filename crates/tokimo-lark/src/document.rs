use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    DocumentService, ImResult, ImError,
    Document, DocumentType, Page,
    CreateDocumentRequest, UpdateDocumentRequest, SearchDocumentRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct CreateDocData {
    document: Option<DocInfo>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DocInfo {
    document_id: Option<String>,
    title: Option<String>,
}

#[derive(Deserialize)]
struct GetDocData {
    document: Option<DocDetail>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct DocDetail {
    document_id: Option<String>,
    title: Option<String>,
}

#[derive(Deserialize)]
struct SearchData {
    #[serde(default)]
    items: Vec<SearchItem>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct SearchItem {
    doc_token: Option<String>,
    title: Option<String>,
    url: Option<String>,
    doc_type: Option<String>,
}

#[async_trait]
impl DocumentService for LarkClient {
    async fn create_document(&self, req: CreateDocumentRequest) -> ImResult<Document> {
        let body = serde_json::json!({
            "title": req.title,
            "folder_token": "",
        });
        let resp = self.post("/open-apis/docx/v1/documents", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<CreateDocData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let doc = data.data.and_then(|d| d.document).unwrap_or(DocInfo { document_id: None, title: None });
        let doc_id = doc.document_id.unwrap_or_default();
        // Write initial content if provided
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
            url: None,
            content: req.content,
            extra: serde_json::Value::Null,
        })
    }

    async fn get_document(&self, doc_id: &str) -> ImResult<Document> {
        let path = format!("/open-apis/docx/v1/documents/{}", doc_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<GetDocData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let doc = data.data.and_then(|d| d.document).ok_or_else(|| ImError::NotFound {
            resource: doc_id.into(),
        })?;
        // Fetch raw content blocks
        let content_path = format!("/open-apis/docx/v1/documents/{}/raw_content", doc_id);
        let content_resp = self.get(&content_path).await?;
        let content_text = content_resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let content_data: LarkResp<serde_json::Value> = serde_json::from_str(&content_text)?;
        let content = content_data.data.and_then(|d| d.get("content").and_then(|v| v.as_str()).map(String::from));

        Ok(Document {
            id: doc_id.to_string(),
            title: doc.title.unwrap_or_default(),
            doc_type: DocumentType::Document,
            url: None,
            content,
            extra: serde_json::Value::Null,
        })
    }

    async fn update_document(&self, req: UpdateDocumentRequest) -> ImResult<()> {
        // Lark uses block-based editing; for simplicity, create a text block
        let body = serde_json::json!({
            "requests": [{
                "create_block": {
                    "block": {
                        "block_type": 2, // text block
                        "text": { "elements": [{ "text_run": { "content": req.content } }] }
                    },
                    "index": 0,
                }
            }]
        });
        let path = format!("/open-apis/docx/v1/documents/{}/blocks/batch_update", req.doc_id);
        let resp = self.post(&path, &body).await?;
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

    async fn search_documents(&self, req: SearchDocumentRequest) -> ImResult<Page<Document>> {
        let body = serde_json::json!({
            "search_key": req.keyword,
            "count": req.limit.unwrap_or(20),
            "offset": req.cursor.as_deref().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0),
        });
        let resp = self.post("/open-apis/suite/docs-api/search/object", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SearchData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let search = data.data.unwrap_or(SearchData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: search.items.into_iter().map(|item| Document {
                id: item.doc_token.unwrap_or_default(),
                title: item.title.unwrap_or_default(),
                doc_type: match item.doc_type.as_deref() {
                    Some("doc") => DocumentType::Document,
                    Some("sheet") => DocumentType::Spreadsheet,
                    Some("wiki") => DocumentType::Wiki,
                    _ => DocumentType::Other,
                },
                url: item.url,
                content: None,
                extra: serde_json::Value::Null,
            }).collect(),
            has_more: search.has_more.unwrap_or(false),
            next_cursor: search.page_token,
        })
    }
}
