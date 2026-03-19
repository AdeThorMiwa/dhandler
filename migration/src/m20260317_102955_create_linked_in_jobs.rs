use loco_rs::schema::{create_table, drop_table, ColType};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "linkedin_seen_jobs",
            &[
                ("id", ColType::PkAuto),
                ("ref_id", ColType::String),
                ("role", ColType::String),
                ("linkedin_job_id", ColType::String),
            ],
            &[],
        )
        .await?;

        m.create_index(
            Index::create()
                .name("idx_linkedin_seen_jobs_unique")
                .table("linkedin_seen_jobs")
                .col("ref_id")
                .col("role")
                .col("linkedin_job_id")
                .unique()
                .to_owned(),
        )
        .await?;

        m.create_index(
            Index::create()
                .name("idx_linkedin_seen_jobs_ref_role")
                .table("linkedin_seen_jobs")
                .col("ref_id")
                .col("role")
                .unique()
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "linkedin_seen_jobs").await
    }
}
