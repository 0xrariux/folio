use crate::state::{Panel, State};
use folio::export;
use rune::{FileDialogOptions, Tx};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread::JoinHandle,
};

#[derive(Clone, Copy)]
pub enum ExportKind {
    Markdown,
    CsvView,
    CsvAll,
}

pub struct ExportTask {
    result: Receiver<Result<PathBuf, String>>,
    thread: Option<JoinHandle<()>>,
}
impl Drop for ExportTask {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl State {
    pub fn export(&mut self, kind: ExportKind, tx: &mut Tx) {
        if self.settings.locked || self.current().is_some_and(|p| p.protected.is_some()) {
            self.export_message = Some("Unlock the page before exporting it.".into());
            return;
        }
        if self.export_busy {
            return;
        }
        self.edit();
        let Some(page) = self.current() else { return };
        let document = match kind {
            ExportKind::Markdown => export::markdown(page, &self.pages),
            ExportKind::CsvView | ExportKind::CsvAll => {
                let sources = self.tracking_sources();
                if page.collection.is_none() && page.dashboard.is_none() {
                    return;
                }
                let mut records = match kind {
                    ExportKind::CsvAll => self
                        .pages
                        .iter()
                        .filter(|p| {
                            !p.archived
                                && sources
                                    .iter()
                                    .any(|s| p.parent_id.as_deref() == Some(&s.id))
                        })
                        .collect(),
                    _ => self.tracking_items(),
                };
                if matches!(kind, ExportKind::CsvAll) {
                    records.sort_by(|a, b| {
                        a.display_title()
                            .cmp(b.display_title())
                            .then_with(|| a.id.cmp(&b.id))
                    });
                }
                export::csv_with_context(page.display_title(), &sources, &records, &self.pages)
            }
        };
        let protected_page = self.protected_page(&page.id);
        let asset_folder = self.asset_folder();
        self.panel = Panel::Closed;
        self.export_busy = true;
        self.export_message = Some("Choose an export destination…".into());
        tx.save_file(
            FileDialogOptions {
                title: if protected_page {
                    "Export the unlocked page as plain text"
                } else {
                    "Export from Folio"
                }
                .into(),
                file_name: Some(document.file_name),
                ..Default::default()
            }
            .filter(
                if document.extension == "md" {
                    "Markdown"
                } else {
                    "CSV"
                },
                &[document.extension],
            ),
            move |s: &mut State, result, _| match result {
                Ok(Some(path)) => {
                    let database = folio::storage::database_path();
                    if crate::workspace_worker::ensure_destination(&database, &path).is_err() {
                        s.export_busy = false;
                        s.export_message =
                            Some("Cannot export: choose a file other than the database.".into());
                        return;
                    }
                    s.export_message = Some("Exporting…".into());
                    let (send, result) = mpsc::channel();
                    let wake = s.export_wake.clone();
                    let thread = std::thread::spawn(move || {
                        let result = folio::workspace::export_assets(
                            &document.assets,
                            &asset_folder,
                            &path
                                .parent()
                                .unwrap_or(std::path::Path::new("."))
                                .join("folio-assets"),
                        )
                        .and_then(|()| export::write_atomic(&path, &document.contents))
                        .map(|()| path)
                        .map_err(|e| format!("{e:#}"));
                        let _ = send.send(result);
                        if let Some(wake) = wake {
                            wake();
                        }
                    });
                    s.export_task = Some(ExportTask {
                        result,
                        thread: Some(thread),
                    });
                }
                Ok(None) => {
                    s.export_busy = false;
                    s.export_message = None;
                }
                Err(error) => {
                    s.export_busy = false;
                    s.export_message = Some(format!("Cannot export: {error}"));
                }
            },
        );
    }

    pub fn poll_export(&mut self) -> bool {
        let Some(task) = &self.export_task else {
            return false;
        };
        let result = match task.result.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => Err("The export service has stopped.".into()),
        };
        self.export_message = Some(match result {
            Ok(path) => format!("Export saved: {}", path.display()),
            Err(error) => format!("Export failed: {error}"),
        });
        self.export_task = None;
        self.export_busy = false;
        true
    }
}
