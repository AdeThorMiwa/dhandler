use crate::models::_entities::knowledge_bases;

pub use super::_entities::knowledge_bases::{ActiveModel, Entity, Model};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::entity::prelude::*;
pub type KnowledgeBases = Entity;
pub type KnowledgeBase = Model;

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
impl KnowledgeBase {}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn update_content(
        mut self,
        db: &DatabaseConnection,
        new_content: String,
    ) -> ModelResult<KnowledgeBase> {
        self.content = sea_orm::ActiveValue::Set(new_content);
        Ok(self.update(db).await?)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    pub async fn create(
        db: &DatabaseConnection,
        payload: CreateKnowledgeBase,
    ) -> ModelResult<KnowledgeBase> {
        let knowledge_base = ActiveModel {
            owner_id: sea_orm::ActiveValue::Set(payload.owner_id),
            pid: sea_orm::ActiveValue::Set(Uuid::new_v4()),
            label: sea_orm::ActiveValue::Set(payload.label),
            content: sea_orm::ActiveValue::Set(payload.content),
            source: sea_orm::ActiveValue::Set(payload.source),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(knowledge_base)
    }

    pub async fn find_by_owner_id(
        db: &DatabaseConnection,
        owner_id: i32,
    ) -> ModelResult<Vec<KnowledgeBase>> {
        let knowledge_bases = Entity::find()
            .filter(knowledge_bases::Column::OwnerId.eq(owner_id))
            .all(db)
            .await?;

        Ok(knowledge_bases)
    }

    pub async fn find_by_pid_and_owner(
        db: &DatabaseConnection,
        pid: Uuid,
        owner_id: i32,
    ) -> ModelResult<KnowledgeBase> {
        let knowledge_bases = Entity::find()
            .filter(knowledge_bases::Column::Pid.eq(pid))
            .filter(knowledge_bases::Column::OwnerId.eq(owner_id))
            .one(db)
            .await?;

        knowledge_bases.ok_or_else(|| ModelError::EntityNotFound)
    }

    pub async fn find_by_source(
        db: &DatabaseConnection,
        owner_id: i32,
        source: &str,
    ) -> ModelResult<Option<KnowledgeBase>> {
        let knowledge_base = Entity::find()
            .filter(knowledge_bases::Column::OwnerId.eq(owner_id))
            .filter(knowledge_bases::Column::Source.eq(source))
            .one(db)
            .await?;

        Ok(knowledge_base)
    }
}

pub struct CreateKnowledgeBase {
    pub owner_id: i32,
    pub label: String,
    pub content: String,
    pub source: String,
}
