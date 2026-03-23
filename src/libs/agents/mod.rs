use std::fmt::Debug;

use anyhow::anyhow;
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::anthropic::{self, Client},
};

pub struct SimpleAgent {
    client: Client,
    context: String,
}

impl Debug for SimpleAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleAgent").finish()
    }
}

impl SimpleAgent {
    pub fn new(context: &str) -> Self {
        let client = anthropic::Client::from_env();

        Self {
            client,
            context: context.to_string(),
        }
    }

    pub fn get_context(&self) -> &str {
        &self.context
    }

    pub async fn ask(&self, question: &str, system_prompt: Option<&str>) -> anyhow::Result<String> {
        let mut builder = self.client.agent("claude-haiku-4-5-20251001");
        if let Some(prompt) = system_prompt {
            builder = builder.preamble(prompt);
        }

        let agent = builder.max_tokens(1000).temperature(0.0).build();

        agent.prompt(question).await.map_err(|e| {
            println!("Error: {e}");
            anyhow!("failed")
        })
    }
}
