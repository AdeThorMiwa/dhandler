use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Settings {
    pub google: GoogleSettings,
    pub encryption: EncryptionSettings,
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
