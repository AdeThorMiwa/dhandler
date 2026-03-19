use chromiumoxide::Browser;
use loco_rs::prelude::*;

pub mod linkedin;
pub mod service;

// @todo we need to accept currency in preference so we can always convert money from job to the user's preferred currency
#[derive(Debug, Clone)]
pub struct Money {
    value: f64,
    #[allow(unused)]
    currency: String,
}

#[derive(Debug)]
pub struct Salary {
    minimum: Money,
    #[allow(unused)]
    maximum: Option<Money>,
}

#[derive(Debug, Clone)]
pub enum Modality {
    Remote,
    Onsite,
    Hybrid,
}

#[derive(Debug, Clone)]
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

#[derive(Debug)]
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
}
