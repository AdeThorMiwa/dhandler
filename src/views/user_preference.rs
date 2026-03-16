use sea_orm::ActiveEnum;
use serde::Serialize;

use crate::models::user_preferences::UserPreference;

#[derive(Serialize)]
pub struct UserPreferenceResponse {
    pub id: String,
    pub directories: Vec<String>,
    pub job_search_at: String,
    pub application_delay: i32,
    pub application_frequency_min: i16,
    pub application_frequency_max: i16,
    pub preferred_roles: Vec<String>,
    pub organization_blacklist: Vec<String>,
    pub minimum_salary: i32,
    pub preferred_modalities: Vec<String>,
    pub preferred_countries: Option<Vec<String>>,
    pub last_updated: String,
}

impl UserPreferenceResponse {
    #[must_use]
    pub fn new(preference: &UserPreference) -> Self {
        Self {
            id: preference.pid.to_string(),
            directories: preference.directories.clone(),
            job_search_at: preference.job_search_at.to_string(),
            application_delay: preference.application_delay,
            application_frequency_min: preference.application_frequency_min,
            application_frequency_max: preference.application_frequency_max,
            preferred_roles: preference.preferred_roles.clone(),
            organization_blacklist: preference.organization_blacklist.clone(),
            minimum_salary: preference.minimum_salary,
            preferred_modalities: preference
                .preferred_modalities
                .iter()
                .map(ActiveEnum::to_value)
                .collect(),
            preferred_countries: preference.preferred_countries.clone(),
            last_updated: preference.updated_at.to_string(),
        }
    }
}
