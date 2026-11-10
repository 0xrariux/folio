use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::document::PageContent;
use crate::model::{Page, welcome};

/// Owned by one background thread: the UI never waits for database I/O.
pub struct Store {
    _database: turso::Database,
    pub(crate) connection: turso::Connection,
}

impl Store {
    pub async fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).context("Création du dossier des notes")?;
        }
        let path = path
            .to_str()
            .context("Le chemin de la base doit être UTF-8")?;
        let database = turso::Builder::new_local(path).build().await?;
        let connection = database.connect()?;
        // One idempotent migration for the initial schema, including across restarts.
        connection
            .execute(include_str!("../migrations/001_initial.sql"), ())
            .await?;
        connection
            .execute(
                "CREATE INDEX IF NOT EXISTS pages_parent ON pages(parent_id)",
                (),
            )
            .await?;
        connection
            .execute(include_str!("../migrations/002_blocks.sql"), ())
            .await?;
        for statement in include_str!("../migrations/003_workspace.sql")
            .split(';')
            .filter(|s| !s.trim().is_empty())
        {
            connection.execute(statement, ()).await?;
        }
        let store = Self {
            _database: database,
            connection,
        };
        store.migrate_legacy().await?;
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
        if pages.is_empty() {
            let page = welcome();
            self.save(&page).await?;
            pages.push(page);
        }
        Ok(pages)
    }

    pub async fn save(&self, page: &Page) -> Result<()> {
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
        serde_json::from_str(&row.get::<String>(8)?).context("Lecture du document de blocs")?;
    anyhow::ensure!(
        matches!(content.version, 1..=6),
        "Cette version du document nécessite une version plus récente de Folio"
    );
    anyhow::ensure!(
        !content.blocks.is_empty(),
        "Le document ne contient aucun bloc"
    );
    let mut ids = std::collections::HashSet::new();
    anyhow::ensure!(
        content.blocks.iter().all(|block| ids.insert(&block.id)),
        "Le document contient des identifiants de blocs dupliqués"
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
        cover: content.cover,
        wide: content.wide,
        collection: content.collection,
        properties: content.properties,
        dashboard: content.dashboard,
        order: content.order,
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
        crate::workspace::Notebook::capture(
            self.load().await?,
            self.preferences().await?,
            self.history(None).await?,
            assets,
        )
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
        tx.commit().await?;
        Ok(())
    }
}
