use crate::utils::settings::Settings;
use di::injectable;
use loco_rs::prelude::*;
use reqwest;
use serde::Deserialize;
use std::sync::Arc;
use tracing::instrument;

#[derive(Deserialize, Debug, Default)]
pub struct ExchangeTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: String,
    pub expires_in: i64,
    pub refresh_token_expires_in: i64,
    pub scope: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

#[injectable]
pub struct GoogleAuthClient {
    settings: Arc<Settings>,
}

impl GoogleAuthClient {
    /// Exchanges an authorization code for an access token.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn exchange_code(&self, code: &str) -> Result<ExchangeTokenResponse> {
        let form_data = [
            ("code", code),
            ("client_id", &self.settings.google.client_id),
            ("client_secret", &self.settings.google.client_secret),
            ("redirect_uri", &self.settings.google.redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let response = reqwest::Client::new()
            .post("https://oauth2.googleapis.com/token")
            .form(&form_data)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to exchange code: {}", e);
                Error::InternalServerError
            })?;

        let token_response: ExchangeTokenResponse = response.json().await.map_err(|e| {
            tracing::error!("Deserialization failed: {}", e);
            Error::InternalServerError
        })?;

        Ok(token_response)
    }

    /// # Errors
    /// Returns an error if the request fails or the response cannot be deserialized.
    #[instrument(skip(self))]
    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        let user = reqwest::Client::new()
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Request failed: {}", e);
                Error::InternalServerError
            })?
            .json::<GoogleUserInfo>()
            .await
            .map_err(|e| {
                tracing::error!("Deserialization failed: {}", e);
                Error::InternalServerError
            })?;

        Ok(user)
    }
}
