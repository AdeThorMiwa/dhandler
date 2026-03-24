mod conductor;
mod dossier;
mod resolver;
mod synthesizer;
mod types;

pub use conductor::{ClaudeResearcher, Researcher};
pub use dossier::{ClaudeDossierGenerator, DossierGenerator};
pub use resolver::{ClaudeIdentityResolver, IdentityResolver};
pub use synthesizer::{ClaudeContextualSynthesizer, ContextualSynthesizer};
pub use types::{
    Dossier, EntityKind, ResearchFinding, ResearchFocus, ResearchVector, ResolvedIdentity,
};

use std::marker::PhantomData;

use crate::services::directory::JobEntry;

// State markers — zero-cost phantom types that enforce pipeline ordering at
// compile time. Calling `synthesize` on an `Unresolved` flow is a type error.
pub struct Unresolved;
pub struct Identified;
pub struct Vectorized;
pub struct Researched;
pub struct Complete;

pub struct ResearchFlow<S> {
    pub job: JobEntry,
    pub knowledge_base: String,
    identity: Option<ResolvedIdentity>,
    vectors: Option<Vec<ResearchVector>>,
    findings: Option<Vec<ResearchFinding>>,
    dossier: Option<Dossier>,
    _state: PhantomData<S>,
}

impl ResearchFlow<Unresolved> {
    pub fn new(job: JobEntry, knowledge_base: &str) -> Self {
        Self {
            job,
            knowledge_base: knowledge_base.to_string(),
            identity: None,
            vectors: None,
            findings: None,
            dossier: None,
            _state: PhantomData,
        }
    }

    pub async fn resolve(
        self,
        resolver: &dyn IdentityResolver,
    ) -> anyhow::Result<ResearchFlow<Identified>> {
        let identity = resolver.resolve(&self.job).await?;
        Ok(ResearchFlow {
            job: self.job,
            knowledge_base: self.knowledge_base,
            identity: Some(identity),
            vectors: None,
            findings: None,
            dossier: None,
            _state: PhantomData,
        })
    }
}

impl ResearchFlow<Identified> {
    pub fn identity(&self) -> &ResolvedIdentity {
        self.identity.as_ref().expect("identity should be set")
    }

    pub async fn synthesize(
        self,
        synthesizer: &dyn ContextualSynthesizer,
    ) -> anyhow::Result<ResearchFlow<Vectorized>> {
        let identity = self.identity();
        let vectors = synthesizer
            .synthesize(identity, &self.knowledge_base, &self.job)
            .await?;
        Ok(ResearchFlow {
            job: self.job,
            knowledge_base: self.knowledge_base,
            identity: self.identity,
            vectors: Some(vectors),
            findings: None,
            dossier: None,
            _state: PhantomData,
        })
    }
}

impl ResearchFlow<Vectorized> {
    pub fn vectors(&self) -> &[ResearchVector] {
        self.vectors.as_deref().expect("vectors should be set")
    }

    pub async fn conduct(
        self,
        researcher: &dyn Researcher,
    ) -> anyhow::Result<ResearchFlow<Researched>> {
        // Destructure to gain owned access to each field independently.
        let ResearchFlow {
            job,
            knowledge_base,
            identity,
            vectors,
            ..
        } = self;

        let vectors = vectors.expect("vectors always set after Stage 2");
        let mut findings = Vec::with_capacity(vectors.len());
        for vector in &vectors {
            findings.push(researcher.research(vector).await?);
        }

        Ok(ResearchFlow {
            job,
            knowledge_base,
            identity,
            vectors: Some(vectors),
            findings: Some(findings),
            dossier: None,
            _state: PhantomData,
        })
    }
}

impl ResearchFlow<Researched> {
    pub fn findings(&self) -> &[ResearchFinding] {
        self.findings.as_deref().expect("findings should be set")
    }

    pub async fn generate(
        self,
        generator: &dyn DossierGenerator,
    ) -> anyhow::Result<ResearchFlow<Complete>> {
        let identity = self.identity.as_ref().expect("identity should be set");
        let findings = self.findings.as_ref().expect("findings should be set");
        let dossier = generator
            .generate(identity, findings, &self.knowledge_base, &self.job)
            .await?;
        Ok(ResearchFlow {
            job: self.job,
            knowledge_base: self.knowledge_base,
            identity: self.identity,
            vectors: self.vectors,
            findings: self.findings,
            dossier: Some(dossier),
            _state: PhantomData,
        })
    }
}

impl ResearchFlow<Complete> {
    pub fn into_dossier(self) -> Dossier {
        self.dossier.expect("dossier always set after Stage 4")
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::testing;

    use super::*;

    #[tokio::test]
    async fn test_research_flow() {
        let _ = testing::setup().await;
        let job = testing::get_test_job();
        let knowledge_base = testing::get_test_aggregated_knowledge_base();

        let dossier = ResearchFlow::new(job, knowledge_base)
            .resolve(&ClaudeIdentityResolver::new())
            .await
            .inspect(|flow| println!("id: {:?}\n\n", flow.identity()))
            .expect("identity resolver failed")
            .synthesize(&ClaudeContextualSynthesizer::new())
            .await
            .inspect(|flow| println!("vectors: {:?}\n\n", flow.vectors()))
            .expect("synthesizer failed")
            .conduct(&ClaudeResearcher::new())
            .await
            .inspect(|flow| println!("findings: {:?}\n\n", flow.findings()))
            .expect("researcher failed")
            .generate(&ClaudeDossierGenerator::new())
            .await
            .expect("dossier generator failed")
            .into_dossier();

        println!("dosier: {dossier:?}");
        assert!(true)
    }
}
