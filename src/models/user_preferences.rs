pub use super::_entities::user_preferences::{ActiveModel, Entity, Model};
use crate::models::_entities::{sea_orm_active_enums::Modality, user_preferences};
use loco_rs::{
    model::{ModelError, ModelResult},
    prelude::model,
};
use sea_orm::{entity::prelude::*, ActiveValue, TryIntoModel};
use serde::Deserialize;
use validator::Validate;
pub type UserPreferences = Entity;
pub type UserPreference = Model;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl UserPreference {}

// implement your write-oriented logic here
impl ActiveModel {
    /// Edit a user preference
    ///
    /// # Errors
    ///
    /// If db update fails
    pub async fn edit(
        mut self,
        db: &DatabaseConnection,
        payload: UpsertUserPreference,
    ) -> ModelResult<UserPreference> {
        self.directories = payload
            .directories
            .map(ActiveValue::Set)
            .unwrap_or(self.directories);

        self.job_search_at = payload
            .job_search_at
            .map(ActiveValue::Set)
            .unwrap_or(self.job_search_at);

        self.application_delay = payload
            .application_delay
            .map(ActiveValue::Set)
            .unwrap_or(self.application_delay);

        self.application_frequency_min = payload
            .application_frequency_min
            .map(ActiveValue::Set)
            .unwrap_or(self.application_frequency_min);

        self.application_frequency_max = payload
            .application_frequency_max
            .map(ActiveValue::Set)
            .unwrap_or(self.application_frequency_max);

        self.preferred_roles = payload
            .preferred_roles
            .map(ActiveValue::Set)
            .unwrap_or(self.preferred_roles);

        self.organization_blacklist = payload
            .organization_blacklist
            .map(ActiveValue::Set)
            .unwrap_or(self.organization_blacklist);

        self.minimum_salary = payload
            .minimum_salary
            .map(ActiveValue::Set)
            .unwrap_or(self.minimum_salary);

        self.preferred_modalities = payload
            .preferred_modalities
            .map(ActiveValue::Set)
            .unwrap_or(self.preferred_modalities);

        self.preferred_countries = payload
            .preferred_countries
            .map(|countries| ActiveValue::Set(Some(countries)))
            .unwrap_or(self.preferred_countries);

        Ok(self.save(db).await?.try_into_model()?)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// Create a new user preference
    ///
    /// # Errors
    ///
    /// If db insert fails
    pub async fn create(
        db: &DatabaseConnection,
        owner_id: i32,
        payload: UpsertUserPreference,
    ) -> ModelResult<UserPreference> {
        let pid = ActiveValue::set(Uuid::new_v4());
        let owner_id = ActiveValue::set(owner_id);
        let directories = ActiveValue::Set(payload.directories.unwrap_or_default());
        let job_search_at = ActiveValue::Set(payload.job_search_at.unwrap_or_default());
        let application_delay = ActiveValue::Set(payload.application_delay.unwrap_or(43_300)); // 12 hours in seconds
        let application_frequency_min =
            ActiveValue::Set(payload.application_frequency_min.unwrap_or_default());
        let application_frequency_max =
            ActiveValue::Set(payload.application_frequency_max.unwrap_or(5));
        let preferred_roles = ActiveValue::Set(payload.preferred_roles.unwrap_or_default());
        let organization_blacklist =
            ActiveValue::Set(payload.organization_blacklist.unwrap_or_default());
        let minimum_salary = ActiveValue::Set(payload.minimum_salary.unwrap_or_default());
        let preferred_modalities = ActiveValue::Set(
            payload
                .preferred_modalities
                .unwrap_or_else(|| vec![Modality::Onsite, Modality::Remote, Modality::Hybrid]),
        );
        let preferred_countries = ActiveValue::Set(payload.preferred_countries);

        let preference = user_preferences::ActiveModel {
            pid,
            directories,
            job_search_at,
            application_delay,
            application_frequency_min,
            application_frequency_max,
            preferred_roles,
            organization_blacklist,
            minimum_salary,
            preferred_modalities,
            preferred_countries,
            owner_id,
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(preference)
    }

    /// Find a user preference by owner id
    ///
    /// # Errors
    ///
    /// If no preference is found
    pub async fn find_by_owner_id(
        db: &DatabaseConnection,
        owner_id: i32,
    ) -> ModelResult<UserPreference> {
        let preference = Self::find()
            .filter(model::query::condition().eq(user_preferences::Column::OwnerId, owner_id))
            .one(db)
            .await?;

        preference.ok_or_else(|| ModelError::EntityNotFound)
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpsertUserPreference {
    pub directories: Option<Vec<String>>,
    pub job_search_at: Option<Time>,
    pub application_delay: Option<i32>,
    pub application_frequency_min: Option<i16>,
    pub application_frequency_max: Option<i16>,
    pub preferred_roles: Option<Vec<String>>,
    pub organization_blacklist: Option<Vec<String>>,
    pub minimum_salary: Option<i32>,
    pub preferred_modalities: Option<Vec<Modality>>,
    pub preferred_countries: Option<Vec<String>>,
}
