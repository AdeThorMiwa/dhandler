use di::{singleton_as_self, Injectable, ServiceCollection, ServiceProvider};
use loco_rs::config::Config;
use loco_rs::prelude::*;
use std::any::type_name;
use std::sync::Arc;

use crate::libs::google::auth::GoogleAuthClient;
use crate::services::auth::AuthService;
use crate::services::encryption::EncryptionService;
use crate::services::google_auth::GoogleAuthService;
use crate::services::knowledge_base::KnowledgeBaseService;
use crate::services::user::UserService;
use crate::utils::settings::Settings;

/// # Errors
/// Returns an error if the type `T` is not found in the shared store.
pub fn get<T: 'static>(ctx: &AppContext) -> Result<Arc<T>> {
    let provider = ctx.shared_store.get::<ServiceProvider>().ok_or_else(|| {
        tracing::error!("Type {} not found in shared store", type_name::<T>());
        Error::InternalServerError // Or a more specific error
    })?;

    Ok(provider.get_required::<T>())
}

/// Gets the PID from the JWT claims.
///
/// # Errors
/// Returns an error if the PID cannot be parsed.
pub fn get_pid(auth: &auth::JWT) -> Result<Uuid> {
    let pid: Uuid = auth.claims.pid.parse().map_err(|e| {
        tracing::error!("Failed to parse pid: {}", e);
        Error::InternalServerError
    })?;

    Ok(pid)
}

pub struct DIContext {
    pub db: Arc<DatabaseConnection>,
    pub config: Arc<Config>,
}

/// Creates a dependency injection provider.
///
/// # Panics
/// Panics if the provider cannot be built.
#[must_use]
pub fn create_di_provider(ctx: &DIContext) -> ServiceProvider {
    let db = ctx.db.clone();
    let config = ctx.config.clone();
    let settings =
        serde_json::from_value::<Settings>(ctx.config.settings.clone().unwrap()).unwrap();

    ServiceCollection::new()
        .add(singleton_as_self::<DatabaseConnection>().from(move |_| db.clone()))
        .add(singleton_as_self::<Config>().from(move |_| config.clone()))
        .add(singleton_as_self::<Settings>().from(move |_| Arc::new(settings.clone())))
        .add(GoogleAuthClient::singleton())
        .add(GoogleAuthService::singleton())
        .add(EncryptionService::singleton())
        .add(UserService::singleton())
        .add(AuthService::singleton())
        .add(KnowledgeBaseService::singleton())
        .build_provider()
        .unwrap()
}
