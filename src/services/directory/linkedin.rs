use chromiumoxide::Browser;
use di::injectable;
use loco_rs::prelude::*;

use crate::services::directory::{FetchJobRequest, JobDirectory, JobEntry};

#[injectable(JobDirectory)]
pub struct LinkedinJobDirectory {}

impl LinkedinJobDirectory {
    pub const ID: &'static str = "linkedin";
}

#[async_trait]
impl JobDirectory for LinkedinJobDirectory {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn name(&self) -> &'static str {
        "LinkedIn"
    }

    async fn authenticate(&self, browser: &mut Browser) -> Result<()> {
        Ok(())
    }

    async fn fetch_jobs(
        &self,
        request: FetchJobRequest,
        browser: &mut Browser,
    ) -> Result<Vec<JobEntry>> {
        println!("hahahahah");
        Ok(Vec::new())
    }
}
