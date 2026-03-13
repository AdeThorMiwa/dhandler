use serde::Serialize;

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
