use crate::{
    editor::EditorBlock,
    state::{Panel, State},
    workspace_worker::{Request, Response},
};
use folio::{
    document::{Kind, NoteBlock},
    workspace::{HistoryEntry, Notebook, Preferences},
};
use rune::{FileDialogOptions, Tx};

#[derive(Default)]
pub struct WorkspaceState {
    pub preferences: Preferences,
    pub preferences_loaded: bool,
    pub busy: bool,
    pub versions: Vec<HistoryEntry>,
    pub preview: Option<Box<Notebook>>,
    pub images: Option<rune::ImageCache>,
    pub calendar_month: String,
    pub selected_records: std::collections::BTreeSet<String>,
    pub entity_drag: Option<crate::entity_drag::Session>,
    pub demo_assets: Option<tempfile::TempDir>,
}
impl State {
    pub fn workspace_request(&mut self, request: Request) {
        if self.demo {
            self.workspace.busy = false;
            self.export_message =
                Some("This storage command is available in your local notebook.".into());
            return;
        }
        if let Err(error) = self
            .worker
            .as_ref()
            .ok_or("The storage service is unavailable".to_string())
            .and_then(|w| w.workspace(request))
        {
            self.workspace.busy = false;
            self.export_message = Some(error);
        }
    }
    pub fn save_preferences(&mut self) {
        self.workspace.preferences.dark = self.dark;
        self.workspace.preferences.last_page = self.selected.clone();
        if self.workspace.preferences_loaded && !self.demo {
            self.workspace_request(Request::Preferences(self.workspace.preferences.clone()));
        }
    }
    pub fn switch_theme(&mut self) {
        self.dark = !self.dark;
        self.save_preferences();
    }
    pub fn backup_dialog(&mut self, tx: &mut Tx) {
        if self.workspace.busy {
            return;
        }
        self.edit();
        tx.save_file(
            FileDialogOptions {
                title: "Back up the entire notebook".into(),
                file_name: Some("My notebook.folio".into()),
                ..Default::default()
            }
            .filter("Folio notebook", &["folio"]),
            |s: &mut State, result, _| match result {
                Ok(Some(path)) => {
                    s.workspace.busy = true;
                    s.workspace_request(Request::Backup(path));
                }
                Ok(None) => {}
                Err(e) => s.export_message = Some(e.to_string()),
            },
        );
    }
    pub fn restore_dialog(&mut self, tx: &mut Tx) {
        if self.workspace.busy {
            return;
        }
        tx.open_file(
            FileDialogOptions {
                title: "Open a Folio backup".into(),
                ..Default::default()
            }
            .filter("Folio notebook", &["folio"]),
            |s: &mut State, result, _| match result {
                Ok(Some(path)) => {
                    s.workspace.busy = true;
                    s.workspace_request(Request::ReadBackup(path));
                }
                Ok(None) => {}
                Err(e) => s.export_message = Some(e.to_string()),
            },
        );
    }
    pub fn confirm_restore(&mut self) {
        let Some(bundle) = self.workspace.preview.take() else {
            return;
        };
        self.workspace.busy = true;
        self.workspace_request(Request::Restore(bundle));
    }
    pub fn open_history(&mut self) {
        self.edit();
        self.workspace.versions.clear();
        self.panel = Panel::History;
        if let Some(id) = self.selected.clone() {
            self.workspace_request(Request::History(id));
        }
    }
    pub fn checkpoint(&mut self) {
        self.edit();
        if let Some(id) = self.selected.clone() {
            self.workspace_request(Request::Checkpoint(id));
        }
    }
    pub fn restore_version(&mut self, id: &str) {
        if let Some(page) = self.selected.clone() {
            self.workspace_request(Request::RestoreVersion {
                page,
                version: id.into(),
            });
        }
    }
    pub fn attachment_dialog(&mut self, tx: &mut Tx) {
        tx.open_file(
            FileDialogOptions {
                title: "Add an image or document".into(),
                ..Default::default()
            },
            |s: &mut State, result, _| match result {
                Ok(Some(path)) => s.import_files(&[path]),
                Ok(None) => {}
                Err(e) => s.export_message = Some(e.to_string()),
            },
        );
    }
    pub fn import_files(&mut self, files: &[std::path::PathBuf]) {
        if self.workspace.busy {
            return;
        }
        let Some(page) = self.selected.clone() else {
            return;
        };
        if self
            .current()
            .is_none_or(|p| p.archived || p.collection.is_some() || p.dashboard.is_some())
        {
            return;
        }
        if self.demo {
            if self.workspace.demo_assets.is_none() {
                self.workspace.demo_assets = tempfile::tempdir().ok();
            }
            let Some(folder) = self.workspace.demo_assets.as_ref() else {
                return;
            };
            let result: anyhow::Result<Vec<_>> = files
                .iter()
                .take(20)
                .map(|p| folio::workspace::import_asset(p, folder.path()))
                .collect();
            match result {
                Ok(assets) => self.accept_workspace(Ok(Response::Imported {
                    page,
                    after: self.active.clone(),
                    assets,
                })),
                Err(e) => self.export_message = Some(e.to_string()),
            }
        } else {
            self.workspace.busy = true;
            self.workspace_request(Request::Import {
                page,
                after: self.active.clone(),
                files: files.to_vec(),
            });
        }
    }
    pub fn asset_folder(&self) -> std::path::PathBuf {
        self.workspace.demo_assets.as_ref().map_or_else(
            || folio::workspace::asset_dir(&folio::storage::database_path()),
            |d| d.path().to_path_buf(),
        )
    }
    pub fn export_asset(&mut self, asset: &folio::workspace::Asset, tx: &mut Tx) {
        let id = asset.id.clone();
        tx.save_file(
            FileDialogOptions {
                title: "Export attachment".into(),
                file_name: Some(asset.name.clone()),
                ..Default::default()
            },
            move |s: &mut State, result, _| match result {
                Ok(Some(path)) => s.workspace_request(Request::ExportAsset {
                    id: id.clone(),
                    path,
                }),
                Ok(None) => {}
                Err(e) => s.export_message = Some(e.to_string()),
            },
        );
    }
    pub fn accept_workspace(&mut self, result: Result<Response, String>) {
        if self.settings.update_backup_pending
            && matches!(&result, Err(_) | Ok(Response::Failure { .. }))
        {
            self.settings.update_backup_pending = false;
            self.settings.install_when_saved = false;
            self.settings.update_status = "Backup failed. Update cancelled.".into();
        }
        match result {
            Ok(Response::Library(Ok(reply))) => self.accept_library(reply),
            Ok(Response::Library(Err(error))) => {
                self.workspace.busy = false;
                self.library.pending = false;
                self.library.error = Some(error);
            }
            Ok(Response::UpdateBackupDone(path)) => {
                self.workspace.busy = false;
                if self.settings.update_backup_pending {
                    self.settings.update_backup_pending = false;
                    self.settings.update_backup = Some(path);
                    self.settings.install_when_saved = true;
                }
            }
            Ok(Response::Deleted { id, parent }) => self.apply_deleted(&id, parent),
            Ok(Response::Failure { message, blocking }) => {
                if blocking {
                    self.workspace.busy = false;
                }
                self.export_message = Some(message);
            }
            Ok(Response::BackupDone(message)) => {
                self.workspace.busy = false;
                self.export_message = Some(message);
            }
            Err(error) => {
                self.workspace.busy = false;
                self.export_message = Some(error);
            }
            Ok(Response::Preferences(prefs)) => {
                self.dark = prefs.dark;
                self.workspace.preferences = prefs;
                self.workspace.preferences_loaded = true;
                if let Some(id) = self
                    .workspace
                    .preferences
                    .last_page
                    .clone()
                    .filter(|id| self.pages.iter().any(|p| p.id == *id && !p.archived))
                {
                    self.select(&id);
                }
            }
            Ok(Response::PreferencesStored) => {}
            Ok(Response::Message(message)) => {
                self.export_message = Some(message);
            }
            Ok(Response::History { page, versions }) => {
                if self.selected.as_ref() == Some(&page) {
                    self.workspace.versions = versions;
                }
            }
            Ok(Response::BackupLocked(path)) => {
                self.workspace.busy = false;
                self.security_form(crate::settings::SecurityForm::UnlockBackup(path));
            }
            Ok(Response::Preview(bundle)) => {
                self.workspace.busy = false;
                self.workspace.preview = Some(bundle);
                self.panel = Panel::RestorePreview;
            }
            Ok(Response::Restored {
                pages,
                preferences,
                modules,
            }) => {
                self.settings.unlocked.clear();
                self.settings.locked_pages.clear();
                self.accept_library(crate::module_library::Reply::Registry(modules));
                self.workspace.busy = false;
                self.pending.clear();
                self.pages = pages;
                self.pages.sort_by_key(|p| p.order);
                self.selected = None;
                self.history.clear();
                self.selection.clear();
                self.accept_workspace(Ok(Response::Preferences(preferences)));
                if self.selected.is_none()
                    && let Some(id) = self.pages.first().map(|p| p.id.clone())
                {
                    self.select(&id);
                }
                self.panel = Panel::Closed;
                self.export_message =
                    Some("Notebook restored. A backup of the previous state has been kept.".into());
            }
            Ok(Response::Page(page)) => {
                self.settings.unlocked.remove(&page.id);
                self.settings.locked_pages.remove(&page.id);
                self.workspace.busy = false;
                let id = page.id.clone();
                if let Some(current) = self.pages.iter_mut().find(|p| p.id == id) {
                    *current = *page;
                }
                self.pages.sort_by_key(|p| p.order);
                if self.selected.as_ref() == Some(&id) {
                    self.selected = None;
                    self.select(&id);
                }
                self.export_message = Some(
                    "Version restored; the previous state is available in the history.".into(),
                );
            }
            Ok(Response::Imported {
                page,
                after,
                assets,
            }) => {
                self.workspace.busy = false;
                let mut blocks: Vec<_> = assets
                    .into_iter()
                    .map(|asset| {
                        let mut b = NoteBlock::text(
                            if asset.mime.starts_with("image/") {
                                Kind::Image
                            } else {
                                Kind::Attachment
                            },
                            &asset.name,
                        );
                        b.asset = Some(asset);
                        b
                    })
                    .collect();
                if self.selected.as_ref() == Some(&page) {
                    let source =
                        after.and_then(|id| self.blocks.iter().position(|b| b.data.id == id));
                    if let Some(i) = source.filter(|&i| {
                        matches!(self.blocks[i].data.kind, Kind::Image | Kind::Attachment)
                            && self.blocks[i].data.asset.is_none()
                    }) && !blocks.is_empty()
                    {
                        let mut first = blocks.remove(0);
                        first.id = self.blocks[i].data.id.clone();
                        first.indent = self.blocks[i].data.indent;
                        self.blocks[i] = EditorBlock::from(first);
                    }
                    let end = source.map_or(self.blocks.len(), |i| self.block_end(i));
                    self.blocks
                        .splice(end..end, blocks.into_iter().map(EditorBlock::from));
                    self.commit_blocks(true);
                } else if let Some(target) = self.pages.iter_mut().find(|p| p.id == page) {
                    target.blocks.extend(blocks);
                    target.sync_document();
                    self.persist(&page);
                }
                self.panel = Panel::Closed;
            }
        }
    }
}
