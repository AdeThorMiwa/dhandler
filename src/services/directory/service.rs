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
        libs::{agents::simple::SimpleAgent, browser::EnvironmentOrchestrator},
        services::directory::{
            linkedin::LinkedinJobDirectory, Answer, ApplyRequest, FetchJobFilters, FetchJobRequest,
            JobEntry, Modality, Question, QuestionHandler,
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
            testing::get_test_aggregated_knowledge_base(),
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

        let job_entry = testing::get_test_job();

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
