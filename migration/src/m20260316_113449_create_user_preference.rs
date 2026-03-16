use loco_rs::schema::*;
use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    sea_orm::{ActiveEnum, DeriveActiveEnum, EnumIter, Schema},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "modality")]
pub enum Modality {
    #[sea_orm(string_value = "onsite")]
    Onsite,
    #[sea_orm(string_value = "remote")]
    Remote,
    #[sea_orm(string_value = "hybrid")]
    Hybrid,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let backend = m.get_database_backend();
        let schema = Schema::new(backend);

        m.create_type(schema.create_enum_from_active_enum::<Modality>())
            .await?;

        create_table(
            m,
            "user_preference",
            &[
                ("id", ColType::PkAuto),
                ("pid", ColType::UuidUniq),
                (
                    "directories",
                    ColType::Array(ColumnType::String(StringLen::N(20))),
                ),
                ("job_search_at", ColType::Time),
                ("application_delay", ColType::Integer),
                ("application_frequency_min", ColType::SmallInteger),
                ("application_frequency_max", ColType::SmallInteger),
                (
                    "preferred_roles",
                    ColType::Array(ColumnType::String(StringLen::N(50))),
                ),
                (
                    "organization_blacklist",
                    ColType::Array(ColumnType::String(StringLen::Max)),
                ),
                ("minimum_salary", ColType::Integer),
                (
                    "preferred_modalities",
                    ColType::Array(ColumnType::Enum {
                        name: Modality::name(),
                        variants: Modality::iden_values(),
                    }),
                ),
                (
                    "preferred_countries",
                    ColType::ArrayNull(ColumnType::String(StringLen::N(60))),
                ),
            ],
            &[("users", "owner_id")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "user_preference").await?;
        m.drop_type(Type::drop().name(Alias::new("modality")).to_owned())
            .await?;
        Ok(())
    }
}
