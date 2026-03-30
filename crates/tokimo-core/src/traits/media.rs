use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::MediaInfo;

/// Media upload / download operations.
///
/// Used to upload images/files for sending, and download media from
/// received messages.
#[async_trait]
pub trait MediaService: Send + Sync {
    /// Upload an image and get a media key for use in image messages.
    async fn upload_image(&self, data: Vec<u8>, filename: &str) -> ImResult<MediaInfo>;

    /// Upload a file and get a media key for use in file messages.
    async fn upload_file(&self, data: Vec<u8>, filename: &str) -> ImResult<MediaInfo>;

    /// Download media content by its key. Returns raw bytes.
    async fn download_media(&self, media_key: &str, message_id: Option<&str>) -> ImResult<Vec<u8>>;
}
