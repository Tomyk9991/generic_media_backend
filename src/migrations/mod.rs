use async_trait::async_trait;
use crate::database::database_context::DatabaseContext;
use crate::utils::version::Version;

pub mod user_migration;

#[async_trait]
pub trait DatabaseMigration {
    fn version(&self) -> &Version;
    async fn migrate(&self, context: &DatabaseContext) -> anyhow::Result<()>;
}