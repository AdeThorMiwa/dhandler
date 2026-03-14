use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "knowledge_bases",
            &[
                ("id", ColType::PkAuto),
                ("pid", ColType::UuidUniq),
                ("label", ColType::String),
                ("content", ColType::Text),
                ("source", ColType::StringUniq),
            ],
            &[("users", "owner_id")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "knowledge_bases").await
    }
}
