use serde::Serialize;

use crate::models::knowledge_bases::KnowledgeBase;

#[derive(Serialize)]
pub struct KnowledgeBaseResponse {
    pub id: String,
    pub label: String,
    pub content: String,
    pub source: String,
    pub last_updated: String,
}

impl KnowledgeBaseResponse {
    #[must_use]
    pub fn new(knowledge_base: &KnowledgeBase) -> Self {
        Self {
            id: knowledge_base.pid.to_string(),
            label: knowledge_base.label.clone(),
            content: knowledge_base.content.clone(),
            source: knowledge_base.source.clone(),
            last_updated: knowledge_base.updated_at.to_string(),
        }
    }
}
