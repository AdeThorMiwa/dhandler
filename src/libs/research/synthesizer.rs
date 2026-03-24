use anyhow::anyhow;
use async_trait::async_trait;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};
use serde::Deserialize;

use crate::{services::directory::JobEntry, utils::llm_helper};

use super::types::{ResearchFocus, ResearchVector, ResolvedIdentity};

#[async_trait]
pub trait ContextualSynthesizer: Send + Sync {
    /// Performs gap analysis between the user's knowledge base and the job description,
    /// returning 3–5 ResearchVectors focused on the delta the candidate should close.
    async fn synthesize(
        &self,
        identity: &ResolvedIdentity,
        knowledge_base: &str,
        job: &JobEntry,
    ) -> anyhow::Result<Vec<ResearchVector>>;
}

// Raw shape Claude must return for each vector.
#[derive(Deserialize)]
struct RawVector {
    id: String,
    query: String,
    rationale: String,
    focus: String,
}

pub struct ClaudeContextualSynthesizer {
    client: anthropic::Client,
}

impl ClaudeContextualSynthesizer {
    pub fn new() -> Self {
        Self {
            client: anthropic::Client::from_env(),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"
You perform gap analysis between a candidate's knowledge base and a job description.
Respond ONLY with a valid JSON array. No markdown. No code blocks. No explanation.

Each element must have exactly these fields:
- id: "v1", "v2", etc.
- query: a specific, actionable research query (treat it as a web search string)
- rationale: one sentence — why this gap matters for this candidate given their background
- focus: exactly one of "Reputation", "Strategy", "Culture", "RecentActivity", "Financials",
         or a short custom label (3 words max) if none of those fit

Generate between 3 and 5 vectors. Prioritise the gaps that would most affect the candidate's fit,
preparation, or ability to speak credibly about the target entity.
"#;

#[async_trait]
impl ContextualSynthesizer for ClaudeContextualSynthesizer {
    async fn synthesize(
        &self,
        identity: &ResolvedIdentity,
        knowledge_base: &str,
        job: &JobEntry,
    ) -> anyhow::Result<Vec<ResearchVector>> {
        let agent = self
            .client
            .agent("claude-haiku-4-5-20251001")
            .preamble(SYSTEM_PROMPT)
            .max_tokens(1024)
            .temperature(0.3)
            .build();

        let prompt = format!(
            "TARGET ENTITY: {}\n\nCANDIDATE KNOWLEDGE BASE:\n{}\n\nJOB POSTING:\n{}",
            identity.display_name,
            knowledge_base,
            job.to_ai_readable_string(),
        );

        let raw = agent.prompt(&prompt).await.map_err(|e| anyhow!("{e}"))?;
        let items: Vec<RawVector> = serde_json::from_str(llm_helper::strip_code_block(raw.trim()))
            .map_err(|e| anyhow!("ContextualSynthesizer: bad JSON: {e}\nRaw response: {raw}"))?;

        Ok(items
            .into_iter()
            .map(|r| ResearchVector {
                id: r.id,
                query: r.query,
                rationale: r.rationale,
                focus: ResearchFocus::from_label(&r.focus),
            })
            .collect())
    }
}
