use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    Document, Page,
    CreateDocumentRequest, UpdateDocumentRequest, SearchDocumentRequest,
};

/// Document operations (create, read, edit, search).
#[async_trait]
pub trait DocumentService: Send + Sync {
    /// Create a new document.
    async fn create_document(&self, req: CreateDocumentRequest) -> ImResult<Document>;

    /// Get document content by ID.
    async fn get_document(&self, doc_id: &str) -> ImResult<Document>;

    /// Update (overwrite) document content.
    async fn update_document(&self, req: UpdateDocumentRequest) -> ImResult<()>;

    /// Search documents by keyword.
    async fn search_documents(&self, req: SearchDocumentRequest) -> ImResult<Page<Document>>;
}
