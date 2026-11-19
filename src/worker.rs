use folio::{
    model::Page,
    vault::{Action, Outcome, Vault},
};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

enum Command {
    Save { page: Box<Page>, revision: u64 },
    Workspace(crate::workspace_worker::Request),
    Security(Action),
    Stop,
}
pub enum Event {
    Loaded(Vec<Page>),
    LoadFailed(String),
    Workspace(Result<crate::workspace_worker::Response, String>),
    Saved {
        id: String,
        revision: u64,
    },
    SaveFailed {
        id: String,
        revision: u64,
        message: String,
    },
    Security(Outcome),
    SecurityFailed(String),
}
pub struct Worker {
    commands: Sender<Command>,
    pub events: Receiver<Event>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start(path: PathBuf, wake: rune::Waker) -> Self {
        let (commands, requests) = mpsc::channel();
        let (responses, events) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            let emit = |event| {
                let _ = responses.send(event);
                wake();
            };
            let lock = (|| -> anyhow::Result<std::fs::File> {
                if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                let file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(path.with_extension("lock"))?;
                file.try_lock().map_err(|_| {
                    anyhow::anyhow!("This notebook is already open in another Folio instance.")
                })?;
                Ok(file)
            })();
            let _lock = match lock {
                Ok(lock) => lock,
                Err(e) => {
                    emit(Event::LoadFailed(e.to_string()));
                    return;
                }
            };
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    emit(Event::LoadFailed(e.to_string()));
                    return;
                }
            };
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                runtime.block_on(service(path, requests, &emit))
            }))
            .is_err()
            {
                emit(Event::LoadFailed("The storage engine stopped unexpectedly. The profile was preserved; reopen Folio or restore a verified backup.".into()));
            }
        });
        Self {
            commands,
            events,
            thread: Some(thread),
        }
    }
    pub fn save(&self, page: Page, revision: u64) -> Result<(), String> {
        self.commands
            .send(Command::Save {
                page: Box::new(page),
                revision,
            })
            .map_err(|_| "The backup service has stopped".into())
    }
    pub fn workspace(&self, request: crate::workspace_worker::Request) -> Result<(), String> {
        self.commands
            .send(Command::Workspace(request))
            .map_err(|_| "The storage service has stopped".into())
    }
    pub fn security(&self, action: Action) -> Result<(), String> {
        self.commands
            .send(Command::Security(action))
            .map_err(|_| "The storage service has stopped".into())
    }
}
async fn service(path: PathBuf, requests: Receiver<Command>, emit: &impl Fn(Event)) {
    let mut vault = match Vault::start(path.clone()).await {
        Ok(vault) => vault,
        Err(e) => {
            emit(Event::LoadFailed(format!("{e:#}")));
            return;
        }
    };
    if vault.store.is_some() {
        if let Err(e) = publish(&vault, emit).await {
            emit(Event::LoadFailed(format!("{e:#}")));
            return;
        }
        if let Err(e) = backup(&vault).await {
            emit(Event::Workspace(Err(format!("Automatic backup: {e:#}"))));
        }
    } else {
        emit(Event::Security(Outcome::Locked));
    }
    let mut dirty = false;
    while let Ok(command) = requests.recv() {
        match command {
            Command::Stop => break,
            Command::Save { page, revision } => {
                let id = page.id.clone();
                match vault.save(&page).await {
                    Ok(()) => {
                        dirty = true;
                        emit(Event::Saved { id, revision });
                    }
                    Err(e) => emit(Event::SaveFailed {
                        id,
                        revision,
                        message: format!("{e:#}"),
                    }),
                }
            }
            Command::Security(action) => {
                let mutates = matches!(&action, Action::Enable(_) | Action::Protect { .. });
                if matches!(action, Action::Lock)
                    && dirty
                    && let Err(e) = backup(&vault).await
                {
                    emit(Event::SecurityFailed(format!(
                        "Backup before locking: {e:#}"
                    )));
                    continue;
                }
                let result = vault.action(action).await;
                match result {
                    Ok(outcome) => {
                        dirty |= mutates;
                        let opened = matches!(outcome, Outcome::Opened { .. });
                        emit(Event::Security(outcome));
                        if opened && let Err(e) = publish(&vault, emit).await {
                            emit(Event::LoadFailed(format!("{e:#}")));
                        }
                    }
                    Err(e) => emit(Event::SecurityFailed(format!("{e:#}"))),
                }
            }
            Command::Workspace(request) => {
                let blocking = request.blocking();
                let mutates = request.mutates();
                let library = matches!(&request, crate::workspace_worker::Request::Library(_));
                let result = workspace(&mut vault, &path, request).await;
                dirty |= mutates && result.is_ok();
                emit(Event::Workspace(result.or_else(|e| {
                    if library {
                        return Ok(crate::workspace_worker::Response::Library(Err(format!(
                            "{e:#}"
                        ))));
                    }
                    Ok(crate::workspace_worker::Response::Failure {
                        message: format!("{e:#}"),
                        blocking,
                    })
                })));
            }
        }
    }
    if dirty
        && vault.store.is_some()
        && let Err(e) = backup(&vault).await
    {
        eprintln!("Folio: automatic backup: {e:#}");
    }
    if let Some(store) = &vault.store
        && let Err(error) = store.checkpoint_wal().await
    {
        eprintln!("Folio: checkpoint failed; keep the database and WAL together: {error:#}");
    }
}
async fn workspace(
    vault: &mut Vault,
    path: &std::path::Path,
    request: crate::workspace_worker::Request,
) -> anyhow::Result<crate::workspace_worker::Response> {
    use crate::workspace_worker::{Request, Response};
    let store = vault.store()?;
    match &request {
        Request::Import { page, .. } => anyhow::ensure!(
            !vault.page_keys.contains_key(page)
                && store
                    .load_one(page)
                    .await?
                    .is_none_or(|p| p.protected.is_none()),
            "Attachments are not available in a protected page."
        ),
        Request::DeletePage(id) => anyhow::ensure!(
            vault.page_keys.contains_key(id)
                || store
                    .load_one(id)
                    .await?
                    .is_none_or(|p| p.protected.is_none()),
            "Unlock the page before deleting it."
        ),
        _ => {}
    }
    let response = crate::workspace_worker::handle_secure(
        store,
        path,
        request,
        vault.config.as_ref().zip(vault.key.as_ref()),
    )
    .await?;
    match &response {
        Response::Restored { .. } => vault.page_keys.clear(),
        Response::Deleted { id, .. } => {
            vault.page_keys.remove(id);
        }
        Response::Page(page) => {
            vault.page_keys.remove(&page.id);
        }
        _ => {}
    }
    Ok(response)
}
async fn publish(vault: &Vault, emit: &impl Fn(Event)) -> anyhow::Result<()> {
    let store = vault.store()?;
    emit(Event::Loaded(store.load_or_seed().await?));
    emit(Event::Workspace(Ok(
        crate::workspace_worker::Response::Preferences(store.preferences().await?),
    )));
    emit(Event::Workspace(Ok(
        crate::workspace_worker::Response::Library(Ok(crate::module_library::Reply::Registry(
            store.module_registry().await?,
        ))),
    )));
    Ok(())
}
async fn backup(vault: &Vault) -> anyhow::Result<()> {
    crate::workspace_worker::automatic_backup_secure(
        vault.store()?,
        &vault.path,
        vault.config.as_ref().zip(vault.key.as_ref()),
    )
    .await
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
