use crate::{
    models::{_entities::linkedin_creds::Column, linkedin_creds},
    services::directory::linkedin::LinkedInAuthCredentials,
};

pub use super::_entities::linkedin_creds::{ActiveModel, Entity, Model};
use loco_rs::model::ModelResult;
use migration::OnConflict;
use sea_orm::{entity::prelude::*, ActiveValue};
pub type LinkedinCreds = Entity;
pub type LinkedinCred = Model;

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
    pub async fn get_by_ref_id(db: &DbConn, ref_id: &str) -> ModelResult<LinkedinCred> {
        let cred = Self::find()
            .filter(Column::RefId.eq(ref_id))
            .one(db)
            .await?;
        cred.ok_or_else(|| loco_rs::model::ModelError::EntityNotFound)
    }

    pub async fn upsert(
        db: &DbConn,
        ref_id: &str,
        creds: LinkedInAuthCredentials,
    ) -> anyhow::Result<()> {
        let model = linkedin_creds::ActiveModel {
            ref_id: ActiveValue::Set(ref_id.to_string()),
            li_at: ActiveValue::Set(creds.li_at),
            j_session_id: ActiveValue::Set(creds.jsessionid),
            ..Default::default()
        };

        Self::insert(model)
            .on_conflict(
                OnConflict::column(Column::RefId)
                    .update_columns([Column::LiAt, Column::JSessionId])
                    .to_owned(),
            )
            .exec(db)
            .await?;

        Ok(())
    }
}
