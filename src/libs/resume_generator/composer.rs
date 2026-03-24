use async_trait::async_trait;

use super::types::{
    CandidateProfile, CertificationsSection, EducationSection, ExperienceSection, HeaderSection,
    ProjectsSection, Resume, Section, SkillsSection, SummarySection, TailoredContent,
};

#[async_trait]
pub trait ResumeComposer: Send + Sync {
    async fn compose(
        &self,
        profile: &CandidateProfile,
        tailored: &TailoredContent,
    ) -> anyhow::Result<Resume>;
}

pub struct DefaultResumeComposer;

#[async_trait]
impl ResumeComposer for DefaultResumeComposer {
    async fn compose(
        &self,
        profile: &CandidateProfile,
        tailored: &TailoredContent,
    ) -> anyhow::Result<Resume> {
        let mut sections = Vec::new();

        if let Some(name) = &profile.name {
            sections.push(Section::Header(HeaderSection {
                name: name.clone(),
                contact: profile.contact.clone(),
            }));
        }

        if !tailored.profile_summary.is_empty() {
            sections.push(Section::Summary(SummarySection {
                content: tailored.profile_summary.clone(),
            }));
        }

        if !tailored.experiences.is_empty() {
            sections.push(Section::Experience(ExperienceSection {
                entries: tailored.experiences.clone(),
            }));
        }

        if !tailored.education.is_empty() {
            sections.push(Section::Education(EducationSection {
                entries: tailored.education.clone(),
            }));
        }

        if !tailored.skills.is_empty() {
            sections.push(Section::Skills(SkillsSection {
                skills: tailored.skills.clone(),
            }));
        }

        if !tailored.projects.is_empty() {
            sections.push(Section::Projects(ProjectsSection {
                entries: tailored.projects.clone(),
            }));
        }

        if !tailored.certifications.is_empty() {
            sections.push(Section::Certifications(CertificationsSection {
                items: tailored.certifications.clone(),
            }));
        }

        Ok(Resume {
            candidate_name: profile.name.clone().unwrap_or_default(),
            sections,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
