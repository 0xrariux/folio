use super::Store;
use crate::modules::{Action, Registry};
use anyhow::{Result, ensure};
impl Store {
    pub async fn module_registry(&self) -> Result<Registry> {
        let mut rows = self
            .connection
            .query("SELECT value FROM preferences WHERE key='modules'", ())
            .await?;
        let registry: Registry = rows
            .next()
            .await?
            .map(|r| r.get::<String>(0))
            .transpose()?
            .map(|s| serde_json::from_str(&s))
            .transpose()?
            .unwrap_or_default();
        registry.validate()?;
        Ok(registry)
    }
    pub async fn module_action(&self, action: Action) -> Result<Registry> {
        ensure!(
            !matches!(&action, Action::Purge(_)),
            "Deleting module data requires a verified recovery backup"
        );
        self.apply_module_action(action).await
    }
    async fn apply_module_action(&self, action: Action) -> Result<Registry> {
        let tx = self.connection.unchecked_transaction().await?;
        let mut registry = self.module_registry().await?;
        registry.apply(action)?;
        tx.execute("INSERT INTO preferences VALUES('modules',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", (serde_json::to_string(&registry)?,)).await?;
        tx.commit().await?;
        Ok(registry)
    }
    pub async fn purge_module(&self, id: String, backup: &std::path::Path) -> Result<Registry> {
        super::recovery::BackupProof::verify(backup)?;
        self.apply_module_action(Action::Purge(id)).await
    }
    fn module_context(&self) -> (uuid::Uuid, u64) {
        (
            self.module_instance,
            self.module_epoch.load(std::sync::atomic::Ordering::Acquire),
        )
    }
    pub async fn module_session(&self, id: &str) -> Result<crate::modules::Session> {
        let mut session = self.module_registry().await?.session(id)?;
        session.context = Some(self.module_context());
        Ok(session)
    }
    pub async fn read_module_data(
        &self,
        session: &crate::modules::Session,
    ) -> Result<serde_json::Value> {
        ensure!(
            session.context == Some(self.module_context()),
            "This module session belongs to a different or restored notebook"
        );
        self.module_registry().await?.read_data(session)
    }
    pub async fn write_module_data(
        &self,
        session: &crate::modules::Session,
        data: serde_json::Value,
    ) -> Result<()> {
        let tx = self.connection.unchecked_transaction().await?;
        let mut registry = self.module_registry().await?;
        ensure!(
            session.context == Some(self.module_context()),
            "This module session belongs to a different or restored notebook"
        );
        registry.write_data(session, data)?;
        tx.execute("INSERT INTO preferences VALUES('modules',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", (serde_json::to_string(&registry)?,)).await?;
        tx.commit().await?;
        Ok(())
    }
}
