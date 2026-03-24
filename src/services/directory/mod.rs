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

impl Money {
    pub fn new(value: f64, currency: String) -> Self {
        Self { value, currency }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Salary {
    minimum: Money,
    #[allow(unused)]
    maximum: Option<Money>,
}

impl Salary {
    pub fn new(minimum: Money, maximum: Option<Money>) -> Self {
        Self { minimum, maximum }
    }
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

impl JobEntry {
    pub fn to_ai_readable_string(&self) -> String {
        let mut parts = vec![];

        parts.push(format!("Job Title: {}", self.title));
        parts.push(format!("Company: {}", self.company));
        parts.push(format!("Location: {}", self.location));
        parts.push(format!("URL: {}", self.url));

        if let Some(seniority) = &self.seniority_level {
            parts.push(format!("Seniority Level: {:?}", seniority));
        }

        if let Some(industry) = &self.industry {
            parts.push(format!("Industry: {}", industry));
        }

        if let Some(posted_at) = &self.posted_at {
            parts.push(format!("Posted At: {}", posted_at));
        }

        if let Some(salary) = &self.salary {
            let min = &salary.minimum;
            match &salary.maximum {
                Some(max) => parts.push(format!(
                    "Salary: {}{} - {}{}",
                    min.currency, min.value, max.currency, max.value
                )),
                None => parts.push(format!("Salary: {}{}", min.currency, min.value)),
            }
        }

        if !self.modalities.is_empty() {
            let modalities = self
                .modalities
                .iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("Work Modality: {}", modalities));
        }

        if !self.recruiter_name.is_empty() {
            parts.push(format!("Recruiter: {}", self.recruiter_name));
        }

        parts.push(format!("Source: {}", self.source));

        parts.push(String::new()); // blank line before description
        parts.push(format!("Job Description:\n{}", self.description));

        parts.join("\n")
    }
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
    Resume,
    /// Explicit signal that the handler intentionally has no answer —
    /// field will be left as-is (only valid for non-required fields)
    Skip,
    /// Explicit signal that the handler intentionally has no answer —
    /// but can also not proceed because field is required
    MissingRequiredInfo,
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
            Answer::Resume => write!(f, "Resume"),
            Answer::Skip => write!(f, "Skip"),
            Answer::MissingRequiredInfo => write!(f, "MissingRequiredInfo"),
        }
    }
}

#[async_trait]
pub trait QuestionHandler: Send + Sync + Debug {
    fn build_system_prompt(&self, knowledge_bases: &str) -> String {
        format!(
            r#"
            You are a job application assistant. Your task is to answer job application questions on behalf of a candidate.

            You will be given:
            1. A JSON object representing a question from a job application form
            2. A knowledge base containing information about the candidate

            Your response must be a single valid JSON object. Nothing else. No markdown. No backticks. No code blocks. No explanation. No whitespace before or after the JSON object.

            The JSON must conform to this structure:
            - For text: {{"kind":"text","value":"..."}}
            - For number: {{"kind":"number","value":5}}
            - For yes/no: {{"kind":"yes_no","value":true}}
            - For single choice: {{"kind":"single_choice","value":"..."}}
            - For dropdown: {{"kind":"dropdown","value":"..."}}
            - For multi choice: {{"kind":"multi_choice","value":["...","..."]}}
            - For file upload: {{"kind":"resume"}}
            - For skip: {{"kind":"skip"}}
            - For missing info: {{"kind": "missing_required_info"}}

            CRITICAL RULES — violating any of these is a failure:

            1. OUTPUT FORMAT
               - Output ONLY the raw JSON object
               - Do NOT wrap in markdown code blocks
               - Do NOT use backticks
               - Do NOT add any text before or after the JSON
               - The first character of your response must be {{
               - The last character of your response must be }}

            2. OPTIONS VALIDATION
               - Each option in the question has a "text" and a "value" field
               - For dropdown and single_choice, your answer value MUST be copied EXACTLY from the "value" field of one of the provided options
               - NEVER invent a value that is not in the options list
               - NEVER use the "text" field as your answer, always use the "value" field
               - If you cannot find a matching option, use {{"kind":"skip"}} if field is not required and use {{"kind": "missing_required_info"}} if field is required

            3. CURRENT VALUE
                - If the question has a current_value that is already one of the valid options, prefer returning that value
                - Only override the current_value if you have a strong reason from the knowledge base

            4. FIELD RULES
                - "text" → answer with a string from the knowledge base
                - "number" → answer with an integer, never a string
                - "yes_no" → answer with true or false, never a string
                - "single_choice" → value must be from the options "value" field verbatim
                - "dropdown" → value must be from the options "value" field verbatim
                - "multi_choice" → values must all be from the options "value" field verbatim
                - "file_upload" → always return {{"kind":"resume"}}
                - If required is false and you are unsure, return {{"kind":"skip"}}
                - If required is true and you cannot answer confidently, make your best effort from the knowledge base

            5. NEVER fabricate information not present in the knowledge base
            6. NEVER return a dropdown or single_choice value that is not in the provided options list

            Knowledge base:
            {knowledge_bases}
        "#,
        )
    }

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
