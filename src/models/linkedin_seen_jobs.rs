use std::collections::HashSet;

use crate::models::_entities::linkedin_seen_jobs;

pub use super::_entities::linkedin_seen_jobs::{ActiveModel, Entity, Model};
use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
pub type LinkedinSeenJobs = Entity;
pub type LinkedinSeenJob = Model;

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
    pub async fn find_by_ref_id_and_role(
        db: &DbConn,
        ref_id: &str,
        role: &str,
    ) -> ModelResult<HashSet<String>> {
        let rows = Self::find()
            .filter(linkedin_seen_jobs::Column::RefId.eq(ref_id))
            .filter(linkedin_seen_jobs::Column::Role.eq(role))
            .all(db)
            .await?;

        Ok(rows.into_iter().map(|r| r.linkedin_job_id).collect())
    }

    pub async fn mark_seen(
        db: &DbConn,
        ref_id: &str,
        role: &str,
        linkedin_job_id: &str,
    ) -> ModelResult<()> {
        let model = linkedin_seen_jobs::ActiveModel {
            ref_id: ActiveValue::Set(ref_id.to_string()),
            role: ActiveValue::Set(role.to_string()),
            linkedin_job_id: ActiveValue::Set(linkedin_job_id.to_string()),
            ..Default::default()
        };
        // on_conflict → ignore duplicates (race-safe)
        use sea_orm::{sea_query::OnConflict, ActiveValue};
        Self::insert(model)
            .on_conflict(
                OnConflict::columns([
                    linkedin_seen_jobs::Column::RefId,
                    linkedin_seen_jobs::Column::Role,
                    linkedin_seen_jobs::Column::LinkedinJobId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(db)
            .await?;
        Ok(())
    }
}
