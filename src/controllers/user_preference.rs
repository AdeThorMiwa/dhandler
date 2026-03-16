use crate::{
    models::user_preferences::UpsertUserPreference,
    services::user_preference::UserPreferenceService, utils,
    views::user_preference::UserPreferenceResponse,
};
use loco_rs::prelude::*;

type UpsertUserPreferenceRequest = UpsertUserPreference;

#[debug_handler]
async fn get_user_preference(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let service = utils::app::get::<UserPreferenceService>(&ctx)?;
    let user = utils::app::get_authenticated_user(&auth, &ctx).await?;
    let preference = service.get_user_preference(user.id).await?;
    format::json(UserPreferenceResponse::new(&preference))
}

#[debug_handler]
async fn upsert_user_preference(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    JsonValidate(req): JsonValidate<UpsertUserPreferenceRequest>,
) -> Result<Response> {
    let service = utils::app::get::<UserPreferenceService>(&ctx)?;
    let user = utils::app::get_authenticated_user(&auth, &ctx).await?;
    let preference = service.upsert_user_preference(user.id, req.into()).await?;
    format::json(UserPreferenceResponse::new(&preference))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("preference")
        .add("/", get(get_user_preference))
        .add("/", patch(upsert_user_preference))
}
