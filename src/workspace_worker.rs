use anyhow::{Context, Result};
use folio::{
    model::Page,
    storage::Store,
    workspace::{self, Asset, HistoryEntry, Notebook, Preferences},
};
use std::path::{Path, PathBuf};

pub enum Request {
    Preferences(Preferences),
    History(String),
    Checkpoint(String),
    RestoreVersion {
        page: String,
        version: String,
    },
    Backup(PathBuf),
    ReadBackup(PathBuf),
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
    Failure {
        message: String,
        blocking: bool,
    },
    BackupDone(String),
    Preferences(Preferences),
    PreferencesStored,
    History {
        page: String,
        versions: Vec<HistoryEntry>,
    },
    Page(Box<Page>),
    Preview(Box<Notebook>),
    Restored {
        pages: Vec<Page>,
        preferences: Preferences,
    },
    Imported {
        page: String,
        after: Option<String>,
        assets: Vec<Asset>,
    },
    Message(String),
}
pub async fn handle(store: &Store, database: &Path, request: Request) -> Result<Response> {
    let assets = workspace::asset_dir(database);
    match request {
        Request::Preferences(prefs) => {
            store.save_preferences(&prefs).await?;
            Ok(Response::PreferencesStored)
        }
        Request::History(page) => Ok(Response::History {
            versions: store.history(Some(&page)).await?,
            page,
        }),
        Request::Checkpoint(page) => {
            let current = store.load_one(&page).await?.context("Page introuvable")?;
            store.checkpoint(&current).await?;
            Ok(Response::History {
                versions: store.history(Some(&page)).await?,
                page,
            })
        }
        Request::RestoreVersion { page, version } => {
            let current = store.load_one(&page).await?.context("Page introuvable")?;
            let mut saved = store
                .history(Some(&page))
                .await?
                .into_iter()
                .find(|h| h.id == version)
                .context("Version introuvable")?;
            let pages = store.load().await?;
            let mut ancestors = std::collections::HashSet::from([page.as_str()]);
            let mut parent = saved.page.parent_id.as_deref();
            while let Some(id) = parent {
                anyhow::ensure!(
                    ancestors.insert(id),
                    "Cette version créerait un cycle dans l’arborescence. Déplacez d’abord la page."
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
        Request::Backup(path) => {
            ensure_destination(database, &path)?;
            store.notebook(&assets).await?.write(&path)?;
            Ok(Response::BackupDone(format!(
                "Carnet sauvegardé : {}",
                path.display()
            )))
        }
        Request::ReadBackup(path) => Ok(Response::Preview(Box::new(Notebook::read(&path)?))),
        Request::Restore(notebook) => {
            automatic_backup(store, database).await?;
            store.restore_notebook(&notebook, &assets).await?;
            Ok(Response::Restored {
                pages: notebook.pages,
                preferences: notebook.preferences,
            })
        }
        Request::Import { page, after, files } => {
            anyhow::ensure!(files.len() <= 20, "Ajoutez au plus 20 fichiers à la fois.");
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
            let folder = path.parent().context("Destination invalide")?;
            let mut temp = tempfile::NamedTempFile::new_in(folder)?;
            temp.write_all(&bytes)?;
            temp.as_file().sync_all()?;
            temp.persist(&path).map_err(|e| e.error)?;
            Ok(Response::Message(format!(
                "Pièce jointe exportée : {}",
                path.display()
            )))
        }
        Request::OpenAsset(id) => {
            let path = workspace::asset_path(&assets, &id)?;
            anyhow::ensure!(path.is_file(), "La pièce jointe est introuvable.");
            let meta = store
                .load()
                .await?
                .into_iter()
                .flat_map(|p| p.blocks)
                .filter_map(|b| b.asset)
                .find(|a| a.id == id)
                .context("Métadonnées de pièce jointe introuvables")?;
            let folder = assets.join("open").join(&id);
            std::fs::create_dir_all(&folder)?;
            let name = std::path::Path::new(&meta.name)
                .file_name()
                .context("Nom de fichier invalide")?;
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
                    .context("Ouverture du document")?;
            }
            Ok(Response::Message(
                "Document ouvert dans son application.".into(),
            ))
        }
    }
}
impl Request {
    pub fn blocking(&self) -> bool {
        matches!(
            self,
            Self::Backup(_)
                | Self::ReadBackup(_)
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
    let protected = [
        database.to_path_buf(),
        PathBuf::from(format!("{}-wal", database.display())),
        PathBuf::from(format!("{}-shm", database.display())),
    ];
    anyhow::ensure!(
        !protected.iter().any(|p| p == path
            || p.canonicalize()
                .ok()
                .zip(path.canonicalize().ok())
                .is_some_and(|(a, b)| a == b)),
        "Choisissez un fichier distinct de la base de données."
    );
    let assets = workspace::asset_dir(database);
    anyhow::ensure!(
        !path.starts_with(&assets)
            && !resolved
                .as_ref()
                .zip(assets.canonicalize().ok())
                .is_some_and(|(p, a)| p.starts_with(a)),
        "Choisissez un emplacement hors du dossier de pièces jointes."
    );
    Ok(())
}
pub async fn automatic_backup(store: &Store, database: &Path) -> Result<()> {
    let folder = database.with_extension("backups");
    std::fs::create_dir_all(&folder)?;
    let name = format!(
        "{}-{}.folio",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        uuid::Uuid::new_v4()
    );
    store
        .notebook(&workspace::asset_dir(database))
        .await?
        .write(&folder.join(name))?;
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
