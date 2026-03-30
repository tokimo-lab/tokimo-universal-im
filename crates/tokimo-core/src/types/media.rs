use serde::{Deserialize, Serialize};

/// Metadata about an uploaded or downloadable media resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    /// Platform-specific media key / ID.
    pub media_key: String,
    pub name: Option<String>,
    pub size: Option<u64>,
    pub mime_type: Option<String>,
    pub media_type: MediaType,
    /// Direct URL if available.
    pub url: Option<String>,
}

/// Type of media resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    File,
    Audio,
    Video,
}
