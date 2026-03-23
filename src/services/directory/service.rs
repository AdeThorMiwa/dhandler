use di::injectable;
use loco_rs::prelude::*;
use std::sync::Arc;

use crate::{
    libs::browser::EnvironmentOrchestrator,
    services::directory::{ApplyRequest, ApplyResult, FetchJobRequest, JobDirectory, JobEntry},
};

pub struct JobDirectoryMetadata {
    pub id: String,
    pub name: String,
}

#[injectable]
pub struct JobDirectoryService {
    directories: Vec<Arc<dyn JobDirectory>>,
}

impl JobDirectoryService {
    pub async fn get_directories(&self) -> Result<Vec<JobDirectoryMetadata>> {
        Ok(self
            .directories
            .iter()
            .map(|dir| JobDirectoryMetadata {
                id: dir.id().to_string(),
                name: dir.name().to_string(),
            })
            .collect())
    }

    pub async fn fetch_jobs(
        &self,
        directory_id: &str,
        request: FetchJobRequest,
        orchestrator: &mut EnvironmentOrchestrator,
    ) -> Result<Vec<JobEntry>> {
        let Some(directory) = self.directories.iter().find(|dir| dir.id() == directory_id) else {
            return Err(Error::NotFound);
        };

        directory
            .fetch_jobs(request, orchestrator.get_browser())
            .await
    }

    pub async fn apply_to_job(
        &self,
        directory_id: &str,
        request: ApplyRequest,
        orchestrator: &mut EnvironmentOrchestrator,
    ) -> Result<ApplyResult> {
        let Some(directory) = self.directories.iter().find(|dir| dir.id() == directory_id) else {
            return Err(Error::NotFound);
        };

        directory
            .apply_to_job(request, orchestrator.get_browser())
            .await
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use uuid::Uuid;

    use crate::{
        libs::{agents::SimpleAgent, browser::EnvironmentOrchestrator},
        services::directory::{
            linkedin::LinkedinJobDirectory, Answer, ApplyRequest, ExperienceLevel, FetchJobFilters,
            FetchJobRequest, JobEntry, Modality, Money, Question, QuestionHandler, Salary,
        },
        utils::testing,
    };

    #[async_trait]
    impl QuestionHandler for SimpleAgent {
        async fn answer(&self, question: &Question) -> anyhow::Result<Answer> {
            println!("answering question: {question:?}");
            let question_json = serde_json::to_string(question).unwrap();
            let system_prompt = self.build_system_prompt(self.get_context());
            let answer = self.ask(&question_json, Some(&system_prompt)).await?;
            println!("answer to question: {answer:?}");
            let answer = match serde_json::from_str::<Answer>(&answer) {
                Ok(a) => {
                    if let Answer::Resume = &a {
                        use base64::{engine::general_purpose::STANDARD, Engine};
                        let mut path = std::env::current_dir().unwrap();
                        path.push("resume.pdf");
                        println!("path: {:?}", path);
                        let bytes = std::fs::read(path.clone())?;
                        let base64 = STANDARD.encode(&bytes);
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file")
                            .to_string();

                        Answer::FileUpload { filename, base64 }
                    } else {
                        a
                    }
                }
                Err(e) => {
                    println!("invalid answer: {e}");
                    Answer::Skip
                }
            };

            Ok(answer)
        }
    }

    fn create_question_handler(job: &JobEntry) -> Box<dyn QuestionHandler> {
        let context = format!(
            "Bio: \n {}\n\nAbout the job: \n{}\n\nGeneral information:\nIf job is remote, then i'm legally authorized to work in the country.",
            include_str!("ctx.test.md"),
            job.to_ai_readable_string()
        );
        Box::new(SimpleAgent::new(&context))
    }

    #[tokio::test]
    async fn test_get_directories() {
        let provider = testing::setup().await.expect("failed to setup provider");

        let service = provider.get_required::<super::JobDirectoryService>();
        let directories = service.get_directories().await;
        assert!(directories.is_ok());
        let directories = directories.unwrap();
        assert_eq!(directories.len(), 1);
    }

    #[tokio::test]
    async fn test_fetch_linkedin_jobs() {
        let provider = testing::setup().await.expect("failed to setup provider");

        let service = provider.get_required::<super::JobDirectoryService>();
        let mut orchestrator = EnvironmentOrchestrator::start()
            .await
            .expect("failed to start orchestrator");

        let filters = FetchJobFilters {
            role: "Software Engineer".to_string(),
            location: Some("Ireland".to_string()),
            modalities: Some(vec![Modality::Hybrid, Modality::Onsite]),
            ..Default::default()
        };

        let request = FetchJobRequest {
            ref_id: Uuid::new_v4(),
            limit: 1,
            filters,
        };

        let jobs = service
            .fetch_jobs(LinkedinJobDirectory::ID, request.clone(), &mut orchestrator)
            .await;

        assert!(jobs.is_ok());
        let jobs = jobs.unwrap();
        println!("jobs: {:#?}", jobs);
        assert_eq!(jobs.len(), request.limit);
    }

    #[tokio::test]
    async fn test_apply_to_job() {
        let provider = testing::setup().await.expect("failed to setup provider");

        let service = provider.get_required::<super::JobDirectoryService>();
        let mut orchestrator = EnvironmentOrchestrator::start()
            .await
            .expect("failed to start orchestrator");

        let job_entry = JobEntry {
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
            salary: Some(Salary {
                minimum: Money {
                    value: 500.0,
                    currency: "EUR".to_string(),
                },
                maximum: Some(Money {
                    value: 700.0,
                    currency: "EUR".to_string(),
                }),
            }),
            recruiter_name: "Cian Crosse".to_string(),
            modalities: vec![Modality::Remote],
            source: "linkedin".to_string(),
        };

        let request = ApplyRequest {
            job_id: job_entry.id.to_string(),
            question_handler: create_question_handler(&job_entry),
        };

        let result = service
            .apply_to_job(LinkedinJobDirectory::ID, request, &mut orchestrator)
            .await;

        assert!(result.is_ok());
    }
}
