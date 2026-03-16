use loco_rs::prelude::*;
use std::sync::Arc;

use crate::utils::app::{create_di_provider, DIContext};

pub struct DIInitializer;

#[async_trait]
impl Initializer for DIInitializer {
    fn name(&self) -> String {
        "Dependency Injection Initializer".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let context = DIContext {
            db: Arc::new(ctx.db.clone()),
            config: Arc::new(ctx.config.clone()),
        };

        let provider = create_di_provider(context);

        ctx.shared_store.insert(provider);
        Ok(())
    }
}
