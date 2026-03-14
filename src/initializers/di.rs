use di::{singleton_as_self, Injectable, ServiceCollection};
use loco_rs::{config::Config, prelude::*};
use std::sync::Arc;

use crate::{
    libs::google::auth::GoogleAuthClient,
    services::{
        auth::AuthService, google_auth::GoogleAuthService, knowledge_base::KnowledgeBaseService,
        user::UserService,
    },
    utils::settings::Settings,
};

pub struct DIInitializer;

#[async_trait]
impl Initializer for DIInitializer {
    fn name(&self) -> String {
        "Dependency Injection Initializer".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let db = ctx.db.clone();
        let config = ctx.config.clone();
        let settings =
            serde_json::from_value::<Settings>(ctx.config.settings.clone().unwrap()).unwrap();

        let provider = ServiceCollection::new()
            .add(singleton_as_self::<DatabaseConnection>().from(move |_| Arc::new(db.clone())))
            .add(singleton_as_self::<Config>().from(move |_| Arc::new(config.clone())))
            .add(singleton_as_self::<Settings>().from(move |_| Arc::new(settings.clone())))
            .add(GoogleAuthClient::singleton())
            .add(GoogleAuthService::singleton())
            .add(UserService::singleton())
            .add(AuthService::singleton())
            .add(KnowledgeBaseService::singleton())
            .build_provider()
            .unwrap();

        ctx.shared_store.insert(provider);
        Ok(())
    }
}
