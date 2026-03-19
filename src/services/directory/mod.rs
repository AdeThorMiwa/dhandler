use chromiumoxide::Browser;
use loco_rs::prelude::*;

pub mod linkedin;
pub mod service;

pub struct JobEntry {}

pub struct FetchJobRequest {
    ref_id: Uuid,
    role: String,
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
