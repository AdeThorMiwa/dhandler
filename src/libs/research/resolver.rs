use anyhow::anyhow;
use async_trait::async_trait;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};
use serde::Deserialize;

use crate::services::directory::JobEntry;

use super::types::{EntityKind, ResolvedIdentity};

#[async_trait]
pub trait IdentityResolver: Send + Sync {
    async fn resolve(&self, job: &JobEntry) -> anyhow::Result<ResolvedIdentity>;
}

// Raw shape Claude must return — decoupled from the domain type.
#[derive(Deserialize)]
struct RawIdentity {
    kind: String,
    name: String,
    title: Option<String>,
    domain: Option<String>,
    signal: String,
    display_name: String,
}

pub struct ClaudeIdentityResolver {
    client: anthropic::Client,
}

impl ClaudeIdentityResolver {
    pub fn new() -> Self {
        Self {
            client: anthropic::Client::from_env(),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"
You resolve the hiring entity from a job listing into a structured identity.
Respond ONLY with a valid JSON object. No markdown. No code blocks. No explanation.

Required fields:
- kind: "Person" or "Organization"
- name: full name (person) or official organization name
- title: professional title if kind is "Person", otherwise null
- domain: primary web domain if kind is "Organization" (e.g. "acme.com"), otherwise null
- signal: the single most canonical unique identifier — prefer a URL or domain over a bare name
- display_name: short human-readable label used in prose
"#;

#[async_trait]
impl IdentityResolver for ClaudeIdentityResolver {
    async fn resolve(&self, job: &JobEntry) -> anyhow::Result<ResolvedIdentity> {
        let agent = self
            .client
            .agent("claude-haiku-4-5-20251001")
            .preamble(SYSTEM_PROMPT)
            .max_tokens(512)
            .temperature(0.0)
            .build();

        let prompt = format!(
            "Company/Entity: {}\nJob URL: {}\nJob Title: {}\nDescription (first 600 chars): {}",
            job.company,
            job.url,
            job.title,
            &job.description[..job.description.len().min(600)],
        );

        let raw = agent.prompt(&prompt).await.map_err(|e| anyhow!("{e}"))?;
        let parsed: RawIdentity = serde_json::from_str(strip_code_block(raw.trim()))
            .map_err(|e| anyhow!("IdentityResolver: bad JSON: {e}\nRaw response: {raw}"))?;

        let kind = match parsed.kind.as_str() {
            "Person" => EntityKind::Person {
                name: parsed.name,
                title: parsed.title,
            },
            _ => EntityKind::Organization {
                name: parsed.name,
                domain: parsed.domain,
            },
        };

        Ok(ResolvedIdentity {
            kind,
            signal: parsed.signal,
            display_name: parsed.display_name,
        })
    }
}

fn strip_code_block(s: &str) -> &str {
    if let Some(inner) = s
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
    {
        inner.trim()
    } else if let Some(inner) = s.strip_prefix("```").and_then(|s| s.strip_suffix("```")) {
        inner.trim()
    } else {
        s
    }
}
