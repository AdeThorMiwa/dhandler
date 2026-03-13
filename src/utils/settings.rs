use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Settings {
    pub google: GoogleSettings,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GoogleSettings {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}
