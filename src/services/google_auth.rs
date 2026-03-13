use crate::libs::google::auth::{GoogleAuthClient, GoogleUserInfo};
use di::injectable;
use loco_rs::prelude::*;
use std::sync::Arc;

#[injectable]
pub struct GoogleAuthService {
    google_auth_client: Arc<GoogleAuthClient>,
}

pub struct UserByCode {
    pub user: GoogleUserInfo,
    pub refresh_token: String,
}

impl GoogleAuthService {
    /// # Errors
    /// Returns an error if some database operation fails.
    pub async fn get_user_by_code(&self, code: &str) -> Result<UserByCode> {
        let exchange = self.google_auth_client.exchange_code(code).await?;
        let user = self
            .google_auth_client
            .get_user_info(&exchange.access_token)
            .await?;

        Ok(UserByCode {
            user,
            refresh_token: exchange.refresh_token.unwrap_or_default(),
        })
    }
}
