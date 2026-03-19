use loco_rs::schema::{create_table, drop_table, ColType};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "linkedin_creds",
            &[
                ("id", ColType::PkAuto),
                ("ref_id", ColType::StringUniq),
                ("li_at", ColType::String),
                ("j_session_id", ColType::StringNull),
            ],
            &[],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "linkedin_creds").await
    }
}
