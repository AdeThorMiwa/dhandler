mod analyzer;
mod composer;
mod extractor;
mod tailor;
mod template;
mod types;

pub use analyzer::{ClaudeRequirementsAnalyzer, RequirementsAnalyzer};
pub use composer::{DefaultResumeComposer, ResumeComposer};
pub use extractor::{ClaudeProfileExtractor, ProfileExtractor};
pub use tailor::{ClaudeContentTailor, ContentTailor};
pub use template::{MarkdownTemplate, ResumeTemplate};
pub use types::{
    CandidateProfile, CertificationsSection, ContactInfo, Education, EducationSection,
    ExperienceSection, HeaderSection, JobRequirements, Project, ProjectsSection, Resume, Section,
    SkillsSection, SummarySection, TailoredContent, WorkExperience,
};

use std::marker::PhantomData;

use crate::services::directory::JobEntry;

pub struct Raw;
pub struct Ready;
pub struct Tailored;
pub struct Composed;

pub struct CvFlow<S> {
    pub job: JobEntry,
    pub knowledge_base: String,
    profile: Option<CandidateProfile>,
    requirements: Option<JobRequirements>,
    tailored: Option<TailoredContent>,
    resume: Option<Resume>,
    _state: PhantomData<S>,
}

impl CvFlow<Raw> {
    pub fn new(job: JobEntry, knowledge_base: &str) -> Self {
        Self {
            job,
            knowledge_base: knowledge_base.to_string(),
            profile: None,
            requirements: None,
            tailored: None,
            resume: None,
            _state: PhantomData,
        }
    }

    pub async fn prepare(
        self,
        extractor: &dyn ProfileExtractor,
        analyzer: &dyn RequirementsAnalyzer,
    ) -> anyhow::Result<CvFlow<Ready>> {
        let (profile, requirements) = tokio::try_join!(
            extractor.extract(&self.knowledge_base),
            analyzer.analyze(&self.job),
        )?;

        Ok(CvFlow {
            job: self.job,
            knowledge_base: self.knowledge_base,
            profile: Some(profile),
            requirements: Some(requirements),
            tailored: None,
            resume: None,
            _state: PhantomData,
        })
    }
}

impl CvFlow<Ready> {
    pub fn profile(&self) -> &CandidateProfile {
        self.profile
            .as_ref()
            .expect("profile always set after prepare()")
    }

    pub fn requirements(&self) -> &JobRequirements {
        self.requirements
            .as_ref()
            .expect("requirements always set after prepare()")
    }

    pub async fn tailor(self, tailor: &dyn ContentTailor) -> anyhow::Result<CvFlow<Tailored>> {
        let profile = self
            .profile
            .as_ref()
            .expect("profile always set after prepare()");
        let requirements = self
            .requirements
            .as_ref()
            .expect("requirements always set after prepare()");
        let tailored = tailor.tailor(profile, requirements, &self.job).await?;

        Ok(CvFlow {
            job: self.job,
            knowledge_base: self.knowledge_base,
            profile: self.profile,
            requirements: self.requirements,
            tailored: Some(tailored),
            resume: None,
            _state: PhantomData,
        })
    }
}

impl CvFlow<Tailored> {
    pub fn tailored(&self) -> &TailoredContent {
        self.tailored
            .as_ref()
            .expect("tailored always set after tailor()")
    }

    pub async fn compose(self, composer: &dyn ResumeComposer) -> anyhow::Result<CvFlow<Composed>> {
        let profile = self
            .profile
            .as_ref()
            .expect("profile always set after prepare()");
        let tailored = self
            .tailored
            .as_ref()
            .expect("tailored always set after tailor()");
        let resume = composer.compose(profile, tailored).await?;

        Ok(CvFlow {
            job: self.job,
            knowledge_base: self.knowledge_base,
            profile: self.profile,
            requirements: self.requirements,
            tailored: self.tailored,
            resume: Some(resume),
            _state: PhantomData,
        })
    }
}

impl CvFlow<Composed> {
    /// Extract the structured resume. Render it with any `ResumeTemplate`.
    pub fn into_resume(self) -> Resume {
        self.resume.expect("resume always set after compose()")
    }
}

#[cfg(test)]
mod tests {
    use crate::{libs::resume_generator::template::PdfTemplate, utils::testing};

    use super::*;

    #[tokio::test]
    async fn test_research_flow() {
        let _ = testing::setup().await;
        let job = testing::get_test_job();
        let knowledge_base = testing::get_test_aggregated_knowledge_base();

        let resume = CvFlow::new(job, knowledge_base)
            .prepare(
                &ClaudeProfileExtractor::new(),
                &ClaudeRequirementsAnalyzer::new(),
            )
            .await
            .inspect(|flow| println!("profile: {:?}\n\n", flow.profile()))
            .expect("prepare stage failed")
            .tailor(&ClaudeContentTailor::new())
            .await
            .inspect(|flow| println!("tailored: {:?}\n\n", flow.tailored()))
            .expect("tailor stage failed")
            .compose(&DefaultResumeComposer)
            .await
            .expect("compose stage failed")
            .into_resume();

        std::fs::write(
            "resume.test.md",
            MarkdownTemplate.render(&resume).expect("rendering failed"),
        )
        .expect("writing failed");

        std::fs::write(
            "resume.test.pdf",
            PdfTemplate.render(&resume).expect("rendering failed"),
        )
        .expect("writing failed");

        assert!(true)
    }
}
