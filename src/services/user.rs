use std::sync::Arc;

use di::injectable;
use loco_rs::prelude::*;

use crate::{
    models::{
        google_auth_users::{CreateGoogleAuthUserPayload, GoogleAuthUsers},
        users::{self, CreateUserPayload, Users},
    },
    services::google_auth::GoogleUser,
};

#[injectable]
pub struct UserService {
    db: Arc<DatabaseConnection>,
}

impl UserService {
    /// # Errors
    /// Returns an error if some database operation fails.
    pub async fn get_or_create_user(&self, payload: GoogleUser) -> Result<users::Model> {
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

        let payload = CreateGoogleAuthUserPayload {
            user_id: user.id,
            sub: payload.user.sub.clone(),
            refresh_token: payload.exchange.refresh_token.to_string(),
        };

        // @todo: encrypt refresh token
        GoogleAuthUsers::create(&tx, payload).await?;

        tx.commit().await?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, pid: &Uuid) -> Result<users::Model> {
        let user = Users::find_by_pid(&self.db, pid).await?;
        Ok(user)
    }
}
