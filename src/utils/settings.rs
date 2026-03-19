use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Settings {
    pub google: GoogleSettings,
    pub encryption: EncryptionSettings,
    pub linkedin: LinkedInSettings,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GoogleSettings {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EncryptionSettings {
    pub key_path: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct LinkedInSettings {
    pub webdriver_url: String,
    pub li_at: Option<String>,
    pub jsessionid: Option<String>,
    pub pagination_size: usize,
    pub pagination_max_pages: usize,
}
