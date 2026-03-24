use anyhow::anyhow;
use async_trait::async_trait;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic,
};

use crate::services::directory::JobEntry;

use super::types::{Dossier, ResearchFinding, ResolvedIdentity};

#[async_trait]
pub trait DossierGenerator: Send + Sync {
    async fn generate(
        &self,
        identity: &ResolvedIdentity,
        findings: &[ResearchFinding],
        knowledge_base: &str,
        job: &JobEntry,
    ) -> anyhow::Result<Dossier>;
}

pub struct ClaudeDossierGenerator {
    client: anthropic::Client,
}

impl ClaudeDossierGenerator {
    pub fn new() -> Self {
        Self {
            client: anthropic::Client::from_env(),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"
You write professional research dossiers in Markdown.

The dossier must follow this structure:
1. **Introduction** — who/what the target entity is and why they matter for this role
2. **Research Findings** — one section per theme, each finding explicitly connected
   to something in the candidate's background
3. **Talking Points** — 3–5 specific, concrete points the candidate can use
   in their application or interview

Be specific, professional, and actionable. Use Markdown headings, bullet points,
and bold text. Do not pad with generic advice.
"#;

#[async_trait]
impl DossierGenerator for ClaudeDossierGenerator {
    async fn generate(
        &self,
        identity: &ResolvedIdentity,
        findings: &[ResearchFinding],
        knowledge_base: &str,
        job: &JobEntry,
    ) -> anyhow::Result<Dossier> {
        let agent = self
            .client
            .agent("claude-sonnet-4-6")
            .preamble(SYSTEM_PROMPT)
            .max_tokens(4096)
            .temperature(0.4)
            .build();

        let findings_text = findings
            .iter()
            .map(|f| {
                format!(
                    "#### {} ({})\n{}\n",
                    f.vector.query,
                    f.vector.focus.label(),
                    f.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "TARGET: {}\nROLE: {} at {}\n\nCANDIDATE BACKGROUND:\n{}\n\nRESEARCH FINDINGS:\n{}\n\nGenerate the dossier.",
            identity.display_name,
            job.title,
            job.company,
            knowledge_base,
            findings_text,
        );

        let content = agent.prompt(&prompt).await.map_err(|e| anyhow!("{e}"))?;

        Ok(Dossier {
            title: format!(
                "Research Dossier: {} — {}",
                identity.display_name, job.title
            ),
            entity: identity.clone(),
            content,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
