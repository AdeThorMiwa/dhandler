use std::sync::Arc;

use di::injectable;
use loco_rs::prelude::*;
use regex::Regex;
use uuid::Uuid;

use crate::{
    models::knowledge_bases::{CreateKnowledgeBase, KnowledgeBase, KnowledgeBases},
    services::user::UserService,
};

#[injectable]
pub struct KnowledgeBaseService {
    user_service: Arc<UserService>,
    db: Arc<DatabaseConnection>,
}

impl KnowledgeBaseService {
    pub async fn add_knowledge_base(&self, payload: AddKnowledgeBase) -> Result<KnowledgeBase> {
        let user = self.user_service.get_user_by_id(&payload.owner_id).await?;
        let source = Self::normalize_source(&payload.source);

        match KnowledgeBases::find_by_source(&self.db, user.id, &source).await {
            Ok(Some(existing)) => {
                let old_content = &existing.content;
                let new_content = self.merge_content(old_content, &payload.content).await?;

                let knowledge_base = existing
                    .into_active_model()
                    .update_content(&self.db, new_content)
                    .await?;

                Ok(knowledge_base)
            }
            Ok(None) => {
                let payload = CreateKnowledgeBase {
                    owner_id: user.id,
                    label: payload.label,
                    content: payload.content,
                    source: source,
                };

                let knowledge_base = KnowledgeBases::create(&self.db, payload).await?;

                Ok(knowledge_base)
            }
            Err(e) => {
                tracing::error!("Find by source failed: {e}");
                Err(Error::InternalServerError)
            }
        }
    }

    fn normalize_source(source: &KnowledgeBaseSource) -> String {
        match source {
            KnowledgeBaseSource::Upload => "upload".to_string(),
            KnowledgeBaseSource::Web(url) => {
                // extract base url only and throw away everything else
                let r = Regex::new("^(https?://[^/]+)/.*").unwrap();
                let captures = r.captures(url).unwrap();
                captures[1].to_string()
            }
        }
    }

    async fn merge_content(&self, old_content: &str, new_content: &str) -> Result<String> {
        // TODO: use a diffing algorithm here or LLM
        if old_content.is_empty() || old_content == new_content {
            return Ok(new_content.to_string());
        }

        Ok(format!("{}\n\n{}", old_content, new_content))
    }
}

pub enum KnowledgeBaseSource {
    Upload,
    Web(String),
}

pub struct AddKnowledgeBase {
    pub owner_id: Uuid,
    pub label: String,
    pub content: String,
    pub source: KnowledgeBaseSource,
}
