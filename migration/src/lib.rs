#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20260313_072713_users;
mod m20260313_133713_google_auth_users;
mod m20260314_081957_knowledge_bases;
mod m20260316_113449_create_user_preference;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260313_072713_users::Migration),
            Box::new(m20260313_133713_google_auth_users::Migration),
            Box::new(m20260314_081957_knowledge_bases::Migration),
            Box::new(m20260316_113449_create_user_preference::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
