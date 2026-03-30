use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    MediaService, ImResult, ImError,
    MediaInfo, MediaType,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct UploadData {
    image_key: Option<String>,
}

#[async_trait]
impl MediaService for LarkClient {
    async fn upload_image(&self, data: Vec<u8>, filename: &str) -> ImResult<MediaInfo> {
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!("{}/open-apis/im/v1/images", self.base_url);
        let part = reqwest::multipart::Part::bytes(data)
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| ImError::Internal(e.to_string()))?;
        let form = reqwest::multipart::Form::new()
            .text("image_type", "message")
            .part("image", part);
        let resp = self.http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let result: LarkResp<UploadData> = serde_json::from_str(&text)?;
        if result.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: result.code.unwrap_or(-1),
                message: result.msg.unwrap_or(text),
            });
        }
        let key = result.data.and_then(|d| d.image_key).unwrap_or_default();
        Ok(MediaInfo {
            media_key: key,
            name: Some(filename.to_string()),
            size: None,
            mime_type: None,
            media_type: MediaType::Image,
            url: None,
        })
    }

    async fn upload_file(&self, data: Vec<u8>, filename: &str) -> ImResult<MediaInfo> {
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!("{}/open-apis/im/v1/files", self.base_url);
        let part = reqwest::multipart::Part::bytes(data)
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| ImError::Internal(e.to_string()))?;
        let form = reqwest::multipart::Form::new()
            .text("file_type", "stream")
            .text("file_name", filename.to_string())
            .part("file", part);
        let resp = self.http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let val: serde_json::Value = serde_json::from_str(&text)?;
        let code = val.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
        if code != 0 {
            return Err(ImError::Platform {
                code,
                message: val.get("msg").and_then(|v| v.as_str()).unwrap_or(&text).to_string(),
            });
        }
        let file_key = val.get("data").and_then(|d| d.get("file_key")).and_then(|v| v.as_str()).unwrap_or_default().to_string();
        Ok(MediaInfo {
            media_key: file_key,
            name: Some(filename.to_string()),
            size: None,
            mime_type: None,
            media_type: MediaType::File,
            url: None,
        })
    }

    async fn download_media(&self, media_key: &str, message_id: Option<&str>) -> ImResult<Vec<u8>> {
        let msg_id = message_id.ok_or_else(|| ImError::InvalidParam {
            message: "message_id is required for Lark media download".into(),
        })?;
        let path = format!("/open-apis/im/v1/messages/{}/resources/{}?type=file", msg_id, media_key);
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!("{}{}", self.base_url, path);
        let resp = self.http
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        resp.bytes().await
            .map(|b| b.to_vec())
            .map_err(|e| ImError::Network(e.to_string()))
    }
}
