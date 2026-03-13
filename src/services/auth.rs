use di::injectable;
use loco_rs::{auth::jwt::JWT, config::Config, prelude::*};
use serde_json::Map;
use std::sync::Arc;

use crate::{
    models::users,
    services::{google_auth::GoogleAuthService, user::UserService},
};

#[injectable]
pub struct AuthService {
    user_service: Arc<UserService>,
    gauth: Arc<GoogleAuthService>,
    config: Arc<Config>,
}

impl AuthService {
    /// # Errors
    /// Returns an error if some database operation fails.
    pub async fn authenticate_with_google(&self, code: &str) -> Result<String> {
        let google_user = self.gauth.get_user_by_code(code).await?;
        self.gauth.check_scope_validity(&google_user.exchange)?;
        let user = self.user_service.get_or_create_user(google_user).await?;
        let token = self.authenticate(&user)?;
        Ok(token)
    }

    /// # Errors
    /// Returns an error if some database operation fails.
    pub fn authenticate(&self, user: &users::Model) -> Result<String> {
        let claims = Map::new();

        let jwt = self.config.get_jwt_config()?;

        let token = JWT::new(&jwt.secret)
            .generate_token(jwt.expiration, user.pid.to_string(), claims)
            .map_err(|_| Error::InternalServerError)?;

        Ok(token)
    }
}
