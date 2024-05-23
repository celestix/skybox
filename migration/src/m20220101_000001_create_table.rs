use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(File::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(File::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(File::Name).string().not_null())
                    .col(ColumnDef::new(File::Size).big_integer().not_null())
                    .col(ColumnDef::new(File::Created).big_integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum File {
    Table,
    Id,
    Name,
    Size,
    Created,
}
