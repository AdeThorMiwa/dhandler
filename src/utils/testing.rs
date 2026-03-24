use di::ServiceProvider;
use dotenv::dotenv;
use loco_rs::environment::Environment;

use crate::{
    services::directory::{ExperienceLevel, JobEntry, Modality, Money, Salary},
    utils::app::{create_di_provider, get_context_for_env},
};

pub async fn setup() -> loco_rs::Result<ServiceProvider> {
    dotenv().ok();
    let ctx = get_context_for_env(&Environment::Test).await?;
    Ok(create_di_provider(&ctx))
}

pub fn get_test_job() -> JobEntry {
    JobEntry {
        id: "4389085055".to_string(),
        title: "Enterprise Architect - Agentic AI".to_string(),
        company: "Methodius IT Recruitment".to_string(),
        location: "Dublin, County Dublin, Ireland".to_string(),
        url: "https://www.linkedin.com/jobs/view/4389085055/".to_string(),
        description: "Barden is partnering with a global tech company in their search for an Agentic AI Enterprise Architect. This is a great opportunity for someone to help define the architecture for next-generation intelligent systems.



        ABOUT THE ROLE:

        Define the end-to-end architecture for the organisation’s agentic platform: infrastructure, patterns, and contracts enabling autonomous capabilities.
        Design agent orchestration, multi-agent coordination, graduated autonomy models, and guardrails for regulated environments.
        Establish observability, evaluation, and trust frameworks to ensure safe, auditable, and explainable agent behaviour.
        Architect a unified API and platform layer for agentic consumption, including tool exposure, context management, versioning, and multi-tenancy.
        Define cross-product composability patterns, resource/tool taxonomy, and integration standards for autonomous agents.
        Provide architectural guidance for ML/statistical models powering analytics products.
        Define model lifecycle, integration with agentic workflows, and alignment with platform engineering standards.
        Advise on build vs. buy decisions and ensure scalable, maintainable ML infrastructure.


        ABOUT YOU:

        10+ years of software architecture/engineering experience, with 2+ years in AI/ML production systems.
        Hands-on experience designing agentic or AI-orchestrated systems with multi-step reasoning, tool use, and autonomous workflows.
        Proven track record defining API strategies for complex, multi-product platforms.
        Production experience with LLM-based systems, orchestration frameworks, evaluation, guardrails, and cost optimisation.


        OTHER DETAILS:

        Our client is based in Cork city centre. For talent outside Cork, occasional travel to the office (2-3 times per month) is required.
        Permanent role: competitive salary plus benefits.


        PLEASE ONLY APPLY IF YOU ARE BASED IN IRELAND AND CAN TRAVEL TO CORK 1-3 TIMES PER MONTH".to_string(),
        seniority_level: Some(ExperienceLevel::SeniorLevel),
        industry: Some("Staffing and Recruiting".to_string()),
        posted_at: Some("2 weeks ago".to_string()),
        salary: Some(Salary::new(Money::new(500.0, "EUR".to_string()), Some(Money::new(700.0, "EUR".to_string())))),
        recruiter_name: "Cian Crosse".to_string(),
        modalities: vec![Modality::Remote],
        source: "linkedin".to_string(),
    }
}

pub fn get_test_aggregated_knowledge_base() -> &'static str {
    include_str!("../../ctx.test.md")
}
