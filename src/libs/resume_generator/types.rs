use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkExperience {
    pub title: String,
    pub company: String,
    pub dates: Option<String>,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub institution: String,
    pub degree: Option<String>,
    pub dates: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub email:        Option<String>,
    pub phone:        Option<String>,
    pub linkedin_url: Option<String>,
    pub github_url:   Option<String>,
    pub website_url:  Option<String>,
    /// Anything not captured by the fields above — free-form.
    pub other:        Option<String>,
}

impl ContactInfo {
    /// Returns true if every field is None.
    pub fn is_empty(&self) -> bool {
        self.email.is_none()
            && self.phone.is_none()
            && self.linkedin_url.is_none()
            && self.github_url.is_none()
            && self.website_url.is_none()
            && self.other.is_none()
    }

    /// Formats all present fields as a single ` · `-separated display string.
    pub fn display(&self) -> String {
        [
            self.email.as_deref(),
            self.phone.as_deref(),
            self.linkedin_url.as_deref(),
            self.github_url.as_deref(),
            self.website_url.as_deref(),
            self.other.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("  ·  ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateProfile {
    pub name: Option<String>,
    pub contact: Option<ContactInfo>,
    pub summary: Option<String>,
    pub experiences: Vec<WorkExperience>,
    pub education: Vec<Education>,
    pub skills: Vec<String>,
    pub projects: Vec<Project>,
    pub certifications: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequirements {
    pub must_have: Vec<String>,
    pub nice_to_have: Vec<String>,
    pub responsibilities: Vec<String>,
    /// Keywords to surface for ATS matching.
    pub ats_keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailoredContent {
    /// Rewritten summary aligned to the job's language and priorities.
    pub profile_summary: String,
    /// Experiences reordered by relevance; bullet points rewritten.
    pub experiences: Vec<WorkExperience>,
    /// Skills ranked by relevance to the role.
    pub skills: Vec<String>,
    pub education: Vec<Education>,
    pub projects: Vec<Project>,
    pub certifications: Vec<String>,
    /// Job requirements with no matching evidence in the candidate's profile.
    pub coverage_gaps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HeaderSection {
    pub name:    String,
    pub contact: Option<ContactInfo>,
}

#[derive(Debug, Clone)]
pub struct SummarySection {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ExperienceSection {
    pub entries: Vec<WorkExperience>,
}

#[derive(Debug, Clone)]
pub struct EducationSection {
    pub entries: Vec<Education>,
}

#[derive(Debug, Clone)]
pub struct SkillsSection {
    pub skills: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectsSection {
    pub entries: Vec<Project>,
}

#[derive(Debug, Clone)]
pub struct CertificationsSection {
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Section {
    Header(HeaderSection),
    Summary(SummarySection),
    Experience(ExperienceSection),
    Education(EducationSection),
    Skills(SkillsSection),
    Projects(ProjectsSection),
    Certifications(CertificationsSection),
}

/// Format-agnostic resume — render it with any `ResumeTemplate`.
#[derive(Debug, Clone)]
pub struct Resume {
    pub candidate_name: String,
    pub sections: Vec<Section>,
    pub generated_at: String,
}
