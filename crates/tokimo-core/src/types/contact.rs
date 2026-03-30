use serde::{Deserialize, Serialize};

/// A user in the IM platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Platform-specific user ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Email address (if available).
    pub email: Option<String>,
    /// Phone/mobile number (if available).
    pub phone: Option<String>,
    /// Avatar URL.
    pub avatar: Option<String>,
    /// Department IDs the user belongs to.
    #[serde(default)]
    pub departments: Vec<Department>,
    /// Platform-specific extra data.
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// A department / organizational unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

/// Parameters for searching users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchUserRequest {
    /// Keyword to search by (name, email, phone, etc.).
    pub keyword: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
