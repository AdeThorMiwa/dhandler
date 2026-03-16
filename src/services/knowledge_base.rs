use std::sync::Arc;

use di::injectable;
use loco_rs::prelude::*;
use regex::Regex;
use uuid::Uuid;

use crate::models::knowledge_bases::{CreateKnowledgeBase, KnowledgeBase, KnowledgeBases};

#[injectable]
pub struct KnowledgeBaseService {
    db: Arc<DatabaseConnection>,
}

impl KnowledgeBaseService {
    /// Adds a knowledge base to the database.
    ///
    /// If a knowledge base with the same source already exists for the user,
    /// the content will be merged with the existing knowledge base.
    ///
    /// # Errors
    ///
    /// Returns an error if the knowledge base cannot be added.
    pub async fn add_knowledge_base(&self, payload: AddKnowledgeBase) -> Result<KnowledgeBase> {
        let source = Self::normalize_source(&payload.source);

        match KnowledgeBases::find_by_source(&self.db, payload.owner_id, &source).await {
            Ok(Some(existing)) => {
                let old_content = &existing.content;
                let new_content = Self::merge_content(old_content, &payload.content);

                let knowledge_base = existing
                    .into_active_model()
                    .update_content(&self.db, new_content)
                    .await?;

                Ok(knowledge_base)
            }
            Ok(None) => {
                let payload = CreateKnowledgeBase {
                    owner_id: payload.owner_id,
                    label: payload.label,
                    content: payload.content,
                    source,
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

    /// Gets a knowledge base by its ID and owner ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the knowledge base cannot be found.
    pub async fn get_user_knowledge_base_by_id(
        &self,
        id: Uuid,
        owner_id: i32,
    ) -> Result<KnowledgeBase> {
        Ok(KnowledgeBases::find_by_pid_and_owner(&self.db, id, owner_id).await?)
    }

    /// Gets an aggregated knowledge base for a given owner ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the knowledge base cannot be found.
    pub async fn get_aggregated_knowledge_base(&self, owner_id: i32) -> Result<String> {
        let aggregated_content = KnowledgeBases::find_by_owner_id(&self.db, owner_id)
            .await?
            .into_iter()
            .map(|kb| format!("Label:\n{}\n\nContent:\n{}", kb.label, kb.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(aggregated_content)
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

    /// Merges two knowledge base contents together.
    ///
    /// # Errors
    ///
    /// Returns an error if the merge fails.
    fn merge_content(old_content: &str, new_content: &str) -> String {
        // TODO: use a diffing algorithm here or LLM
        if old_content.is_empty() || old_content == new_content {
            return new_content.to_string();
        }

        format!("{old_content}\n\n{new_content}")
    }
}

pub enum KnowledgeBaseSource {
    Upload,
    Web(String),
}

pub struct AddKnowledgeBase {
    pub owner_id: i32,
    pub label: String,
    pub content: String,
    pub source: KnowledgeBaseSource,
}
