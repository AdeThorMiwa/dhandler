use anyhow::anyhow;
use async_trait::async_trait;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};

use crate::{services::directory::JobEntry, utils::llm_helper};

use super::types::{CandidateProfile, JobRequirements, TailoredContent};

#[async_trait]
pub trait ContentTailor: Send + Sync {
    async fn tailor(
        &self,
        profile: &CandidateProfile,
        requirements: &JobRequirements,
        job: &JobEntry,
    ) -> anyhow::Result<TailoredContent>;
}

pub struct ClaudeContentTailor {
    client: anthropic::Client,
}

impl ClaudeContentTailor {
    pub fn new() -> Self {
        Self {
            client: anthropic::Client::from_env(),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"
You tailor a candidate's profile to a specific job, producing a rewritten resume payload.
Respond ONLY with a valid JSON object. No markdown. No code blocks. No explanation.

Your output must follow this exact structure:
{
  "profile_summary": string,         // rewritten summary using the job's language and priorities
  "experiences": [
    { "title": string, "company": string, "dates": string | null, "highlights": [string] }
  ],                                 // reordered by relevance; bullet points rewritten to reflect the role
  "skills":          [string],       // ranked by relevance to this role, most relevant first
  "education": [
    { "institution": string, "degree": string | null, "dates": string | null }
  ],
  "projects": [
    { "name": string, "description": string, "highlights": [string] }
  ],
  "certifications":  [string],
  "coverage_gaps":   [string]        // requirements or responsibilities with no matching evidence
}

Rules:
- Only use information already present in the candidate profile — never fabricate
- Rewrite highlights in action-verb, impact-first style using the job's own terminology where appropriate
- Rank skills and experiences by relevance; drop entries that are entirely irrelevant
- Identify coverage gaps honestly — these help the candidate know what to address
"#;

#[async_trait]
impl ContentTailor for ClaudeContentTailor {
    async fn tailor(
        &self,
        profile: &CandidateProfile,
        requirements: &JobRequirements,
        job: &JobEntry,
    ) -> anyhow::Result<TailoredContent> {
        let agent = self
            .client
            .agent("claude-sonnet-4-6")
            .preamble(SYSTEM_PROMPT)
            .max_tokens(4096)
            .temperature(0.3)
            .build();

        let prompt = format!(
            "JOB:\n{}\n\nREQUIREMENTS:\n{}\n\nCANDIDATE PROFILE:\n{}",
            job.to_ai_readable_string(),
            serde_json::to_string_pretty(requirements)
                .unwrap_or_else(|_| format!("{requirements:?}")),
            serde_json::to_string_pretty(profile).unwrap_or_else(|_| format!("{profile:?}")),
        );

        let raw = agent.prompt(&prompt).await.map_err(|e| anyhow!("{e}"))?;
        serde_json::from_str(llm_helper::strip_code_block(raw.trim()))
            .map_err(|e| anyhow!("ContentTailor: bad JSON: {e}\nRaw response: {raw}"))
    }
}
