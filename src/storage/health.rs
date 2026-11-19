//! Fail closed before any schema/content write. No automatic repair or salvage.
use super::Store;
use crate::workspace::{HistoryEntry, validate_structure};
use anyhow::{Context, Result, ensure};
use std::collections::HashSet;

pub const SCHEMA_VERSION: i64 = 4;
pub const APPLICATION_ID: i64 = 0x464f4c49; // FOLI
const TABLES: &[(&str, &[&str])] = &[
    (
        "pages",
        &[
            "id",
            "parent_id",
            "title",
            "body_html",
            "plain_text",
            "favorite",
            "archived",
            "updated_at",
        ],
    ),
    ("page_content", &["page_id", "content_json"]),
    ("preferences", &["key", "value"]),
    (
        "page_history",
        &["id", "page_id", "created_at", "content_json"],
    ),
];
impl Store {
    pub async fn scalar(&self, sql: &str) -> Result<i64> {
        let mut rows = self.connection.query(sql, ()).await?;
        rows.next()
            .await?
            .context("Missing database metadata")?
            .get(0)
            .map_err(Into::into)
    }
    pub async fn checkpoint_wal(&self) -> Result<()> {
        let mut rows = self
            .connection
            .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
            .await?;
        while let Some(row) = rows.next().await? {
            ensure!(
                row.get::<i64>(0)? == 0,
                "The database checkpoint is busy; the WAL has been preserved"
            );
        }
        Ok(())
    }
    pub async fn check_integrity(&self) -> Result<()> {
        let mut rows = self
            .connection
            .query("PRAGMA integrity_check", ())
            .await
            .context("Checking database integrity")?;
        let mut checked = false;
        while let Some(row) = rows.next().await? {
            // Do not include page contents in an integrity diagnostic.
            ensure!(
                row.get::<String>(0)? == "ok",
                "Database integrity check failed. Preserve this profile and restore a verified backup."
            );
            checked = true;
        }
        ensure!(checked, "Database engine returned no integrity result");
        Ok(())
    }
    pub(super) async fn preflight(&self) -> Result<bool> {
        let version = self.scalar("PRAGMA user_version").await?;
        let app = self.scalar("PRAGMA application_id").await?;
        ensure!(
            matches!(version, 0..=SCHEMA_VERSION),
            "This database requires a newer version of Folio"
        );
        ensure!(
            app == 0 || app == APPLICATION_ID,
            "This is not a Folio database"
        );
        self.check_integrity().await?;
        let mut rows = self
            .connection
            .query(
                "SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                (),
            )
            .await?;
        let mut names = HashSet::new();
        while let Some(row) = rows.next().await? {
            names.insert(row.get::<String>(0)?);
        }
        drop(rows);
        ensure!(
            names
                .iter()
                .all(|name| TABLES.iter().any(|(known, _)| known == name)),
            "Unknown database tables; use a compatible Folio version"
        );
        for (table, expected) in TABLES {
            if !names.contains(*table) {
                continue;
            }
            let mut rows = self
                .connection
                .query(format!("PRAGMA table_info({table})"), ())
                .await?;
            let mut columns = Vec::new();
            while let Some(row) = rows.next().await? {
                columns.push(row.get::<String>(1)?);
            }
            ensure!(
                columns == *expected,
                "Incompatible database schema in {table}; no migration was performed"
            );
        }
        if names.is_empty() {
            ensure!(
                version == 0 && app == 0,
                "Folio database tables are missing; refusing to create an empty notebook"
            );
            return Ok(true);
        }
        ensure!(names.contains("pages"), "The pages table is missing");
        if version != 0 || app != 0 {
            ensure!(
                names.len() == TABLES.len(),
                "Folio database tables are missing; no automatic repair was attempted"
            );
        }
        // A v1 database has only pages; v2 adds page_content; v3 adds both workspace tables.
        ensure!(
            names.contains("preferences") == names.contains("page_history"),
            "Incomplete workspace schema"
        );
        ensure!(
            !names.contains("preferences") || names.contains("page_content"),
            "The block documents table is missing"
        );
        if names.contains("page_content") {
            ensure!(self.scalar("SELECT COUNT(*) FROM pages p LEFT JOIN page_content c ON c.page_id=p.id WHERE c.page_id IS NULL").await? == 0, "A page's block document is missing; refusing to regenerate it from HTML");
            ensure!(self.scalar("SELECT COUNT(*) FROM page_content c LEFT JOIN pages p ON p.id=c.page_id WHERE p.id IS NULL").await? == 0, "An orphaned block document exists");
            let pages = self.load().await?;
            let history: Vec<HistoryEntry> = if names.contains("page_history") {
                self.history(None).await?
            } else {
                vec![]
            };
            validate_structure(&pages, &history)?;
        }
        if names.contains("preferences") {
            ensure!(
                self.scalar(
                    "SELECT COUNT(*) FROM preferences WHERE key NOT IN ('app','initialized','modules')"
                )
                .await?
                    == 0,
                "Unknown stored preferences; use a compatible Folio version"
            );
            self.preferences().await?;
            self.module_registry().await?;
        }
        Ok(version != SCHEMA_VERSION || app != APPLICATION_ID)
    }
    /// Checks the whole logical notebook as well as the engine's page/index structures.
    pub async fn verify_storage(&self) -> Result<()> {
        ensure!(
            !self.preflight().await?,
            "Database migration has not been completed"
        );
        Ok(())
    }
}
