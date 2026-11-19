use anyhow::{Context, Result};
use folio::{
    model::Page,
    storage::Store,
    workspace::{self, Asset, HistoryEntry, Notebook, Preferences},
};
use std::path::{Path, PathBuf};

pub enum Request {
    Library(crate::module_library::Job),
    Preferences(Preferences),
    History(String),
    DeletePage(String),
    Checkpoint(String),
    RestoreVersion {
        page: String,
        version: String,
    },
    Backup(PathBuf),
    UpdateBackup(PathBuf),
    ReadBackup(PathBuf),
    ReadBackupWithPassword {
        path: PathBuf,
        password: zeroize::Zeroizing<String>,
    },
    Restore(Box<Notebook>),
    Import {
        page: String,
        after: Option<String>,
        files: Vec<PathBuf>,
    },
    ExportAsset {
        id: String,
        path: PathBuf,
    },
    OpenAsset(String),
}
pub enum Response {
    Library(Result<crate::module_library::Reply, String>),
    Failure {
        message: String,
        blocking: bool,
    },
    BackupDone(String),
    UpdateBackupDone(PathBuf),
    Deleted {
        id: String,
        parent: Option<String>,
    },
    Preferences(Preferences),
    PreferencesStored,
    History {
        page: String,
        versions: Vec<HistoryEntry>,
    },
    Page(Box<Page>),
    Preview(Box<Notebook>),
    BackupLocked(PathBuf),
    Restored {
        pages: Vec<Page>,
        preferences: Preferences,
        modules: folio::modules::Registry,
    },
    Imported {
        page: String,
        after: Option<String>,
        assets: Vec<Asset>,
    },
    Message(String),
}
#[cfg(test)]
pub async fn handle(store: &Store, database: &Path, request: Request) -> Result<Response> {
    handle_secure(store, database, request, None).await
}
pub async fn handle_secure(
    store: &Store,
    database: &Path,
    request: Request,
    vault: Option<(&folio::security::VaultConfig, &folio::security::Key)>,
) -> Result<Response> {
    let assets = workspace::asset_dir(database);
    match request {
        Request::Library(job) => {
            use crate::module_library::{Job, Reply};
            use folio::modules::{self, Action};
            Ok(Response::Library(Ok(match job {
                Job::Inspect(path) => {
                    let raw = workspace::read_limited(&path, modules::MAX_MANIFEST)?;
                    Reply::Candidate(Box::new(modules::Candidate::new(serde_json::from_slice(
                        &raw,
                    )?)?))
                }
                Job::Catalog(path) => {
                    let raw = workspace::read_limited(&path, modules::MAX_CATALOG)?;
                    let catalog: modules::Catalog = serde_json::from_slice(&raw)?;
                    Reply::Registry(store.module_action(Action::Catalog(catalog)).await?)
                }
                Job::Apply(action) => {
                    // Never delete retained data without a complete, verified recovery backup.
                    if let Action::Purge(id) = &action {
                        let mut preview = store.module_registry().await?;
                        preview.apply(action.clone())?;
                        let folder = database.with_extension("backups").join("recovery");
                        std::fs::create_dir_all(&folder)?;
                        let path = folder.join(format!(
                            "before-module-delete-{}.folio",
                            uuid::Uuid::new_v4()
                        ));
                        folio::storage::recovery::write_backup(store, database, &path, vault, true)
                            .await?;
                        return Ok(Response::Library(Ok(Reply::Registry(
                            store.purge_module(id.clone(), &path).await?,
                        ))));
                    }
                    Reply::Registry(store.module_action(action).await?)
                }
            })))
        }
        Request::DeletePage(id) => {
            let parent = store.delete_confirmed(&id).await?;
            Ok(Response::Deleted { id, parent })
        }
        Request::Preferences(prefs) => {
            store.save_preferences(&prefs).await?;
            Ok(Response::PreferencesStored)
        }
        Request::History(page) => Ok(Response::History {
            versions: store.history(Some(&page)).await?,
            page,
        }),
        Request::Checkpoint(page) => {
            let current = store.load_one(&page).await?.context("Page not found")?;
            store.checkpoint(&current).await?;
            Ok(Response::History {
                versions: store.history(Some(&page)).await?,
                page,
            })
        }
        Request::RestoreVersion { page, version } => {
            let current = store.load_one(&page).await?.context("Page not found")?;
            let mut saved = store
                .history(Some(&page))
                .await?
                .into_iter()
                .find(|h| h.id == version)
                .context("Version not found")?;
            let pages = store.load().await?;
            let mut ancestors = std::collections::HashSet::from([page.as_str()]);
            let mut parent = saved.page.parent_id.as_deref();
            while let Some(id) = parent {
                anyhow::ensure!(
                    ancestors.insert(id),
                    "This version would create a cycle in the hierarchy. Move the page first."
                );
                parent = pages
                    .iter()
                    .find(|p| p.id == id)
                    .and_then(|p| p.parent_id.as_deref());
            }
            store.checkpoint(&current).await?;
            saved.page.updated_at = folio::model::now();
            store.save(&saved.page).await?;
            Ok(Response::Page(Box::new(saved.page)))
        }
        request @ (Request::Backup(_) | Request::UpdateBackup(_)) => {
            let update = matches!(request, Request::UpdateBackup(_));
            let path = match request {
                Request::Backup(p) | Request::UpdateBackup(p) => p,
                _ => unreachable!(),
            };
            ensure_destination(database, &path)?;
            folio::storage::recovery::write_backup(store, database, &path, vault, update).await?;
            if update {
                return Ok(Response::UpdateBackupDone(path));
            }
            Ok(Response::BackupDone(format!(
                "Notebook backed up: {}",
                path.display()
            )))
        }
        Request::ReadBackup(path) => {
            match folio::security::read_backup(&path, vault.map(|(_, key)| key), None) {
                Ok(notebook) => Ok(Response::Preview(Box::new(notebook))),
                Err(error) => {
                    let bytes =
                        folio::workspace::read_limited(&path, folio::workspace::MAX_BACKUP)?;
                    if serde_json::from_slice::<folio::security::Sealed>(&bytes).is_ok() {
                        Ok(Response::BackupLocked(path))
                    } else {
                        Err(error)
                    }
                }
            }
        }
        Request::ReadBackupWithPassword { path, password } => {
            anyhow::ensure!(
                vault.is_some(),
                "Encrypt the destination notebook before restoring an encrypted backup."
            );
            Ok(Response::Preview(Box::new(folio::security::read_backup(
                &path,
                None,
                Some(&password),
            )?)))
        }
        Request::Restore(notebook) => {
            anyhow::ensure!(
                vault.is_some()
                    || notebook
                        .pages
                        .iter()
                        .chain(notebook.history.iter().map(|h| &h.page))
                        .all(|p| p.protected.is_none()),
                "Encrypt the destination notebook before restoring protected pages."
            );
            let recovery = database.with_extension("backups").join("recovery");
            std::fs::create_dir_all(&recovery)?;
            folio::storage::recovery::rehearse(&notebook, vault, &recovery).await?;
            let backup = recovery.join(format!("before-restore-{}.folio", uuid::Uuid::new_v4()));
            folio::storage::recovery::write_backup(store, database, &backup, vault, true).await?;
            store.restore_notebook(&notebook, &assets).await?;
            Ok(Response::Restored {
                pages: notebook.pages,
                preferences: notebook.preferences,
                modules: notebook.modules,
            })
        }
        Request::Import { page, after, files } => {
            anyhow::ensure!(files.len() <= 20, "Add no more than 20 files at a time.");
            let mut imported = Vec::new();
            for path in files.iter().take(20) {
                imported.push(workspace::import_asset(path, &assets)?);
            }
            Ok(Response::Imported {
                page,
                after,
                assets: imported,
            })
        }
        Request::ExportAsset { id, path } => {
            ensure_destination(database, &path)?;
            let bytes = workspace::read_limited(
                &workspace::asset_path(&assets, &id)?,
                workspace::MAX_FILE,
            )?;
            use std::io::Write;
            let folder = path.parent().context("Invalid destination")?;
            let mut temp = tempfile::NamedTempFile::new_in(folder)?;
            temp.write_all(&bytes)?;
            temp.as_file().sync_all()?;
            temp.persist(&path).map_err(|e| e.error)?;
            Ok(Response::Message(format!(
                "Attachment exported: {}",
                path.display()
            )))
        }
        Request::OpenAsset(id) => {
            let path = workspace::asset_path(&assets, &id)?;
            anyhow::ensure!(path.is_file(), "The attachment could not be found.");
            let meta = store
                .load()
                .await?
                .into_iter()
                .flat_map(|p| p.blocks)
                .filter_map(|b| b.asset)
                .find(|a| a.id == id)
                .context("Attachment metadata not found")?;
            let folder = assets.join("open").join(&id);
            std::fs::create_dir_all(&folder)?;
            let name = std::path::Path::new(&meta.name)
                .file_name()
                .context("Invalid file name")?;
            let display_path = folder.join(name);
            std::fs::copy(&path, &display_path)?;
            if !cfg!(test) {
                let app = if cfg!(target_os = "macos") {
                    "open"
                } else if cfg!(target_os = "windows") {
                    "explorer.exe"
                } else {
                    "xdg-open"
                };
                std::process::Command::new(app)
                    .arg(display_path)
                    .spawn()
                    .context("Opening document")?;
            }
            Ok(Response::Message(
                "Document opened in its application.".into(),
            ))
        }
    }
}
impl Request {
    pub fn mutates(&self) -> bool {
        matches!(
            self,
            Self::Preferences(_)
                | Self::DeletePage(_)
                | Self::Checkpoint(_)
                | Self::RestoreVersion { .. }
                | Self::Restore(_)
                | Self::Import { .. }
                | Self::Library(
                    crate::module_library::Job::Apply(_) | crate::module_library::Job::Catalog(_)
                )
        )
    }
    pub fn blocking(&self) -> bool {
        matches!(
            self,
            Self::Library(_)
                | Self::DeletePage(_)
                | Self::Backup(_)
                | Self::UpdateBackup(_)
                | Self::ReadBackup(_)
                | Self::ReadBackupWithPassword { .. }
                | Self::Restore(_)
                | Self::RestoreVersion { .. }
                | Self::Import { .. }
        )
    }
}
pub fn ensure_destination(database: &Path, path: &Path) -> Result<()> {
    let resolved = path
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .zip(path.file_name())
        .map(|(p, n)| p.join(n));
    let protected = folio::security::storage_files(database);
    anyhow::ensure!(
        !protected.iter().any(|p| p == path
            || p.canonicalize()
                .ok()
                .zip(path.canonicalize().ok())
                .is_some_and(|(a, b)| a == b)),
        "Choose a file other than the database."
    );
    let assets = workspace::asset_dir(database);
    anyhow::ensure!(
        !path.starts_with(&assets)
            && !resolved
                .as_ref()
                .zip(assets.canonicalize().ok())
                .is_some_and(|(p, a)| p.starts_with(a)),
        "Choose a location outside the attachments directory."
    );
    Ok(())
}
pub async fn automatic_backup_secure(
    store: &Store,
    database: &Path,
    vault: Option<(&folio::security::VaultConfig, &folio::security::Key)>,
) -> Result<()> {
    let folder = database.with_extension("backups");
    std::fs::create_dir_all(&folder)?;
    let name = format!(
        "{}-{}.folio",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        uuid::Uuid::new_v4()
    );
    folio::storage::recovery::write_backup(store, database, &folder.join(name), vault, false)
        .await?;
    let mut files: Vec<_> = std::fs::read_dir(&folder)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "folio"))
        .collect();
    files.sort();
    let excess = files.len().saturating_sub(14);
    for file in files.into_iter().take(excess) {
        std::fs::remove_file(file)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restoring_a_page_version_refuses_a_cycle_after_later_tree_moves() {
        let temp = tempfile::tempdir().unwrap();
        let database = temp.path().join("history.db");
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let store = Store::open(&database).await.unwrap();
            let mut a = Page::new(None);
            let mut b = Page::new(Some(a.id.clone()));
            store.save(&a).await.unwrap();
            store.save(&b).await.unwrap();
            store.checkpoint(&b).await.unwrap();
            let version = store.history(Some(&b.id)).await.unwrap()[0].id.clone();
            b.parent_id = None;
            store.save(&b).await.unwrap();
            a.parent_id = Some(b.id.clone());
            store.save(&a).await.unwrap();
            let before = store.load().await.unwrap();
            let error = handle(
                &store,
                &database,
                Request::RestoreVersion {
                    page: b.id.clone(),
                    version,
                },
            )
            .await
            .err()
            .unwrap();
            assert!(error.to_string().contains("cycle"));
            assert_eq!(store.load().await.unwrap(), before);
        });
    }
}
