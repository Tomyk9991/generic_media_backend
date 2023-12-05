use async_trait::async_trait;
use mongodb::bson::doc;
use crate::database::database_context::DatabaseContext;
use crate::migrations::DatabaseMigration;
use crate::utils::version::Version;

pub struct UserMigration {
    pub version: Version
}

#[async_trait]
impl DatabaseMigration for UserMigration {
    fn version(&self) -> &Version {
        &self.version
    }

    async fn migrate(&self, context: &DatabaseContext) -> anyhow::Result<()> {
        println!("Migration for version {:?}. Applying \"is_bot\"-field to user documents", self.version);
        let user_repo = context.user();
        let context = user_repo.get_context();

        let update = doc! { "$set": { "is_bot": false } };
        let options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        context.update_many(doc! { }, update, options).await?;

        Ok(())
    }
}