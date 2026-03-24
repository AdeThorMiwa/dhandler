use anyhow::anyhow;
use async_trait::async_trait;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};

use crate::utils::llm_helper;

use super::types::CandidateProfile;

#[async_trait]
pub trait ProfileExtractor: Send + Sync {
    async fn extract(&self, knowledge_base: &str) -> anyhow::Result<CandidateProfile>;
}

pub struct ClaudeProfileExtractor {
    client: anthropic::Client,
}

impl ClaudeProfileExtractor {
    pub fn new() -> Self {
        Self {
            client: anthropic::Client::from_env(),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"
You extract structured career data from a candidate's knowledge base.
Respond ONLY with a valid JSON object. No markdown. No code blocks. No explanation.

Required fields (use null or [] when data is absent — never omit a field):
{
  "name":    string | null,
  "contact": {
    "email":        string | null,
    "phone":        string | null,
    "linkedin_url": string | null,
    "github_url":   string | null,
    "website_url":  string | null,
    "other":        string | null   // anything not captured above, free-form
  },
  "summary": string | null,
  "experiences": [
    { "title": string, "company": string, "dates": string | null, "highlights": [string] }
  ],
  "education": [
    { "institution": string, "degree": string | null, "dates": string | null }
  ],
  "skills":          [string],
  "projects": [
    { "name": string, "description": string, "highlights": [string] }
  ],
  "certifications":  [string]
}

Extract only what is explicitly present. Do not invent or infer data.
"#;

#[async_trait]
impl ProfileExtractor for ClaudeProfileExtractor {
    async fn extract(&self, knowledge_base: &str) -> anyhow::Result<CandidateProfile> {
        let agent = self
            .client
            .agent("claude-haiku-4-5-20251001")
            .preamble(SYSTEM_PROMPT)
            .max_tokens(2048)
            .temperature(0.0)
            .build();

        let raw = agent
            .prompt(knowledge_base)
            .await
            .map_err(|e| anyhow!("{e}"))?;
        serde_json::from_str(llm_helper::strip_code_block(raw.trim()))
            .map_err(|e| anyhow!("ProfileExtractor: bad JSON: {e}\nRaw response: {raw}"))
    }
}
