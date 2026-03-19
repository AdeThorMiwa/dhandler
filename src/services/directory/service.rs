use di::injectable;
use loco_rs::prelude::*;
use std::sync::Arc;

use crate::{
    libs::browser::EnvironmentOrchestrator,
    services::directory::{FetchJobRequest, JobDirectory, JobEntry},
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
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        libs::browser::EnvironmentOrchestrator,
        services::directory::{linkedin::LinkedinJobDirectory, FetchJobRequest},
        utils::testing,
    };

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

        let request = FetchJobRequest {
            ref_id: Uuid::new_v4(),
            role: "Software Engineer".to_string(),
        };

        let jobs = service
            .fetch_jobs(LinkedinJobDirectory::ID, request, &mut orchestrator)
            .await;

        assert!(jobs.is_ok());
        let jobs = jobs.unwrap();
        assert_eq!(jobs.len(), 0);
    }
}
