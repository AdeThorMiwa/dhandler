use serde::Serialize;

use crate::models::users;

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

impl LoginResponse {
    #[must_use]
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub username: String,
}

impl AuthenticatedUser {
    #[must_use]
    pub fn new(user: &users::Model) -> Self {
        Self {
            id: user.pid.to_string(),
            username: user.username.to_string(),
        }
    }
}
