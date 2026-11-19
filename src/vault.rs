//! Storage session and explicit transitions between a plain notebook and an encrypted vault.
use crate::{
    model::Page,
    security::{self, Key, PageKey, VaultConfig},
    storage::Store,
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;
pub enum Action {
    Unlock(Zeroizing<String>),
    Enable(Zeroizing<String>),
    Lock,
    Protect { id: String, pin: Zeroizing<String> },
    UnlockPage { id: String, pin: Zeroizing<String> },
    LockPage(String),
}
pub enum Outcome {
    Opened {
        encrypted: bool,
        warning: Option<String>,
    },
    Locked,
    Page {
        page: Box<Page>,
        unlocked: bool,
        warning: Option<String>,
    },
}
pub struct Vault {
    pub path: PathBuf,
    pub store: Option<Store>,
    pub config: Option<VaultConfig>,
    pub key: Option<Key>,
    pub page_keys: HashMap<String, PageKey>,
    failures: HashMap<String, (u32, Instant)>,
}
impl Vault {
    pub async fn start(path: PathBuf) -> Result<Self> {
        let config = VaultConfig::read(&path)?;
        // Once encryption is configured, never silently create an empty replacement database.
        if config.is_some() {
            ensure!(
                security::encrypted_path(&path).is_file(),
                "The encrypted database is missing. Restore the vault files."
            );
        }
        ensure!(
            config.is_some() || !security::encrypted_path(&path).exists(),
            "An encrypted vault exists without its configuration. Also restore the .security.json file; no empty database will be created."
        );
        let store = if config.is_none() {
            Some(Store::open(&path).await?)
        } else {
            None
        };
        Ok(Self {
            path,
            store,
            config,
            key: None,
            page_keys: HashMap::new(),
            failures: HashMap::new(),
        })
    }
    pub fn store(&self) -> Result<&Store> {
        self.store.as_ref().context("The notebook is locked")
    }
    fn allowed(&self, id: &str) -> Result<()> {
        if let Some((failures, last)) = self.failures.get(id) {
            ensure!(
                *failures < 5 || last.elapsed() >= Duration::from_secs(30),
                "Too many attempts. Wait 30 seconds."
            );
        }
        Ok(())
    }
    fn failed(&mut self, id: &str) {
        let entry = self
            .failures
            .entry(id.into())
            .or_insert((0, Instant::now()));
        if entry.1.elapsed() >= Duration::from_secs(30) {
            entry.0 = 0;
        }
        entry.0 += 1;
        entry.1 = Instant::now();
    }
    fn cleanup_legacy(&self, remove_original: bool) -> Result<()> {
        // Recovery is repeatable after a crash between the configuration commit and cleanup.
        let separate_plain_database = self.path.exists() && !remove_original;
        if remove_original {
            remove_database_files(&self.path)?;
        }
        for file in backup_files(&self.path)? {
            let bytes = crate::workspace::read_limited(&file, crate::workspace::MAX_BACKUP)?;
            if serde_json::from_slice::<security::Sealed>(&bytes).is_ok() {
                continue;
            }
            let notebook = security::read_backup(&file, None, None)?;
            security::write_backup(
                &notebook,
                &file,
                self.config.as_ref().context("Missing configuration")?,
                self.key.as_ref().context("Missing key")?,
            )?;
            crate::storage::recovery::read_back(&notebook, &file, self.key.as_ref())?;
        }
        ensure!(
            !self.path.with_extension("recovery").exists(),
            "Earlier raw recovery snapshots remain unencrypted. Preserve them securely before removing them; the verified encrypted recovery backup is in the backups/recovery folder."
        );
        ensure!(
            !separate_plain_database,
            "A separate unencrypted database exists beside the vault and has been kept. Check it before deleting anything."
        );
        Ok(())
    }
    fn protect_backups(&self, id: &str, keys: &PageKey) -> Result<()> {
        for file in backup_files(&self.path)? {
            let mut notebook = security::read_backup(&file, self.key.as_ref(), None)?;
            let mut changed = false;
            for page in notebook
                .pages
                .iter_mut()
                .chain(notebook.history.iter_mut().map(|h| &mut h.page))
            {
                if page.id == id && page.protected.is_none() {
                    *page = security::seal_page(page, keys)?;
                    changed = true;
                }
            }
            if changed {
                security::write_backup(
                    &notebook,
                    &file,
                    self.config.as_ref().context("Missing configuration")?,
                    self.key.as_ref().context("Missing key")?,
                )?;
                crate::storage::recovery::read_back(&notebook, &file, self.key.as_ref())?;
            }
        }
        Ok(())
    }
    pub async fn save(&self, page: &Page) -> Result<()> {
        let old = self.store()?.load_one(&page.id).await?;
        let protected = old.as_ref().is_some_and(|p| p.protected.is_some());
        let value = if page.protected.is_none() && protected {
            security::seal_page(
                page,
                self.page_keys
                    .get(&page.id)
                    .context("Unlock the page before editing it")?,
            )?
        } else {
            page.clone()
        };
        self.store()?.save(&value).await
    }
    pub async fn action(&mut self, action: Action) -> Result<Outcome> {
        match action {
            Action::Unlock(password) => {
                self.allowed("vault")?;
                let config = self.config.as_ref().context("No encrypted vault")?;
                let key = match config.unlock(&password) {
                    Ok(key) => key,
                    Err(_) => {
                        self.failed("vault");
                        anyhow::bail!("Incorrect password or damaged vault");
                    }
                };
                let store = Store::open_with_encryption(
                    &security::encrypted_path(&self.path),
                    Some(config.encryption_options(&key)),
                )
                .await?;
                // Verify readable content before publishing the session/key.
                store.load().await?;
                self.store = Some(store);
                self.key = Some(key);
                self.failures.remove("vault");
                let warning=self.cleanup_legacy(false).err().map(|e|format!("The vault is active. Cleanup of older unencrypted copies is incomplete: {e:#}"));
                Ok(Outcome::Opened {
                    encrypted: true,
                    warning,
                })
            }
            Action::Enable(password) => {
                ensure!(self.config.is_none(), "Encryption is already enabled");
                ensure!(
                    !std::fs::symlink_metadata(&self.path)?
                        .file_type()
                        .is_symlink(),
                    "Use the database's real path instead of a symbolic link before encrypting it."
                );
                let (config, key) = VaultConfig::create(&password)?;
                let target = security::encrypted_path(&self.path);
                ensure!(
                    !target.exists(),
                    "An encrypted database already exists. Do not replace it without checking."
                );
                self.store()?.verify_storage().await?;
                let notebook = self
                    .store()?
                    .notebook(&crate::workspace::asset_dir(&self.path))
                    .await?;
                let prepared = async {
                    let encrypted =
                        Store::open_with_encryption(&target, Some(config.encryption_options(&key)))
                            .await?;
                    encrypted
                        .restore_notebook(&notebook, &crate::workspace::asset_dir(&self.path))
                        .await?;
                    ensure!(
                        encrypted
                            .notebook(&crate::workspace::asset_dir(&self.path))
                            .await?
                            == notebook,
                        "Vault migration verification failed"
                    );
                    let mut rows = encrypted
                        .connection
                        .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
                        .await?;
                    while rows.next().await?.is_some() {}
                    drop(rows);
                    drop(encrypted);
                    let reopened =
                        Store::open_with_encryption(&target, Some(config.encryption_options(&key)))
                            .await?;
                    reopened.verify_storage().await?;
                    ensure!(
                        reopened
                            .notebook(&crate::workspace::asset_dir(&self.path))
                            .await?
                            == notebook,
                        "Vault migration failed verification after reopening"
                    );
                    let recovery = self.path.with_extension("backups").join("recovery");
                    std::fs::create_dir_all(&recovery)?;
                    let backup =
                        recovery.join(format!("before-encryption-{}.folio", uuid::Uuid::new_v4()));
                    security::write_backup(&notebook, &backup, &config, &key)?;
                    crate::storage::recovery::read_back(&notebook, &backup, Some(&key))?;
                    config.write(&self.path)?;
                    Ok::<_, anyhow::Error>(reopened)
                }
                .await;
                let encrypted = match prepared {
                    Ok(store) => store,
                    Err(e) => {
                        if !security::config_path(&self.path).exists() {
                            let _ = remove_database_files(&target);
                        }
                        return Err(e);
                    }
                };
                self.store = Some(encrypted);
                self.config = Some(config);
                self.key = Some(key);
                let warning=self.cleanup_legacy(true).err().map(|e|format!("The vault is active. Cleanup of older unencrypted copies is incomplete: {e:#}"));
                Ok(Outcome::Opened {
                    encrypted: true,
                    warning,
                })
            }
            Action::Lock => {
                ensure!(
                    self.config.is_some(),
                    "Enable encryption before locking the notebook"
                );
                self.page_keys.clear();
                self.store = None;
                self.key = None;
                Ok(Outcome::Locked)
            }
            Action::Protect { id, pin } => {
                ensure!(
                    self.config.is_some() && self.key.is_some(),
                    "Encrypt the database in Settings first"
                );
                let keys = security::new_page_key(&pin)?;
                let page = self.store()?.protect_page(&id, &keys).await?;
                // Protection is committed before cleanup. A cleanup failure must still lock
                // the UI, rather than present plaintext as though protection never happened.
                let cleanup = async {
                    self.protect_backups(&id, &keys)?;
                    self.store()?.connection.execute("VACUUM", ()).await?;
                    let mut rows = self
                        .store()?
                        .connection
                        .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
                        .await?;
                    while rows.next().await?.is_some() {}
                    Ok::<_, anyhow::Error>(())
                }
                .await;
                let warning = cleanup.err().map(|e| {
                    format!(
                        "The page is protected, but cleanup of older copies is incomplete: {e:#}"
                    )
                });
                Ok(Outcome::Page {
                    page: Box::new(page),
                    unlocked: false,
                    warning,
                })
            }
            Action::UnlockPage { id, pin } => {
                let counter = format!("page:{id}");
                self.allowed(&counter)?;
                let page = self
                    .store()?
                    .load_one(&id)
                    .await?
                    .context("Page not found")?;
                let (page, key) = match security::unlock_page(&page, &pin) {
                    Ok(value) => value,
                    Err(_) => {
                        self.failed(&counter);
                        anyhow::bail!("Incorrect PIN or damaged page");
                    }
                };
                let warning = self
                    .protect_backups(&id, &key)
                    .err()
                    .map(|e| format!("Some older backups still need cleanup: {e:#}"));
                self.page_keys.insert(id.clone(), key);
                self.failures.remove(&counter);
                Ok(Outcome::Page {
                    page: Box::new(page),
                    unlocked: true,
                    warning,
                })
            }
            Action::LockPage(id) => {
                let page = self
                    .store()?
                    .load_one(&id)
                    .await?
                    .context("Page not found")?;
                self.page_keys.remove(&id);
                Ok(Outcome::Page {
                    page: Box::new(page),
                    unlocked: false,
                    warning: None,
                })
            }
        }
    }
}
fn remove_database_files(path: &Path) -> Result<()> {
    for file in [
        path.to_path_buf(),
        PathBuf::from(format!("{}-wal", path.display())),
        PathBuf::from(format!("{}-shm", path.display())),
    ] {
        match std::fs::remove_file(file) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

fn backup_files(database: &Path) -> Result<Vec<PathBuf>> {
    let folder = database.with_extension("backups");
    if !folder.exists() {
        return Ok(vec![]);
    }
    let mut stack = vec![folder];
    let mut files = Vec::new();
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            ensure!(
                !kind.is_symlink(),
                "A backup symlink was left untouched; review the backup directory"
            );
            if kind.is_dir() {
                stack.push(entry.path());
            } else if kind.is_file() && entry.path().extension().is_some_and(|e| e == "folio") {
                files.push(entry.path());
            }
        }
    }
    Ok(files)
}
