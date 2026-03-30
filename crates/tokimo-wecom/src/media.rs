use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MediaService, ImResult, ImError,
    MediaInfo,
};
use crate::client::WeComClient;

#[derive(Deserialize)]
struct MediaResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    media_item: Option<WcMedia>,
}

#[derive(Deserialize)]
struct WcMedia {
    media_id: Option<String>,
    name: Option<String>,
    #[serde(rename = "type")]
    media_type: Option<String>,
    local_path: Option<String>,
    size: Option<u64>,
    content_type: Option<String>,
}

#[async_trait]
impl MediaService for WeComClient {
    async fn upload_image(&self, _data: Vec<u8>, _filename: &str) -> ImResult<MediaInfo> {
        Err(ImError::NotSupported {
            feature: "upload_image (WeCom bot API doesn't support media upload)".into(),
            platform: "wecom".into(),
        })
    }

    async fn upload_file(&self, _data: Vec<u8>, _filename: &str) -> ImResult<MediaInfo> {
        Err(ImError::NotSupported {
            feature: "upload_file (WeCom bot API doesn't support media upload)".into(),
            platform: "wecom".into(),
        })
    }

    async fn download_media(&self, media_key: &str, _message_id: Option<&str>) -> ImResult<Vec<u8>> {
        let body = serde_json::json!({ "media_id": media_key });
        let resp = self.post("/cgi-bin/message/get_media", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: MediaResp = serde_json::from_str(&text)?;
        if data.errcode.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.errcode.unwrap_or(-1),
                message: data.errmsg.unwrap_or(text),
            });
        }
        // WeCom returns media info with local_path; actual binary download
        // would be done via the local_path or a separate download endpoint.
        // For the API abstraction, return the info as JSON bytes.
        let info = data.media_item.ok_or_else(|| ImError::NotFound {
            resource: format!("media {}", media_key),
        })?;
        // In production this would stream the file; here we return metadata
        Ok(serde_json::to_vec(&serde_json::json!({
            "media_id": info.media_id,
            "name": info.name,
            "type": info.media_type,
            "local_path": info.local_path,
            "size": info.size,
            "content_type": info.content_type,
        })).unwrap_or_default())
    }
}
