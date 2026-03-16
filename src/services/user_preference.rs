use std::sync::Arc;

use crate::models::user_preferences::{UpsertUserPreference, UserPreference, UserPreferences};
use di::injectable;
use loco_rs::prelude::*;

#[injectable]
pub struct UserPreferenceService {
    db: Arc<DatabaseConnection>,
}

impl UserPreferenceService {
    /// Get a user preference
    ///
    /// # Errors
    ///
    /// fail is no user preference is found or some db error
    pub async fn get_user_preference(&self, user_id: i32) -> Result<UserPreference> {
        match UserPreferences::find_by_owner_id(&self.db, user_id).await {
            Ok(preference) => Ok(preference),
            Err(ModelError::EntityNotFound) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    /// Upsert user preference
    ///
    /// # Errors
    ///
    /// fails if some db error occurs
    pub async fn upsert_user_preference(
        &self,
        user_id: i32,
        preference: UpsertUserPreference,
    ) -> Result<UserPreference> {
        match UserPreferences::find_by_owner_id(&self.db, user_id).await {
            Ok(existing) => {
                let editted = existing
                    .into_active_model()
                    .edit(&self.db, preference)
                    .await?;
                Ok(editted)
            }
            Err(ModelError::EntityNotFound) => {
                Ok(UserPreferences::create(&self.db, user_id, preference).await?)
            }
            Err(e) => Err(e.into()),
        }
    }
}
