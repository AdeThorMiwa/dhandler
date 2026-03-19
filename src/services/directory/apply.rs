// src/services/directory/apply.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Every piece of information about a form field that the handler
/// needs to produce a good answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// Raw label text as scraped from the page
    /// e.g. "How many years of experience do you have with Rust?"
    pub label: String,

    /// Lowercased, whitespace-normalised label — convenient for matching
    pub label_normalized: String,

    /// The kind of input this question expects
    pub kind: QuestionKind,

    /// Whether the field must be filled before advancing
    pub required: bool,

    /// For choice-based questions, the available options
    pub options: Vec<String>,

    /// Any placeholder / hint text on the input
    pub hint: Option<String>,

    /// The field's current pre-filled value, if any
    pub current_value: Option<String>,
}

impl Question {
    pub fn new(
        label: impl Into<String>,
        kind: QuestionKind,
        required: bool,
        options: Vec<String>,
        hint: Option<String>,
        current_value: Option<String>,
    ) -> Self {
        let label = label.into();
        let label_normalized = label
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        Self {
            label,
            label_normalized,
            kind,
            required,
            options,
            hint,
            current_value,
        }
    }
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
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum Answer {
    /// For Text, Number questions
    Text(String),
    /// For YesNo questions
    YesNo(bool),
    /// For SingleChoice, Dropdown — the chosen option verbatim
    SingleChoice(String),
    /// For MultiChoice — one or more chosen options
    MultiChoice(Vec<String>),
    /// For FileUpload — raw bytes + filename
    FileUpload { filename: String, bytes: Vec<u8> },
    /// Explicit signal that the handler intentionally has no answer —
    /// field will be left as-is (only valid for non-required fields)
    Skip,
}

#[async_trait]
pub trait QuestionHandler: Send + Sync {
    async fn answer(&self, question: &Question) -> anyhow::Result<Answer>;
}

/// Convenience type alias
pub type BoxedQuestionHandler = Box<dyn QuestionHandler>;
