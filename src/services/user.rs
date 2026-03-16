use std::sync::Arc;

use di::injectable;
use loco_rs::prelude::*;

use crate::{
    models::{
        google_auth_users::{CreateGoogleAuthUserPayload, GoogleAuthUsers},
        users::{CreateUserPayload, User, Users},
    },
    services::{encryption::EncryptionService, google_auth::GoogleUser},
};

#[injectable]
pub struct UserService {
    db: Arc<DatabaseConnection>,
    encryption: Arc<EncryptionService>,
}

impl UserService {
    /// # Errors
    /// Returns an error if some database operation fails.
    pub async fn get_or_create_user(&self, payload: GoogleUser) -> Result<User> {
        if let Ok(google_user) = GoogleAuthUsers::find_by_sub(&self.db, &payload.user.sub).await {
            return Ok(Users::find_by_db_id(&self.db, google_user.user_id).await?);
        }

        let tx = self.db.begin().await?;

        let create_user_payload = CreateUserPayload {
            email: payload.user.email.clone().unwrap_or_default(),
            username: payload.user.name.clone().unwrap_or_default(),
            auth_provider: "google_oauth2".to_string(),
        };

        let user = Users::create(&tx, create_user_payload).await?;

        let refresh_token = self
            .encryption
            .encrypt(&payload.exchange.refresh_token)
            .await?;

        let payload = CreateGoogleAuthUserPayload {
            user_id: user.id,
            sub: payload.user.sub.clone(),
            refresh_token: refresh_token,
        };

        GoogleAuthUsers::create(&tx, payload).await?;

        tx.commit().await?;

        Ok(user.into())
    }

    pub async fn get_user_by_id(&self, pid: &Uuid) -> Result<User> {
        let user = Users::find_by_pid(&self.db, pid).await?;
        Ok(user)
    }
}
