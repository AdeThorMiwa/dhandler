use crate::{
    services::directory::service::JobDirectoryService, utils,
    views::directories::JobDirectoryListResponse,
};
use axum::debug_handler;
use loco_rs::prelude::*;

#[debug_handler]
async fn get_all_directories(_auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let service = utils::app::get::<JobDirectoryService>(&ctx)?;
    let directories = service.get_all_directories().await?;
    format::json(JobDirectoryListResponse::new(directories))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("directory")
        .add("/", get(get_all_directories))
}
