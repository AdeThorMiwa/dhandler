use crate::services::directory::{JobDirectory, JobDirectoryMetadata};
use di::injectable;
use loco_rs::prelude::*;
use std::sync::Arc;

#[injectable]
pub struct JobDirectoryService {
    directories: Vec<Arc<dyn JobDirectory>>,
}

impl JobDirectoryService {
    pub async fn get_all_directories(&self) -> Result<Vec<JobDirectoryMetadata>> {
        Ok(self.directories.iter().map(|d| d.metadata()).collect())
    }
}
