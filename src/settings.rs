use crate::state::{Panel, State};
use folio::{
    updates,
    vault::{Action, Outcome},
};
use rune::{Latest, TextEdit, Tx};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Clone)]
pub enum SecurityForm {
    EnableVault,
    UnlockVault,
    ProtectPage(String),
    UnlockPage(String),
    UnlockBackup(PathBuf),
}
#[derive(Clone)]
pub(crate) enum SecurityOperation {
    Vault,
    Lock,
    Page { id: String, unlocked: bool },
}
pub struct Settings {
    pub encrypted: bool,
    pub locked: bool,
    pub busy: bool,
    pub(crate) pending_security: Option<SecurityOperation>,
    pub form: Option<SecurityForm>,
    pub secret: TextEdit,
    pub confirmation: TextEdit,
    pub error: Option<String>,
    pub unlocked: HashSet<String>,
    pub locked_pages: HashMap<String, folio::model::Page>,
    pub version_check: Latest<Result<updates::RemoteBuild, String>>,
    pub checking: bool,
    pub remote: Option<updates::RemoteBuild>,
    pub update_status: String,
    pub update_task: Option<updates::Task>,
    pub ready: Option<updates::Prepared>,
    pub install_when_saved: bool,
    pub update_backup_pending: bool,
    pub update_backup: Option<PathBuf>,
    pub next_version_check: std::time::Instant,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            encrypted: false,
            locked: false,
            busy: false,
            pending_security: None,
            form: None,
            secret: TextEdit::new("").secure(),
            confirmation: TextEdit::new("").secure(),
            error: None,
            unlocked: HashSet::new(),
            locked_pages: HashMap::new(),
            version_check: Latest::default(),
            checking: false,
            remote: None,
            update_status: String::new(),
            update_task: None,
            ready: None,
            install_when_saved: false,
            update_backup_pending: false,
            update_backup: None,
            next_version_check: std::time::Instant::now(),
        }
    }
}
impl State {
    pub fn open_settings(&mut self) {
        if let Some(id) = self
            .selected
            .clone()
            .filter(|id| self.settings.unlocked.contains(id))
        {
            self.lock_page(&id);
            if let Some(locked) = self.settings.locked_pages.remove(&id)
                && let Some(page) = self.pages.iter_mut().find(|p| p.id == id)
            {
                *page = locked;
            }
            self.settings.unlocked.remove(&id);
            self.selected = None;
            self.select(&id);
        }
        self.panel = Panel::Settings;
    }
    pub fn cancel_security(&mut self) {
        self.settings.secret = TextEdit::new("").secure();
        self.settings.confirmation = TextEdit::new("").secure();
        self.settings.form = None;
        self.settings.error = None;
        self.panel = Panel::Closed;
    }
    pub fn security_form(&mut self, form: SecurityForm) {
        if self.settings.busy {
            return;
        }
        self.settings.form = Some(form);
        self.settings.secret = TextEdit::new("").secure();
        self.settings.confirmation = TextEdit::new("").secure();
        self.settings.error = None;
        self.panel = Panel::Security;
    }
    pub fn protected_page(&self, id: &str) -> bool {
        self.settings.unlocked.contains(id)
            || self
                .pages
                .iter()
                .any(|p| p.id == id && p.protected.is_some())
    }
    pub fn protect_current(&mut self) {
        if self
            .selected
            .as_ref()
            .is_some_and(|id| self.protected_page(id))
        {
            self.export_message = Some("This page is already protected.".into());
            return;
        }
        let Some(page) = self.current() else {
            return;
        };
        if !self.settings.encrypted {
            self.open_settings();
            self.settings.error = Some("Enable database encryption first.".into());
            return;
        }
        if page.collection.is_some()
            || page.dashboard.is_some()
            || self
                .pages
                .iter()
                .any(|p| p.parent_id.as_deref() == Some(&page.id))
        {
            self.settings.error = Some(
                "PIN protection is available for notes without subpages, databases, or dashboards."
                    .into(),
            );
            self.open_settings();
            return;
        }
        self.security_form(SecurityForm::ProtectPage(page.id.clone()));
    }
    pub fn submit_security(&mut self) {
        if self.settings.busy {
            return;
        }
        let Some(form) = self.settings.form.clone() else {
            return;
        };
        if matches!(
            form,
            SecurityForm::EnableVault | SecurityForm::ProtectPage(_)
        ) && self.settings.secret.text != self.settings.confirmation.text
        {
            self.settings.error = Some("The two entries do not match.".into());
            return;
        }
        let password = Zeroizing::new(self.settings.secret.text.clone());
        self.settings.secret = TextEdit::new("").secure();
        self.settings.confirmation = TextEdit::new("").secure();
        let action = match form {
            SecurityForm::EnableVault => {
                self.edit();
                Action::Enable(password)
            }
            SecurityForm::UnlockVault => Action::Unlock(password),
            SecurityForm::ProtectPage(id) => {
                self.edit();
                Action::Protect { id, pin: password }
            }
            SecurityForm::UnlockPage(id) => Action::UnlockPage { id, pin: password },
            SecurityForm::UnlockBackup(path) => {
                self.workspace.busy = true;
                self.workspace_request(crate::workspace_worker::Request::ReadBackupWithPassword {
                    path,
                    password,
                });
                return;
            }
        };
        self.send_security(action);
    }
    pub fn send_security(&mut self, action: Action) {
        if self.demo {
            self.settings.error =
                Some("Security is available in your local notebook, outside demo mode.".into());
            return;
        }
        if self.export_busy {
            self.settings.error = Some("Wait for the export to finish.".into());
            return;
        }
        let operation = match &action {
            Action::Enable(_) | Action::Unlock(_) => SecurityOperation::Vault,
            Action::Lock => SecurityOperation::Lock,
            Action::Protect { id, .. } | Action::LockPage(id) => SecurityOperation::Page {
                id: id.clone(),
                unlocked: false,
            },
            Action::UnlockPage { id, .. } => SecurityOperation::Page {
                id: id.clone(),
                unlocked: true,
            },
        };
        match self
            .worker
            .as_ref()
            .ok_or("Storage service unavailable".to_owned())
            .and_then(|w| w.security(action))
        {
            Ok(()) => {
                self.settings.busy = true;
                self.settings.pending_security = Some(operation);
                self.settings.error = None;
            }
            Err(e) => self.settings.error = Some(e),
        }
    }
    pub fn lock_notebook(&mut self) {
        if !self.settings.busy {
            self.edit();
            self.save_preferences();
            self.send_security(Action::Lock);
        }
    }
    pub fn lock_page(&mut self, id: &str) {
        if !self.settings.busy {
            self.edit();
            self.send_security(Action::LockPage(id.into()));
        }
    }
    pub fn accept_security(&mut self, outcome: Outcome) {
        let completed = match (&self.settings.pending_security, &outcome) {
            (Some(SecurityOperation::Vault), Outcome::Opened { .. })
            | (Some(SecurityOperation::Lock), Outcome::Locked) => true,
            (
                Some(SecurityOperation::Page { id, unlocked }),
                Outcome::Page {
                    page,
                    unlocked: result,
                    ..
                },
            ) => id == &page.id && unlocked == result,
            _ => false,
        };
        if completed || !matches!(outcome, Outcome::Page { .. }) {
            self.settings.pending_security = None;
            self.settings.busy = false;
            self.settings.error = None;
            self.settings.secret = TextEdit::new("").secure();
            self.settings.confirmation = TextEdit::new("").secure();
        }
        match outcome {
            Outcome::Opened { encrypted, warning } => {
                self.export_message = warning.clone();
                self.settings.error = warning;
                self.settings.encrypted = encrypted;
                self.settings.locked = false;
                self.settings.form = None;
                self.settings.unlocked.clear();
                self.settings.locked_pages.clear();
                if self.workspace.images.is_none()
                    && let Some(wake) = self.export_wake.clone()
                {
                    self.workspace.images = Some(rune::ImageCache::new(wake));
                }
                self.panel = Panel::Settings;
            }
            Outcome::Locked => {
                self.library = Default::default();
                self.pages.clear();
                self.selected = None;
                self.blocks.clear();
                self.title = TextEdit::new("");
                self.active = None;
                self.undo.clear();
                self.redo.clear();
                self.history.clear();
                self.selection.clear();
                self.workspace.versions.clear();
                self.workspace.preview = None;
                self.pending.clear();
                self.settings.unlocked.clear();
                self.settings.locked_pages.clear();
                self.search = TextEdit::new("");
                self.collection_search = TextEdit::new("");
                self.workspace.images = None;
                self.settings.encrypted = true;
                self.settings.locked = true;
                self.settings.form = Some(SecurityForm::UnlockVault);
                self.panel = Panel::Security;
            }
            Outcome::Page {
                page,
                unlocked,
                warning,
            } => {
                let previous_panel = self.panel.clone();
                let was_settings = matches!(self.panel, Panel::Settings);
                let was_current = self.selected.as_deref() == Some(&page.id);
                let form_matches = completed
                    && matches!(self.settings.form.as_ref(),Some(SecurityForm::ProtectPage(id)|SecurityForm::UnlockPage(id)) if id==&page.id);
                if let Some(warning) = warning {
                    self.export_message = Some(warning);
                }
                let id = page.id.clone();
                if unlocked {
                    if let Some(locked) = self
                        .pages
                        .iter()
                        .find(|p| p.id == id && p.protected.is_some())
                    {
                        self.settings
                            .locked_pages
                            .insert(id.clone(), locked.clone());
                    }
                    self.settings.unlocked.insert(id.clone());
                } else {
                    self.settings.unlocked.remove(&id);
                    self.settings.locked_pages.remove(&id);
                }
                if let Some(current) = self.pages.iter_mut().find(|p| p.id == id) {
                    *current = *page;
                }
                self.workspace.versions.clear();
                self.workspace.preview = None;
                if self.selected.as_deref() == Some(&id) {
                    self.selected = None;
                    self.select(&id);
                }
                self.refresh_references(&id);
                if form_matches {
                    self.settings.form = None;
                }
                if matches!(
                    previous_panel,
                    Panel::Library | Panel::ModuleReview | Panel::ModuleConfirm
                ) {
                    self.panel = previous_panel;
                } else if was_settings {
                    self.panel = Panel::Settings;
                } else if completed && (was_current || form_matches) {
                    self.panel = Panel::Closed;
                } else {
                    self.panel = previous_panel;
                }
            }
        }
    }
    pub fn check_version(&mut self) {
        if self.settings.checking || self.settings.update_task.is_some() {
            return;
        }
        self.settings.next_version_check =
            std::time::Instant::now() + std::time::Duration::from_secs(24 * 60 * 60);
        self.settings.checking = true;
        self.settings.update_status = "Checking GitHub…".into();
        self.settings
            .version_check
            .request((), |_| updates::check().map_err(|e| format!("{e:#}")));
    }
    pub fn prepare_update(&mut self) {
        if self.demo || self.settings.locked || self.settings.update_task.is_some() {
            return;
        }
        let Some(remote) = self.settings.remote.clone() else {
            return;
        };
        if !remote.ahead {
            return;
        }
        let Some(wake) = self.export_wake.clone() else {
            return;
        };
        self.edit();
        self.settings.ready = None;
        self.settings.update_backup = None;
        self.settings.update_status = "Preparing the update…".into();
        self.settings.update_task = Some(updates::Task::start(remote, wake));
    }
    pub fn poll_settings(&mut self, tx: &mut Tx) -> bool {
        let mut changed = false;
        if !cfg!(test)
            && !self.demo
            && !self.settings.checking
            && self.settings.update_task.is_none()
            && std::time::Instant::now() >= self.settings.next_version_check
        {
            self.check_version();
            changed = true;
        }
        if let Some(result) = self.settings.version_check.take() {
            self.settings.checking = false;
            changed = true;
            match result {
                Ok(remote) => {
                    self.settings.update_status = if remote.revision == updates::REVISION
                        && remote.version == updates::VERSION
                        && env!("FOLIO_BUILD_DIRTY") != "true"
                    {
                        "Up to date.".into()
                    } else {
                        if remote.ahead {
                            "Update available.".into()
                        } else {
                            "No newer version.".into()
                        }
                    };
                    self.settings.remote = Some(remote);
                }
                Err(e) => {
                    self.settings.remote = None;
                    self.settings.update_status = e;
                }
            }
        }
        let events: Vec<_> = self
            .settings
            .update_task
            .as_ref()
            .map(|task| task.events.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            changed = true;
            match event {
                updates::Event::Stage(message) => self.settings.update_status = message,
                updates::Event::Failed(message) => {
                    self.settings.update_status = message;
                    self.settings.update_task = None;
                }
                updates::Event::Ready(prepared) => {
                    self.settings.ready = Some(*prepared);
                    self.settings.update_status = "Ready to install.".into();
                    self.settings.update_task = None;
                }
            }
        }
        if self.settings.install_when_saved && self.pending.values().any(|p| p.error.is_some()) {
            self.settings.install_when_saved = false;
            self.settings.update_status = "Resolve save errors before installing.".into();
            changed = true;
        }
        if self.settings.install_when_saved && self.pending.is_empty() {
            self.settings.install_when_saved = false;
            self.install_update(tx);
            changed = true;
        }
        changed
    }
    pub fn install_update(&mut self, tx: &mut Tx) {
        if self.export_busy
            || self.workspace.busy
            || self.settings.busy
            || self.settings.locked
            || self.demo
        {
            self.settings.update_status = "Finish the current operations before installing.".into();
            return;
        }
        self.edit();
        if self.pending.values().any(|p| p.error.is_some()) {
            self.settings.update_status = "Resolve save errors before installing.".into();
            return;
        }
        if !self.pending.is_empty() {
            self.settings.install_when_saved = true;
            self.settings.update_status = "Saving notes before installation…".into();
            return;
        }
        if self.settings.ready.is_none() || self.settings.update_backup_pending {
            return;
        }
        if self.settings.update_backup.is_none() {
            let result = (|| -> anyhow::Result<PathBuf> {
                updates::installed_bundle(&std::env::current_exe()?)?;
                let directory = folio::storage::database_path()
                    .with_extension("backups")
                    .join("updates");
                std::fs::create_dir_all(&directory)?;
                Ok(directory.join(format!("before-update-{}.folio", uuid::Uuid::new_v4())))
            })();
            match result {
                Ok(path) => {
                    self.save_preferences();
                    self.settings.update_backup_pending = true;
                    self.settings.update_status = "Backing up notebook…".into();
                    self.workspace.busy = true;
                    self.workspace_request(crate::workspace_worker::Request::UpdateBackup(path));
                    if !self.workspace.busy {
                        self.settings.update_backup_pending = false;
                        self.settings.update_status =
                            "Backup unavailable. Update cancelled.".into();
                    }
                }
                Err(error) => self.settings.update_status = format!("Cannot install: {error:#}"),
            }
            return;
        }
        let prepared = self.settings.ready.take().expect("prepared update");
        let backup = self
            .settings
            .update_backup
            .take()
            .expect("completed backup");
        // FIFO shutdown flushes queued writes and releases the database before handoff.
        drop(self.worker.take());
        match updates::handoff(prepared, &backup) {
            Ok(()) => tx.quit(),
            Err(error) => {
                self.settings.update_status = format!("Installation failed: {error:#}");
                if let Some(wake) = self.export_wake.clone() {
                    self.worker = Some(crate::worker::Worker::start(
                        folio::storage::database_path(),
                        wake,
                    ));
                }
            }
        }
    }
}
