use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{
    services::{auth::AuthService, user::UserService},
    utils,
    views::auth::{AuthenticatedUser, LoginResponse},
};

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

#[debug_handler]
async fn get_authenticated_user(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let service = utils::app::get::<UserService>(&ctx)?;
    let pid: Uuid = utils::app::get_pid(&auth)?;
    let user = service.get_user_by_id(&pid).await?;
    format::json(AuthenticatedUser::new(&user))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("auth")
        .add("with-google", post(login_with_google))
        .add("/me", get(get_authenticated_user))
}
