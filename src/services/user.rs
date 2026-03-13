use std::sync::Arc;

use di::injectable;
use loco_rs::prelude::*;

use crate::{
    libs::google::auth::GoogleUserInfo,
    models::{
        google_auth_users::{CreateGoogleAuthUserPayload, GoogleAuthUsers},
        users::{self, CreateUserPayload, Users},
    },
};

#[injectable]
pub struct UserService {
    db: Arc<DatabaseConnection>,
}

impl UserService {
    /// # Errors
    /// Returns an error if some database operation fails.
    pub async fn get_or_create_user(
        &self,
        user_info: &GoogleUserInfo,
        refresh_token: &str,
    ) -> Result<users::Model> {
        if let Ok(google_user) = GoogleAuthUsers::find_by_sub(&self.db, &user_info.sub).await {
            return Ok(Users::find_by_db_id(&self.db, google_user.user_id).await?);
        }

        let tx = self.db.begin().await?;

        let payload = CreateUserPayload {
            email: user_info.email.clone().unwrap_or_default(),
            username: user_info.name.clone().unwrap_or_default(),
            auth_provider: "google_oauth2".to_string(),
        };

        let user = Users::create(&tx, payload).await?;

        let payload = CreateGoogleAuthUserPayload {
            user_id: user.id,
            sub: user_info.sub.clone(),
            refresh_token: refresh_token.to_string(),
        };

        GoogleAuthUsers::create(&tx, payload).await?;

        tx.commit().await?;

        Ok(user)
    }
}
