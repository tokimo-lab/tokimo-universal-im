use crate::error::ImResult;
use crate::types::{AccessToken, Credentials};
use async_trait::async_trait;

/// Authentication operations.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Obtain an access token using the provided credentials.
    async fn get_access_token(&self, credentials: &Credentials) -> ImResult<AccessToken>;

    /// Refresh an existing token. Returns a new [`AccessToken`].
    async fn refresh_token(&self, refresh_token: &str) -> ImResult<AccessToken>;
}
