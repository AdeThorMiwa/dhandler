use crate::utils::app::create_di_provider;
use loco_rs::prelude::*;

pub struct DIInitializer;

#[async_trait]
impl Initializer for DIInitializer {
    fn name(&self) -> String {
        "Dependency Injection Initializer".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let provider = create_di_provider(ctx);
        ctx.shared_store.insert(provider);
        Ok(())
    }
}
