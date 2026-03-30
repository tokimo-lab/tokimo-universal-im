use serde::{Deserialize, Serialize};

/// Identifies which IM platform a resource belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    DingTalk,
    WeCom,
    Lark,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::DingTalk => write!(f, "dingtalk"),
            Platform::WeCom => write!(f, "wecom"),
            Platform::Lark => write!(f, "lark"),
        }
    }
}

/// A cursor-based pagination wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// Common pagination request parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageRequest {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// A target for sending messages — either a user or a group chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum ChatTarget {
    /// Direct message to a user by user ID.
    User(String),
    /// Message to a group chat by chat/conversation ID.
    Group(String),
}

/// Authentication credentials that each platform provider requires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
}

/// An access token obtained from the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub refresh_token: Option<String>,
}
