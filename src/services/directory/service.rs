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
        libs::browser::EnvironmentOrchestrator,
        services::directory::{
            linkedin::LinkedinJobDirectory, Answer, ApplyRequest, FetchJobFilters, FetchJobRequest,
            Modality, Question, QuestionHandler,
        },
        utils::testing,
    };

    fn create_question_handler() -> Box<dyn QuestionHandler> {
        #[derive(Debug)]
        struct Handler {}

        #[async_trait]
        impl QuestionHandler for Handler {
            async fn answer(&self, question: &Question) -> anyhow::Result<Answer> {
                let answer = if question.label == "Email address" {
                    Answer::Dropdown("adethormiwa1@gmail.com".to_string())
                } else if question.label == "Phone country code" {
                    let o = question
                        .options
                        .iter()
                        .find(|o| o.text.contains("234"))
                        .unwrap();
                    Answer::Dropdown(o.value.clone())
                } else if question.label == "Mobile phone number" {
                    Answer::Text("8000660000".to_string())
                } else if question.label.contains("Upload resume button") {
                    use base64::{engine::general_purpose::STANDARD, Engine};
                    let mut path = std::env::current_dir().unwrap();
                    path.push("resume.pdf");
                    println!("path: {:?}", path);
                    let bytes = std::fs::read(path.clone())?;
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file")
                        .to_string();

                    Answer::FileUpload {
                        filename,
                        base64: STANDARD.encode(&bytes),
                    }
                } else {
                    Answer::Skip
                };

                Ok(answer)
            }
        }

        Box::new(Handler {})
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

        let request = ApplyRequest {
            job_id: "4387910766".to_string(),
            question_handler: create_question_handler(),
        };

        let result = service
            .apply_to_job(LinkedinJobDirectory::ID, request, &mut orchestrator)
            .await;

        assert!(result.is_ok());
    }
}
