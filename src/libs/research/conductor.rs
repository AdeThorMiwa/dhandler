use anyhow::anyhow;
use async_trait::async_trait;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};

use super::types::{ResearchFinding, ResearchVector};

#[async_trait]
pub trait Researcher: Send + Sync {
    async fn research(&self, vector: &ResearchVector) -> anyhow::Result<ResearchFinding>;
}

pub struct ClaudeResearcher {
    client: anthropic::Client,
}

impl ClaudeResearcher {
    pub fn new() -> Self {
        Self {
            client: anthropic::Client::from_env(),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"
You are a research assistant. Provide detailed, factual findings on the given query.

Structure your response as:
1. A concise summary paragraph (2–3 sentences)
2. Key findings as bullet points
3. Optionally, if you are drawing from specific named sources, list them at the end
   under a line that reads exactly "Sources:" — one per line

Be specific. Avoid filler. If you are uncertain, say so briefly rather than fabricating.
"#;

#[async_trait]
impl Researcher for ClaudeResearcher {
    async fn research(&self, vector: &ResearchVector) -> anyhow::Result<ResearchFinding> {
        let agent = self
            .client
            .agent("claude-haiku-4-5-20251001")
            .preamble(SYSTEM_PROMPT)
            .max_tokens(2048)
            .temperature(0.2)
            .build();

        let prompt = format!(
            "Focus area: {}\n\nResearch query: {}",
            vector.focus.label(),
            vector.query,
        );

        let raw = agent.prompt(&prompt).await.map_err(|e| anyhow!("{e}"))?;
        let (content, sources) = split_sources(&raw);

        Ok(ResearchFinding {
            vector: vector.clone(),
            content: content.to_string(),
            sources,
        })
    }
}

/// splits a research response into (content, sources).
/// looks for a "Sources:" section marker; everything after it is treated as one source per line.
fn split_sources(raw: &str) -> (&str, Vec<String>) {
    // Match "Sources:" at the start of a line (case-sensitive, as instructed in the prompt).
    if let Some(pos) = raw.find("\nSources:") {
        let content = raw[..pos].trim();
        let sources = raw[pos + "\nSources:".len()..]
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        (content, sources)
    } else {
        (raw.trim(), vec![])
    }
}
