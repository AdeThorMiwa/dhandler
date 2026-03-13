use crate::models::_entities::users;

pub use super::_entities::users::{ActiveModel, Entity, Model};
use loco_rs::{
    model::{ModelError, ModelResult},
    prelude::{model, Validatable},
};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::Deserialize;
use validator::Validate;

pub type Users = Entity;

#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(email(message = "invalid email"))]
    pub email: String,
    #[validate(length(min = 2, message = "Name must be at least 2 characters long."))]
    pub username: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            username: self.username.as_ref().to_owned(),
            email: self.email.as_ref().to_owned(),
        })
    }
}

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
    /// finds a user by provided email or create user if no user with email
    ///
    /// # Errors
    ///
    /// When could not find user by the given token or DB insert error
    pub async fn create<C: ConnectionTrait>(
        db: &C,
        payload: CreateUserPayload,
    ) -> ModelResult<Model> {
        let user = users::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            email: ActiveValue::set(payload.email.clone()),
            username: ActiveValue::set(payload.username.clone()),
            auth_provider: ActiveValue::set(payload.auth_provider.clone()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(user)
    }

    /// finds a user by email
    ///
    /// # Errors
    ///
    /// When could not find user by the given email or DB query error
    pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> ModelResult<Model> {
        let user = Self::find()
            .filter(model::query::condition().eq(users::Column::Email, email))
            .one(db)
            .await?;

        user.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// finds a user by the provided pid
    ///
    /// # Errors
    ///
    /// When could not find user by the given pid or DB query error
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &Uuid) -> ModelResult<Model> {
        let user = Self::find()
            .filter(model::query::condition().eq(users::Column::Pid, *pid))
            .one(db)
            .await?;

        user.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// finds a user by the provided db id
    ///
    /// # Errors
    ///
    /// When could not find user by the given db id or DB query error
    pub async fn find_by_db_id(db: &DatabaseConnection, user_id: i32) -> ModelResult<Model> {
        let user = Self::find_by_id(user_id).one(db).await?;

        user.ok_or_else(|| ModelError::EntityNotFound)
    }
}

pub struct CreateUserPayload {
    pub email: String,
    pub username: String,
    pub auth_provider: String,
}

impl CreateUserPayload {
    #[must_use]
    pub const fn new(email: String, username: String, auth_provider: String) -> Self {
        Self {
            email,
            username,
            auth_provider,
        }
    }
}

pub struct UserInfo {
    pub user_id: i32,
}
