use crate::models::_entities::google_auth_users;

pub use super::_entities::google_auth_users::{ActiveModel, Entity, Model};
use loco_rs::{
    model::{ModelError, ModelResult},
    prelude::model,
};
use sea_orm::{entity::prelude::*, ActiveValue};
pub type GoogleAuthUsers = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// finds a user by provided sub and refresh token
    ///
    /// # Errors
    ///
    /// When could not find user by the given token or DB insert error
    pub async fn create<C: ConnectionTrait>(
        db: &C,
        payload: CreateGoogleAuthUserPayload,
    ) -> ModelResult<Model> {
        let user = google_auth_users::ActiveModel {
            user_id: ActiveValue::set(payload.user_id),
            sub: ActiveValue::set(payload.sub),
            refresh_token: ActiveValue::set(payload.refresh_token),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(user)
    }

    /// finds a user by sub
    ///
    /// # Errors
    ///
    /// When could not find sub by the given sub or DB query error
    pub async fn find_by_sub(db: &DatabaseConnection, sub: &str) -> ModelResult<Model> {
        let user = Self::find()
            .filter(model::query::condition().eq(google_auth_users::Column::Sub, sub))
            .one(db)
            .await?;

        user.ok_or_else(|| ModelError::EntityNotFound)
    }
}

pub struct CreateGoogleAuthUserPayload {
    pub user_id: i32,
    pub sub: String,
    pub refresh_token: String,
}
