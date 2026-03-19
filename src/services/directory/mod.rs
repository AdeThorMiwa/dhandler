use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    models::_entities::sea_orm_active_enums::Modality,
    services::directory::apply::BoxedQuestionHandler,
};
pub mod apply;
pub mod currency;
pub mod linkedin;
pub mod money;
pub mod service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEntry {
    pub id: String,
    pub title: String,
    pub company: String,
    pub location: String,
    pub url: String,
    pub description: String,
    pub seniority_level: Option<String>,
    pub employment_type: Option<String>,
    pub industry: Option<String>,
    pub job_function: Option<String>,
    pub applicant_count: Option<String>,
    pub posted_at: Option<String>,
    pub salary_range: Option<String>,
    pub easy_apply: bool,
    pub recruiter_name: Option<String>,
    pub modality: Modality,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobDirectoryMetadata {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobSearchFilters {
    pub org_blacklist: Vec<String>,
    pub minimum_salary: Option<f64>,
    pub modalities: Option<Vec<Modality>>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchJobsRequest {
    pub role: String,
    pub ref_id: String,
    pub limit: usize,
    #[serde(default)]
    pub filters: JobSearchFilters,
}

pub struct ApplyRequest {
    pub ref_id: String,

    /// The handler invoked for every question in the Easy Apply modal.
    /// Typically wraps an LLM with full user context.
    pub question_handler: BoxedQuestionHandler,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ApplyResult {
    /// Application submitted successfully via Easy Apply
    Applied {
        job_id: String,
        applied_at: chrono::DateTime<chrono::Utc>,
    },

    /// Easy Apply flow started but hit a step that requires human judgment —
    /// e.g. an unrecognised question type, a CAPTCHA, or a multi-page form
    /// that couldn't be completed automatically.
    /// The browser session was left open long enough to screenshot the blocker.
    RequiresManualAction {
        job_id: String,
        reason: String,
        /// Base64-encoded PNG screenshot of the blocking step
        screenshot: Option<String>,
    },

    /// Job does not support Easy Apply — external application required.
    /// Not yet implemented; returned when easy_apply == false on the JobEntry.
    ExternalApplicationRequired {
        job_id: String,
        external_url: String,
    },
}

#[async_trait]
pub trait JobDirectory: Send + Sync + 'static {
    fn metadata(&self) -> JobDirectoryMetadata;
    async fn fetch_jobs(&self, request: FetchJobsRequest) -> anyhow::Result<Vec<JobEntry>>;
    async fn apply(&self, job: JobEntry, request: ApplyRequest) -> anyhow::Result<ApplyResult>;
}
