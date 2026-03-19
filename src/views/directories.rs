use serde::Serialize;

use crate::services::directory::JobDirectoryMetadata;

#[derive(Serialize)]
pub struct JobDirectoryListResponse {
    items: Vec<JobDirectoryMetadata>,
}

impl JobDirectoryListResponse {
    #[must_use]
    pub fn new(items: Vec<JobDirectoryMetadata>) -> Self {
        Self { items }
    }
}
