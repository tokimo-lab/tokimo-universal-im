use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{User, Page, SearchUserRequest};

/// Contact / address book operations.
#[async_trait]
pub trait ContactService: Send + Sync {
    /// Get the currently authenticated user's profile.
    async fn get_self(&self) -> ImResult<User>;

    /// Search users by keyword (name, email, phone …).
    async fn search_users(&self, req: SearchUserRequest) -> ImResult<Page<User>>;

    /// Get detailed info for a list of user IDs.
    async fn get_users(&self, user_ids: &[String]) -> ImResult<Vec<User>>;
}
