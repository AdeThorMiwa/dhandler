use anyhow::anyhow;
use async_trait::async_trait;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};

use crate::{services::directory::JobEntry, utils::llm_helper};

use super::types::JobRequirements;

#[async_trait]
pub trait RequirementsAnalyzer: Send + Sync {
    async fn analyze(&self, job: &JobEntry) -> anyhow::Result<JobRequirements>;
}

pub struct ClaudeRequirementsAnalyzer {
    client: anthropic::Client,
}

impl ClaudeRequirementsAnalyzer {
    pub fn new() -> Self {
        Self {
            client: anthropic::Client::from_env(),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"
You extract structured hiring requirements from a job posting.
Respond ONLY with a valid JSON object. No markdown. No code blocks. No explanation.

Required fields:
{
  "must_have":        [string],  // non-negotiable qualifications and hard skills
  "nice_to_have":     [string],  // preferred but not blocking
  "responsibilities": [string],  // core duties of the role
  "ats_keywords":     [string]   // important terms a resume should contain to pass screening
}

Be specific and concrete. Extract only what the posting explicitly states.
"#;

#[async_trait]
impl RequirementsAnalyzer for ClaudeRequirementsAnalyzer {
    async fn analyze(&self, job: &JobEntry) -> anyhow::Result<JobRequirements> {
        let agent = self
            .client
            .agent("claude-haiku-4-5-20251001")
            .preamble(SYSTEM_PROMPT)
            .max_tokens(1024)
            .temperature(0.0)
            .build();

        let raw = agent
            .prompt(&job.to_ai_readable_string())
            .await
            .map_err(|e| anyhow!("{e}"))?;

        serde_json::from_str(llm_helper::strip_code_block(raw.trim()))
            .map_err(|e| anyhow!("RequirementsAnalyzer: bad JSON: {e}\nRaw response: {raw}"))
    }
}
