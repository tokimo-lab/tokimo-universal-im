use serde::{Deserialize, Serialize};

/// A document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Platform-specific document ID.
    pub id: String,
    /// Document title / name.
    pub title: String,
    /// Document type.
    pub doc_type: DocumentType,
    /// Access URL.
    pub url: Option<String>,
    /// Document content (markdown or raw).
    pub content: Option<String>,
    /// Platform-specific extra data.
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// Type of document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Document,
    Spreadsheet,
    Smartsheet,
    Wiki,
    Other,
}

/// Request to create a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub doc_type: DocumentType,
    /// Initial content (markdown).
    pub content: Option<String>,
}

/// Request to update document content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub doc_id: String,
    /// New content (markdown). Replaces entire document.
    pub content: String,
}

/// Request to search documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocumentRequest {
    pub keyword: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
