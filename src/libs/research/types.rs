#[derive(Debug, Clone)]
pub enum EntityKind {
    Person {
        name: String,
        title: Option<String>,
    },
    Organization {
        name: String,
        domain: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ResolvedIdentity {
    pub kind: EntityKind,
    /// The most canonical unique identifier — a URL, domain, LinkedIn URL, or full name.
    pub signal: String,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub enum ResearchFocus {
    /// Technical analysis, codebase insights, product features.
    Technical,
    /// Public perception, press coverage, reviews, awards.
    Reputation,
    /// Direction, roadmap, stated priorities, key bets.
    Strategy,
    /// Values, team dynamics, leadership style, ways of working.
    Culture,
    /// News, hires, launches, pivots, or announcements from the past 12 months.
    RecentActivity,
    /// Funding, revenue model, burn rate, or business sustainability.
    Financials,
    /// Anything that doesn't fit the above — stored as a short free-form label.
    Custom(String),
}

impl ResearchFocus {
    pub fn from_label(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "reputation" => Self::Reputation,
            "strategy" => Self::Strategy,
            "culture" => Self::Culture,
            "recentactivity" | "recent_activity" | "recent activity" => Self::RecentActivity,
            "financials" | "financial" => Self::Financials,
            "tech" | "technical" => Self::Technical,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Reputation => "Reputation",
            Self::Strategy => "Strategy",
            Self::Culture => "Culture",
            Self::RecentActivity => "Recent Activity",
            Self::Financials => "Financials",
            Self::Technical => "Technical",
            Self::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResearchVector {
    pub id: String,
    /// The search query to execute.
    pub query: String,
    /// Why this gap matters for this specific candidate.
    pub rationale: String,
    pub focus: ResearchFocus,
}

#[derive(Debug, Clone)]
pub struct ResearchFinding {
    pub vector: ResearchVector,
    pub content: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Dossier {
    pub title: String,
    pub entity: ResolvedIdentity,
    /// Markdown-formatted research report.
    pub content: String,
    pub generated_at: String,
}
