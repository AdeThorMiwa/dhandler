use crate::{
    libs::{markdown, multipart_parser::parse_multipart},
    services::knowledge_base::{AddKnowledgeBase, KnowledgeBaseService, KnowledgeBaseSource},
    utils,
    views::knowledge_base::KnowledgeBaseResponse,
};
use loco_rs::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Validate, Debug)]
pub struct UploadKnowledgeBaseRequest {
    #[validate(length(min = 2))]
    pub label: String,
}

#[derive(Deserialize, Validate, Debug)]
pub struct AddKnowledgeBaseRequest {
    #[validate(length(min = 2))]
    pub label: String,
    #[validate(length(min = 10))]
    pub content: String,
}

#[debug_handler]
async fn upload_knowledge_base(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    multipart: Multipart,
) -> Result<Response> {
    let service = utils::app::get::<KnowledgeBaseService>(&ctx)?;
    let pid: Uuid = utils::app::get_pid(&auth)?;
    let mut req = parse_multipart::<UploadKnowledgeBaseRequest>(multipart, vec!["content"]).await?;
    let content = markdown::read_from_stream(req.files.get_mut("content").unwrap()).await?;
    let payload = AddKnowledgeBase {
        owner_id: pid,
        label: req.body.label.clone(),
        content,
        source: KnowledgeBaseSource::Upload,
    };

    let knowledge_base = service.add_knowledge_base(payload).await?;
    format::json(KnowledgeBaseResponse::new(&knowledge_base))
}

#[debug_handler]
async fn add_knowledge_base(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    JsonValidate(payload): JsonValidate<AddKnowledgeBaseRequest>,
) -> Result<Response> {
    let service = utils::app::get::<KnowledgeBaseService>(&ctx)?;
    let pid: Uuid = utils::app::get_pid(&auth)?;
    let payload = AddKnowledgeBase {
        owner_id: pid,
        label: payload.label.clone(),
        content: payload.content.clone(),
        source: KnowledgeBaseSource::Upload,
    };

    let knowledge_base = service.add_knowledge_base(payload).await?;
    format::json(KnowledgeBaseResponse::new(&knowledge_base))
}

#[debug_handler]
async fn get_knowledge_base(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let service = utils::app::get::<KnowledgeBaseService>(&ctx)?;
    let pid: Uuid = utils::app::get_pid(&auth)?;
    let knowledge_base = service.get_user_knowledge_base_by_id(id, pid).await?;
    format::json(KnowledgeBaseResponse::new(&knowledge_base))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("knowledge-base")
        .add("/{id}", get(get_knowledge_base))
        .add("add", post(add_knowledge_base))
        .add("upload", post(upload_knowledge_base))
}
