use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::document::PageContent;
use crate::model::{Page, welcome};

mod health;
mod modules;
pub mod recovery;
pub use health::{APPLICATION_ID, SCHEMA_VERSION};
pub const TURSO_VERSION: &str = env!("FOLIO_TURSO_VERSION");
pub fn storage_contract() -> String {
    format!(
        "turso-{TURSO_VERSION}-schema{SCHEMA_VERSION}-notebook{}-vault{}",
        crate::document::CURRENT_VERSION,
        crate::security::FORMAT_VERSION
    )
}

/// Owned by one background thread: the UI never waits for database I/O.
pub struct Store {
    _database: turso::Database,
    pub(crate) connection: turso::Connection,
    pub(crate) module_instance: uuid::Uuid,
    pub(crate) module_epoch: std::sync::atomic::AtomicU64,
}

impl Store {
    pub async fn open(path: &Path) -> Result<Self> {
        Self::open_with_encryption(path, None).await
    }
    pub async fn open_with_encryption(
        path: &Path,
        encryption: Option<turso::EncryptionOpts>,
    ) -> Result<Self> {
        // A dependency/schema bump cannot silently keep advertising the old storage contract.
        let _ = crate::updates::config();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).context("Creating the notes directory")?;
        }
        let existed = path.exists();
        let wal = PathBuf::from(format!("{}-wal", path.display()));
        let has_wal = wal.metadata().is_ok_and(|m| m.len() > 32);
        anyhow::ensure!(
            existed || !has_wal,
            "The main database is missing but its WAL exists. Preserve both files; no empty notebook was created."
        );
        if !existed {
            anyhow::ensure!(
                !path.with_extension("backups").exists()
                    && !path.with_extension("recovery").exists()
                    && !crate::workspace::asset_dir(path).exists(),
                "The notebook database is missing but earlier profile files exist. Restore a verified backup; no empty replacement was created."
            );
        }
        if existed {
            anyhow::ensure!(
                path.metadata()?.len() > 0 || has_wal,
                "The database is empty or truncated. Preserve the profile and restore a verified backup."
            );
        }
        // Turso may convert a SQLite journal header when opening a file. Validate a
        // private copy first, so even a refused/foreign/corrupt profile stays byte-for-byte intact.
        let saved = if existed {
            Some(recovery::snapshot(path, "before open")?)
        } else {
            None
        };
        if let Some(saved) = &saved {
            let probe = tempfile::Builder::new()
                .prefix("preflight-")
                .tempdir_in(saved)?;
            let check = probe.path().join("check.db");
            std::fs::copy(saved.join("database"), &check)?;
            if saved.join("database-wal").exists() {
                std::fs::copy(
                    saved.join("database-wal"),
                    probe.path().join("check.db-wal"),
                )?;
            }
            let candidate = Self::connect(&check, encryption.clone()).await?;
            candidate.preflight().await?;
            anyhow::ensure!(
                candidate
                    .scalar(
                        "SELECT COUNT(*) FROM sqlite_schema WHERE type='table' AND name='pages'"
                    )
                    .await?
                    == 1,
                "Existing profile contains no pages table. No empty replacement was created."
            );
        }
        let store = Self::connect(path, encryption).await?;
        let migration = store.preflight().await?;
        if migration {
            if existed {
                anyhow::ensure!(
                    store
                        .scalar(
                            "SELECT COUNT(*) FROM sqlite_schema WHERE type='table' AND name='pages'"
                        )
                        .await?
                        == 1,
                    "Existing profile contains no pages table. No empty replacement was created."
                );
            }
            let tx = store.connection.unchecked_transaction().await?;
            tx.execute(include_str!("../migrations/001_initial.sql"), ())
                .await?;
            tx.execute(
                "CREATE INDEX IF NOT EXISTS pages_parent ON pages(parent_id)",
                (),
            )
            .await?;
            tx.execute(include_str!("../migrations/002_blocks.sql"), ())
                .await?;
            for statement in include_str!("../migrations/003_workspace.sql")
                .split(';')
                .filter(|s| !s.trim().is_empty())
            {
                tx.execute(statement, ()).await?;
            }
            store.migrate_legacy().await?;
            crate::workspace::validate_structure(
                &store.load().await?,
                &store.history(None).await?,
            )?;
            store.preferences().await?;
            tx.execute(format!("PRAGMA user_version={SCHEMA_VERSION}"), ())
                .await?;
            tx.execute(format!("PRAGMA application_id={APPLICATION_ID}"), ())
                .await?;
            tx.commit().await?;
        }
        store.verify_storage().await?;
        if let Some(saved) = saved {
            recovery::mark_verified(&saved, migration || has_wal)?;
        }
        Ok(store)
    }
    async fn connect(path: &Path, encryption: Option<turso::EncryptionOpts>) -> Result<Self> {
        let mut builder =
            turso::Builder::new_local(path.to_str().context("The database path must be UTF-8")?)
                .experimental_vacuum(true);
        if let Some(encryption) = encryption {
            builder = builder
                .experimental_encryption(true)
                .with_encryption(encryption);
        }
        let database = builder.build().await?;
        let connection = database.connect()?;
        connection.execute("PRAGMA synchronous=FULL", ()).await?;
        let store = Self {
            _database: database,
            connection,
            module_instance: uuid::Uuid::new_v4(),
            module_epoch: std::sync::atomic::AtomicU64::new(0),
        };
        anyhow::ensure!(
            store.scalar("PRAGMA synchronous").await? == 2,
            "The database engine did not enable durable transactions"
        );
        Ok(store)
    }

    async fn migrate_legacy(&self) -> Result<()> {
        let mut rows = self.connection.query(
            "SELECT p.id, p.body_html FROM pages p LEFT JOIN page_content c ON c.page_id=p.id WHERE c.page_id IS NULL", ()
        ).await?;
        let mut legacy = Vec::new();
        while let Some(row) = rows.next().await? {
            legacy.push((row.get::<String>(0)?, row.get::<String>(1)?));
        }
        drop(rows);
        // Persist generated block ids once; keep the original HTML and flags untouched.
        for (id, html) in legacy {
            let json = serde_json::to_string(&PageContent::legacy(&html))?;
            self.connection.execute("INSERT INTO page_content (page_id, content_json) VALUES (?, ?) ON CONFLICT(page_id) DO NOTHING", (id, json)).await?;
        }
        Ok(())
    }

    pub async fn load(&self) -> Result<Vec<Page>> {
        let mut rows = self.connection.query(
            "SELECT p.id, p.parent_id, p.title, p.body_html, p.plain_text, p.favorite, p.archived, p.updated_at, c.content_json
             FROM pages p JOIN page_content c ON c.page_id=p.id ORDER BY p.updated_at, p.id", ()
        ).await?;
        let mut pages = Vec::new();
        while let Some(row) = rows.next().await? {
            pages.push(decode_page(row)?);
        }
        Ok(pages)
    }

    pub async fn load_or_seed(&self) -> Result<Vec<Page>> {
        let mut pages = self.load().await?;
        let mut initialized = self
            .connection
            .query("SELECT value FROM preferences WHERE key='initialized'", ())
            .await?;
        let seeded = initialized.next().await?.is_some();
        drop(initialized);
        if pages.is_empty() && !seeded {
            let page = welcome();
            self.save(&page).await?;
            pages.push(page);
        }
        self.connection.execute("INSERT INTO preferences VALUES ('initialized', 'true') ON CONFLICT(key) DO NOTHING", ()).await?;
        Ok(pages)
    }

    pub async fn save(&self, page: &Page) -> Result<()> {
        crate::workspace::validate_page(page)?;
        let old = self.load_one(&page.id).await?;
        let record = if let Some(old) = old.filter(|old| old != page) {
            let mut rows = self
                .connection
                .query(
                    "SELECT MAX(created_at) FROM page_history WHERE page_id=?",
                    (page.id.clone(),),
                )
                .await?;
            let last = rows
                .next()
                .await?
                .and_then(|r| r.get::<Option<i64>>(0).ok())
                .flatten()
                .unwrap_or(0);
            (crate::model::now() - last >= 60).then_some(old)
        } else {
            None
        };
        let content = serde_json::to_string(&page.content())?;
        let transaction = self.connection.unchecked_transaction().await?;
        transaction.execute(
            "INSERT INTO pages (id, parent_id, title, body_html, plain_text, favorite, archived, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET parent_id=excluded.parent_id, title=excluded.title,
             body_html=excluded.body_html, plain_text=excluded.plain_text,
             favorite=excluded.favorite, archived=excluded.archived, updated_at=excluded.updated_at",
            (page.id.clone(), page.parent_id.clone(), page.title.clone(), page.body_html.clone(),
             page.plain_text.clone(), i64::from(page.favorite), i64::from(page.archived), page.updated_at),
        ).await?;
        transaction.execute("INSERT INTO page_content (page_id, content_json) VALUES (?, ?) ON CONFLICT(page_id) DO UPDATE SET content_json=excluded.content_json",
            (page.id.clone(), content)).await?;
        if let Some(old) = record {
            transaction
                .execute(
                    "INSERT INTO page_history VALUES (?, ?, ?, ?)",
                    (
                        uuid::Uuid::new_v4().to_string(),
                        old.id.clone(),
                        crate::model::now(),
                        serde_json::to_string(&old)?,
                    ),
                )
                .await?;
            transaction.execute("DELETE FROM page_history WHERE page_id=? AND id NOT IN (SELECT id FROM page_history WHERE page_id=? ORDER BY created_at DESC, rowid DESC LIMIT 50)", (old.id.clone(), old.id)).await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}

pub fn database_path() -> PathBuf {
    if let Some(path) = std::env::var_os("FOLIO_DB_PATH") {
        return PathBuf::from(path);
    }
    let base = if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
    } else if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
    };
    base.unwrap_or_else(|| PathBuf::from("data"))
        .join("Folio/folio.db")
}

fn decode_page(row: turso::Row) -> Result<Page> {
    let content: PageContent =
        serde_json::from_str(&row.get::<String>(8)?).context("Reading the block document")?;
    anyhow::ensure!(
        matches!(content.version, 1..=crate::document::CURRENT_VERSION),
        "This document requires a newer version of Folio"
    );
    anyhow::ensure!(
        !content.blocks.is_empty(),
        "The document contains no blocks"
    );
    let mut ids = std::collections::HashSet::new();
    anyhow::ensure!(
        content.blocks.iter().all(|block| ids.insert(&block.id)),
        "The document contains duplicate block IDs"
    );
    Ok(Page {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        title: row.get(2)?,
        body_html: row.get(3)?,
        plain_text: row.get(4)?,
        favorite: row.get::<i64>(5)? != 0,
        archived: row.get::<i64>(6)? != 0,
        updated_at: row.get(7)?,
        blocks: content.blocks,
        emoji: content.emoji,
        icon_color: content.icon_color,
        cover: content.cover,
        wide: content.wide,
        collection: content.collection,
        properties: content.properties,
        dashboard: content.dashboard,
        order: content.order,
        protected: content.protected,
    })
}

impl Store {
    pub async fn load_one(&self, id: &str) -> Result<Option<Page>> {
        let mut rows = self.connection.query("SELECT p.id, p.parent_id, p.title, p.body_html, p.plain_text, p.favorite, p.archived, p.updated_at, c.content_json FROM pages p JOIN page_content c ON c.page_id=p.id WHERE p.id=?", (id,)).await?;
        rows.next().await?.map(decode_page).transpose()
    }
    pub async fn preferences(&self) -> Result<crate::workspace::Preferences> {
        let mut rows = self
            .connection
            .query("SELECT value FROM preferences WHERE key='app'", ())
            .await?;
        let mut prefs: crate::workspace::Preferences = rows
            .next()
            .await?
            .map(|r| r.get::<String>(0))
            .transpose()?
            .map(|v| serde_json::from_str(&v))
            .transpose()?
            .unwrap_or_default();
        prefs.sidebar_width = prefs.sidebar_width.clamp(180, 360);
        Ok(prefs)
    }
    pub async fn save_preferences(&self, prefs: &crate::workspace::Preferences) -> Result<()> {
        self.connection.execute("INSERT INTO preferences VALUES ('app', ?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", (serde_json::to_string(prefs)?,)).await?;
        Ok(())
    }
    pub async fn history(&self, page: Option<&str>) -> Result<Vec<crate::workspace::HistoryEntry>> {
        let mut rows = self.connection.query("SELECT id, page_id, created_at, content_json FROM page_history WHERE (? IS NULL OR page_id=?) ORDER BY created_at DESC, rowid DESC", (page, page)).await?;
        let mut versions = Vec::new();
        while let Some(row) = rows.next().await? {
            versions.push(crate::workspace::HistoryEntry {
                id: row.get(0)?,
                page_id: row.get(1)?,
                created_at: row.get(2)?,
                page: serde_json::from_str(&row.get::<String>(3)?)?,
            });
        }
        Ok(versions)
    }
    pub async fn checkpoint(&self, page: &Page) -> Result<()> {
        crate::workspace::validate_page(page)?;
        self.connection
            .execute(
                "INSERT INTO page_history VALUES (?, ?, ?, ?)",
                (
                    uuid::Uuid::new_v4().to_string(),
                    page.id.clone(),
                    crate::model::now(),
                    serde_json::to_string(page)?,
                ),
            )
            .await?;
        self.connection.execute("DELETE FROM page_history WHERE page_id=? AND id NOT IN (SELECT id FROM page_history WHERE page_id=? ORDER BY created_at DESC, rowid DESC LIMIT 50)", (page.id.clone(), page.id.clone())).await?;
        Ok(())
    }
    pub async fn notebook(&self, assets: &Path) -> Result<crate::workspace::Notebook> {
        let mut notebook = crate::workspace::Notebook::capture(
            self.load().await?,
            self.preferences().await?,
            self.history(None).await?,
            assets,
        )?;
        notebook.modules = self.module_registry().await?;
        if !notebook.modules.is_empty() {
            notebook.format = 2;
        }
        notebook.validate()?;
        Ok(notebook)
    }
    /// Asset files are installed first. A failed SQL transaction keeps the current notebook intact.
    pub async fn restore_notebook(
        &self,
        notebook: &crate::workspace::Notebook,
        assets: &Path,
    ) -> Result<()> {
        notebook.install_assets(assets)?;
        let tx = self.connection.unchecked_transaction().await?;
        tx.execute("DELETE FROM page_content", ()).await?;
        tx.execute("DELETE FROM pages", ()).await?;
        tx.execute("DELETE FROM page_history", ()).await?;
        for p in &notebook.pages {
            tx.execute(
                "INSERT INTO pages VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    p.id.clone(),
                    p.parent_id.clone(),
                    p.title.clone(),
                    p.body_html.clone(),
                    p.plain_text.clone(),
                    i64::from(p.favorite),
                    i64::from(p.archived),
                    p.updated_at,
                ),
            )
            .await?;
            tx.execute(
                "INSERT INTO page_content VALUES (?, ?)",
                (p.id.clone(), serde_json::to_string(&p.content())?),
            )
            .await?;
        }
        for h in notebook.history.iter().rev() {
            tx.execute(
                "INSERT INTO page_history VALUES (?, ?, ?, ?)",
                (
                    h.id.clone(),
                    h.page_id.clone(),
                    h.created_at,
                    serde_json::to_string(&h.page)?,
                ),
            )
            .await?;
        }
        tx.execute("INSERT INTO preferences VALUES ('app', ?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", (serde_json::to_string(&notebook.preferences)?,)).await?;
        tx.execute("INSERT INTO preferences VALUES('modules',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", (serde_json::to_string(&notebook.modules)?,)).await?;
        tx.execute(
            "INSERT INTO preferences VALUES ('initialized', 'true') ON CONFLICT(key) DO NOTHING",
            (),
        )
        .await?;
        tx.commit().await?;
        self.module_epoch
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        Ok(())
    }
}

impl Store {
    /// Replace the current page and every saved revision in one transaction.
    pub async fn protect_page(&self, id: &str, keys: &crate::security::PageKey) -> Result<Page> {
        let page = self.load_one(id).await?.context("Page not found")?;
        let locked = crate::security::seal_page(&page, keys)?;
        let history = self.history(Some(id)).await?;
        let protected_history: Vec<_> = history
            .into_iter()
            .map(|h| {
                crate::security::seal_page(&h.page, keys)
                    .and_then(|p| Ok((h.id, serde_json::to_string(&p)?)))
            })
            .collect::<Result<_>>()?;
        let tx = self.connection.unchecked_transaction().await?;
        tx.execute(
            "UPDATE pages SET title=?, body_html=?, plain_text=? WHERE id=?",
            (
                locked.title.clone(),
                locked.body_html.clone(),
                locked.plain_text.clone(),
                id,
            ),
        )
        .await?;
        tx.execute(
            "UPDATE page_content SET content_json=? WHERE page_id=?",
            (serde_json::to_string(&locked.content())?, id),
        )
        .await?;
        for (version, json) in protected_history {
            tx.execute(
                "UPDATE page_history SET content_json=? WHERE id=?",
                (json, version),
            )
            .await?;
        }
        tx.commit().await?;
        Ok(locked)
    }
}

impl Store {
    pub async fn delete_archived(&self, id: &str) -> Result<Option<String>> {
        self.delete_page(id, true).await
    }

    pub async fn delete_confirmed(&self, id: &str) -> Result<Option<String>> {
        self.delete_page(id, false).await
    }

    async fn delete_page(&self, id: &str, require_archived: bool) -> Result<Option<String>> {
        let page = self.load_one(id).await?.context("Page not found")?;
        anyhow::ensure!(
            !require_archived || page.archived,
            "Archive the page before permanently deleting it."
        );
        let tx = self.connection.unchecked_transaction().await?;
        tx.execute(
            "UPDATE pages SET parent_id=? WHERE parent_id=?",
            (page.parent_id.clone(), id),
        )
        .await?;
        tx.execute("DELETE FROM page_history WHERE page_id=?", (id,))
            .await?;
        tx.execute("DELETE FROM page_content WHERE page_id=?", (id,))
            .await?;
        tx.execute("DELETE FROM pages WHERE id=?", (id,)).await?;
        tx.execute(
            "INSERT INTO preferences VALUES ('initialized','true') ON CONFLICT(key) DO NOTHING",
            (),
        )
        .await?;
        tx.commit().await?;
        Ok(page.parent_id)
    }
}
