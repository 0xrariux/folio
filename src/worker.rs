use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use folio::{model::Page, storage::Store};

enum Command {
    Save { page: Box<Page>, revision: u64 },
    Workspace(crate::workspace_worker::Request),
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
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    emit(Event::LoadFailed(error.to_string()));
                    return;
                }
            };
            runtime.block_on(async {
                let store = match Store::open(&path).await {
                    Ok(store) => store,
                    Err(error) => {
                        emit(Event::LoadFailed(format!("{error:#}")));
                        return;
                    }
                };
                match store.load_or_seed().await {
                    Ok(pages) => emit(Event::Loaded(pages)),
                    Err(error) => {
                        emit(Event::LoadFailed(format!("{error:#}")));
                        return;
                    }
                }
                match store.preferences().await {
                    Ok(prefs) => emit(Event::Workspace(Ok(
                        crate::workspace_worker::Response::Preferences(prefs),
                    ))),
                    Err(error) => emit(Event::Workspace(Err(format!("Préférences : {error:#}")))),
                }
                if let Err(error) = crate::workspace_worker::automatic_backup(&store, &path).await {
                    emit(Event::Workspace(Err(format!(
                        "Sauvegarde automatique : {error:#}"
                    ))));
                }
                let mut dirty = false;
                while let Ok(command) = requests.recv() {
                    match command {
                        Command::Save { page, revision } => {
                            dirty = true;
                            let id = page.id.clone();
                            match store.save(&page).await {
                                Ok(()) => emit(Event::Saved { id, revision }),
                                Err(error) => emit(Event::SaveFailed {
                                    id,
                                    revision,
                                    message: format!("{error:#}"),
                                }),
                            }
                        }
                        Command::Workspace(request) => {
                            let blocking = request.blocking();
                            emit(Event::Workspace(
                                crate::workspace_worker::handle(&store, &path, request)
                                    .await
                                    .or_else(|e| {
                                        Ok(crate::workspace_worker::Response::Failure {
                                            message: format!("{e:#}"),
                                            blocking,
                                        })
                                    }),
                            ));
                        }
                        Command::Stop => break,
                    }
                }
                if dirty
                    && let Err(error) =
                        crate::workspace_worker::automatic_backup(&store, &path).await
                {
                    eprintln!("Folio : sauvegarde automatique : {error:#}");
                }
            });
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
            .map_err(|_| "Le service de sauvegarde est arrêté".into())
    }
    pub fn workspace(&self, request: crate::workspace_worker::Request) -> Result<(), String> {
        self.commands
            .send(Command::Workspace(request))
            .map_err(|_| "Le service de stockage est arrêté".into())
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // FIFO: every queued save finishes before Stop, including on window close.
        let _ = self.commands.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
