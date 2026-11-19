use crate::{
    state::{Panel, Phase, State},
    workspace_worker::Request,
};
use folio::modules::{Action, Candidate, Registry};
use rune::{FileDialogOptions, TextEdit, Tx};
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Discover,
    Installed,
    Retained,
}
#[derive(Clone)]
pub enum Confirmation {
    Uninstall(String),
    Purge(String),
}
#[derive(Default)]
pub struct Library {
    pub registry: Registry,
    pub loaded: bool,
    pub pending: bool,
    pub tab: Tab,
    pub search: TextEdit,
    pub sources: bool,
    pub visible_limit: usize,
    pub visible_query: String,
    pub review: Option<Candidate>,
    pub confirmation: Option<Confirmation>,
    pub error: Option<String>,
}
pub enum Job {
    Inspect(std::path::PathBuf),
    Catalog(std::path::PathBuf),
    Apply(Action),
}
pub enum Reply {
    Registry(Registry),
    Candidate(Box<Candidate>),
}
impl State {
    pub fn open_library(&mut self) {
        if self.settings.locked || self.workspace.busy || self.settings.busy {
            return;
        }
        self.library.review = None;
        self.library.confirmation = None;
        self.library.error = None;
        self.edit();
        self.open_settings();
        self.panel = Panel::Library;
    }
    pub fn library_job(&mut self, job: Job) {
        if self.demo
            || self.settings.locked
            || self.settings.busy
            || self.workspace.busy
            || !self.library.loaded
            || !matches!(self.phase, Phase::Ready)
        {
            return;
        }
        self.library.pending = true;
        self.library.error = None;
        self.workspace.busy = true;
        self.workspace_request(Request::Library(job));
        if !self.workspace.busy {
            self.library.pending = false;
            self.library.error = self.export_message.take();
        }
    }
    pub fn library_import(&mut self, catalog: bool, tx: &mut Tx) {
        if self.demo || self.workspace.busy || self.settings.locked {
            return;
        }
        tx.open_file(
            FileDialogOptions {
                title: if catalog {
                    "Add a module catalog"
                } else {
                    "Add a module definition"
                }
                .into(),
                ..Default::default()
            }
            .filter("Folio module JSON", &["json"]),
            move |s: &mut State, result, _| match result {
                Ok(Some(path)) => s.library_job(if catalog {
                    Job::Catalog(path)
                } else {
                    Job::Inspect(path)
                }),
                Ok(None) => {}
                Err(error) => s.library.error = Some(error.to_string()),
            },
        );
    }
    pub fn review_module(&mut self, manifest: serde_json::Value) {
        if self.demo || self.workspace.busy || self.settings.locked {
            return;
        }
        match Candidate::new(manifest).and_then(|candidate| {
            // Exact same rules as persistence, including retained data and version conflicts.
            let mut registry = self.library.registry.clone();
            registry.apply(Action::Install {
                manifest: candidate.manifest.clone(),
                fingerprint: candidate.fingerprint.clone(),
            })?;
            Ok(candidate)
        }) {
            Ok(candidate) => {
                self.library.review = Some(candidate);
                self.library.error = None;
                self.panel = Panel::ModuleReview;
            }
            Err(error) => self.library.error = Some(format!("{error:#}")),
        }
    }
    pub fn install_reviewed_module(&mut self) {
        let Some(candidate) = self.library.review.clone() else {
            return;
        };
        self.library_job(Job::Apply(Action::Install {
            manifest: candidate.manifest,
            fingerprint: candidate.fingerprint,
        }));
    }
    pub fn confirm_module_action(&mut self, action: Confirmation) {
        if self.workspace.busy || self.settings.locked {
            return;
        }
        self.library.confirmation = Some(action);
        self.panel = Panel::ModuleConfirm;
    }
    pub fn apply_module_confirmation(&mut self) {
        let Some(action) = self.library.confirmation.clone() else {
            return;
        };
        self.library_job(Job::Apply(match action {
            Confirmation::Uninstall(id) => Action::Uninstall(id),
            Confirmation::Purge(id) => Action::Purge(id),
        }));
    }
    pub fn close_module_dialog(&mut self) {
        if self.workspace.busy {
            return;
        }
        self.library.review = None;
        self.library.confirmation = None;
        self.library.error = None;
        self.panel = Panel::Library;
    }
    pub fn accept_library(&mut self, reply: Reply) {
        let pending = self.library.pending;
        self.library.pending = false;
        if pending {
            self.workspace.busy = false;
        }
        self.library.error = None;
        match reply {
            Reply::Registry(registry) => {
                self.library.registry = registry;
                self.library.loaded = true;
                if pending {
                    self.library.review = None;
                    self.library.confirmation = None;
                    self.panel = Panel::Library;
                }
            }
            Reply::Candidate(candidate) => self.review_module(candidate.manifest),
        }
    }
}
