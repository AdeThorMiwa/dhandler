use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{services::auth::AuthService, utils, views::auth::LoginResponse};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginWithGoogleParams {
    pub code: String,
}

#[debug_handler]
async fn login_with_google(
    State(ctx): State<AppContext>,
    JsonValidate(params): JsonValidate<LoginWithGoogleParams>,
) -> Result<Response> {
    let service = utils::app::get::<AuthService>(&ctx)?;
    let token = service.authenticate_with_google(&params.code).await?;
    format::json(LoginResponse::new(&token))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("auth")
        .add("with-google", post(login_with_google))
}
