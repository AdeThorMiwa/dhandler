use std::fmt::Debug;

use chromiumoxide::Browser;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

pub mod linkedin;
pub mod service;

// @todo we need to accept currency in preference so we can always convert money from job to the user's preferred currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    value: f64,
    #[allow(unused)]
    currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Salary {
    minimum: Money,
    #[allow(unused)]
    maximum: Option<Money>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modality {
    Remote,
    Onsite,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Internship,
    EntryLevel,
    Associate,
    MidLevel,
    SeniorLevel,
    Director,
    Executive,
    NotApplicable,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobEntry {
    pub id: String,
    pub title: String,
    pub company: String,
    pub location: String,
    pub url: String,
    pub description: String,
    pub seniority_level: Option<ExperienceLevel>,
    pub industry: Option<String>,
    pub posted_at: Option<String>,
    pub salary: Option<Salary>,
    pub recruiter_name: String,
    pub modalities: Vec<Modality>,
    pub source: String,
}

#[derive(Default, Debug, Clone)]
pub struct FetchJobFilters {
    role: String,
    location: Option<String>,
    modalities: Option<Vec<Modality>>,
    org_blacklist: Option<Vec<String>>,
    minimum_salary: Option<Money>,
    experience_level: Option<Vec<ExperienceLevel>>,
}

#[derive(Clone, Debug)]
pub struct FetchJobRequest {
    #[allow(unused)]
    ref_id: Uuid,
    limit: usize,
    filters: FetchJobFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    /// Free-form text input
    Text,
    /// Numeric input (years of experience, salary, etc.)
    Number,
    /// Single yes/no toggle or two-option radio
    YesNo,
    /// Radio buttons — pick exactly one from `options`
    SingleChoice,
    /// Checkboxes — pick one or more from `options`
    MultiChoice,
    /// <select> dropdown — pick exactly one from `options`
    Dropdown,
    /// File upload (resume, cover letter, portfolio)
    FileUpload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    text: String,
    value: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    /// Raw label text as scraped from the page
    /// e.g. "How many years of experience do you have with Rust?"
    pub label: String,

    /// The kind of input this question expects
    pub kind: QuestionKind,

    /// Whether the field must be filled before advancing
    pub required: bool,

    /// For choice-based questions, the available options
    pub options: Vec<QuestionOption>,

    /// Any placeholder / hint text on the input
    pub hint: Option<String>,

    /// The field's current pre-filled value, if any
    pub current_value: Option<String>,
}

impl Debug for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Question")
            .field("label", &self.label)
            .field("kind", &self.kind)
            .field("required", &self.required)
            .field("options", &format!("[{} options]", self.options.len()))
            .field("hint", &self.hint)
            .field("current_value", &self.current_value)
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum Answer {
    /// For Text, Number questions
    Text(String),
    Dropdown(String),
    /// For Number questions — the numeric value as a string
    Number(usize),
    /// For YesNo questions
    YesNo(bool),
    /// For SingleChoice, Dropdown — the chosen option verbatim
    SingleChoice(String),
    /// For MultiChoice — one or more chosen options
    MultiChoice(Vec<String>),
    /// For FileUpload — raw bytes + filename
    FileUpload {
        filename: String,
        base64: String, // base64 encoded, not raw bytes
    },
    /// Explicit signal that the handler intentionally has no answer —
    /// field will be left as-is (only valid for non-required fields)
    Skip,
}

impl std::fmt::Debug for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Answer::Text(s) => f.debug_tuple("Text").field(s).finish(),
            Answer::Dropdown(s) => f.debug_tuple("Dropdown").field(s).finish(),
            Answer::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Answer::YesNo(b) => f.debug_tuple("YesNo").field(b).finish(),
            Answer::SingleChoice(s) => f.debug_tuple("SingleChoice").field(s).finish(),
            Answer::MultiChoice(v) => f.debug_tuple("MultiChoice").field(v).finish(),
            Answer::FileUpload { filename, base64 } => f
                .debug_struct("FileUpload")
                .field("filename", filename)
                .field("base64", &format!("[{} bytes]", base64.len()))
                .finish(),
            Answer::Skip => write!(f, "Skip"),
        }
    }
}

#[async_trait]
pub trait QuestionHandler: Send + Sync + Debug {
    async fn answer(&self, question: &Question) -> anyhow::Result<Answer>;
}

#[derive(Debug)]
pub struct ApplyRequest {
    job_id: String,
    question_handler: Box<dyn QuestionHandler>,
}

pub enum ApplyResult {
    /// Application submitted successfully via Easy Apply
    Applied {
        job_id: String,
        applied_at: chrono::DateTime<chrono::Utc>,
    },

    /// There was a blocking step and the question handler requires manual action
    RequiresManualAction {
        job_id: String,
        reason: String,
        /// Base64-encoded PNG screenshot of the blocking step
        screenshot: Option<String>,
    },
}

#[async_trait]
pub trait JobDirectory {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    async fn authenticate(&self, browser: &mut Browser) -> Result<()>;
    async fn fetch_jobs(
        &self,
        request: FetchJobRequest,
        browser: &mut Browser,
    ) -> Result<Vec<JobEntry>>;
    async fn apply_to_job(
        &self,
        request: ApplyRequest,
        browser: &mut Browser,
    ) -> Result<ApplyResult>;
}
