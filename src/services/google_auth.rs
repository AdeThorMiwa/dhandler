use crate::libs::google::auth::{ExchangeTokenResponse, GoogleAuthClient, GoogleUserInfo};
use di::injectable;
use loco_rs::prelude::*;
use std::sync::Arc;

#[injectable]
pub struct GoogleAuthService {
    google_auth_client: Arc<GoogleAuthClient>,
}

const VALID_SCOPES: &[&str] = &[
    // Drive file access scope
    "https://www.googleapis.com/auth/drive.file",
    // Userinfo email scope
    "https://www.googleapis.com/auth/userinfo.email",
    // Userinfo profile scope
    "https://www.googleapis.com/auth/userinfo.profile",
    // OpenID scope
    "openid",
    // Gmail readonly scope
    "https://www.googleapis.com/auth/gmail.readonly",
];

pub struct GoogleUser {
    pub user: GoogleUserInfo,
    pub exchange: ExchangeTokenResponse,
}

impl GoogleAuthService {
    /// # Errors
    /// Returns an error if some database operation fails.
    pub async fn get_user_by_code(&self, code: &str) -> Result<GoogleUser> {
        let exchange = self.google_auth_client.exchange_code(code).await?;
        let user = self
            .google_auth_client
            .get_user_info(&exchange.access_token)
            .await?;

        Ok(GoogleUser { user, exchange })
    }

    pub fn check_scope_validity(&self, exchange: &ExchangeTokenResponse) -> Result<()> {
        let scopes = exchange.scope.split_whitespace().collect::<Vec<_>>();
        if !scopes.iter().all(|s| VALID_SCOPES.contains(s)) {
            return Err(Error::Unauthorized("Incomplete scope".to_owned()));
        }
        Ok(())
    }
}
