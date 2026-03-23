use di::ServiceProvider;
use dotenv::dotenv;
use loco_rs::environment::Environment;

use crate::utils::app::{create_di_provider, get_context_for_env};

pub async fn setup() -> loco_rs::Result<ServiceProvider> {
    dotenv().ok();
    let ctx = get_context_for_env(&Environment::Test).await?;
    Ok(create_di_provider(&ctx))
}
