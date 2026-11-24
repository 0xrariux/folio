mod block_drag;
mod calendar;
mod collections;
mod command_palette;
mod custom_fields;
mod demo_tracking;
mod demo_workspace;
mod editor;
mod entity_drag;
mod exporting;
mod linking;
mod lists;
mod module_library;
mod navigation;
mod notebook;
mod overview;
mod saved_views;
mod selection;
mod settings;
mod state;
mod table_layout;
mod tracking;
mod ui;
mod worker;
mod workspace_worker;

use folio::storage::database_path;
use rune::{AppSpec, Key, KeyEvent, Tx};
use state::State;

fn init() -> State {
    if std::env::var_os("FOLIO_DEMO").is_some() {
        State::demo()
    } else {
        State::new()
    }
}

fn setup(state: &mut State, wake: rune::Waker) {
    state.workspace.images = Some(rune::ImageCache::new(wake.clone()));
    state.export_wake = Some(wake.clone());
    state.settings.version_check.set_waker(wake.clone());
    if let Ok(message) = std::env::var("FOLIO_UPDATE_ERROR") {
        state.settings.update_status = message;
        state.settings.page = settings::SettingsPage::Updates;
        state.panel = state::Panel::Settings;
    }
    if !state.demo {
        state.worker = Some(worker::Worker::start(database_path(), wake));
    }
}

fn poll(state: &mut State, _: f64, tx: &mut Tx) -> bool {
    let events: Vec<_> = state
        .worker
        .as_ref()
        .map(|worker| worker.events.try_iter().collect())
        .unwrap_or_default();
    let mut changed = poll_updates(state, events, tx);
    changed |= state.poll_settings(tx);
    if !state.demo
        && ((matches!(state.phase, state::Phase::Ready)
            && state.workspace.preferences_loaded
            && state.library.loaded)
            || state.settings.locked)
    {
        folio::updates::acknowledge_startup();
    }
    if !state.demo {
        let today = folio::dashboard::local_today();
        if today != state.today {
            state.today = today;
            changed = true;
        }
        tx.wake_after(std::time::Duration::from_secs(60));
    }
    if changed {
        tx.instant();
    }
    changed
}

fn poll_updates(state: &mut State, events: Vec<worker::Event>, tx: &mut Tx) -> bool {
    let had_pending = !state.pending.is_empty();
    let previous_message = state.export_message.clone();
    let mut errors = Vec::new();
    let mut saved = false;
    let changed = state.poll_export() || !events.is_empty();
    for event in events {
        use worker::Event;
        use workspace_worker::Response;
        match &event {
            Event::LoadFailed(message)
            | Event::Workspace(Err(message))
            | Event::Workspace(Ok(Response::Failure { message, .. })) => {
                errors.push(message.clone());
            }
            Event::SaveFailed {
                id,
                revision,
                message,
            } if state
                .pending
                .get(id)
                .is_some_and(|p| p.revision == *revision) =>
            {
                errors.push(format!("Save failed: {message}"));
            }
            Event::Saved { .. } => saved = true,
            _ => {}
        }
        state.accept(event);
    }
    // Tx keeps one announcement per poll: combine errors and give them priority.
    if !errors.is_empty() {
        tx.announce(errors.join(". "), rune::LiveRegion::Assertive);
    } else if state.export_message != previous_message {
        if let Some(message) = &state.export_message {
            let how = if message.starts_with("Export failed") {
                rune::LiveRegion::Assertive
            } else {
                rune::LiveRegion::Polite
            };
            tx.announce(message, how);
        }
    } else if saved && had_pending && state.pending.is_empty() {
        tx.announce("All changes saved", rune::LiveRegion::Polite);
    }
    changed
}

fn on_key(state: &mut State, event: &KeyEvent, tx: &mut Tx) -> bool {
    if state.workspace.busy || state.settings.busy || state.settings.locked {
        return true;
    }
    if state.handle_selection_key(event, tx) {
        return true;
    }
    match event.key {
        Key::Command('n') => {
            state.create(false);
            tx.focus("page-title", false);
        }
        Key::Command('z') => state.undo_document(event.mods.shift, tx),
        Key::Command('s') => {
            state.edit();
            state.retry();
        }
        Key::Command('k') => {
            state.open_palette(crate::command_palette::Mode::Commands);
            tx.focus("palette-search", false);
        }
        Key::Escape => {
            if matches!(
                state.panel,
                state::Panel::ModuleReview | state::Panel::ModuleConfirm
            ) {
                state.close_module_dialog();
            } else if matches!(state.panel, state::Panel::Security) {
                state.cancel_security();
            } else {
                state.panel = state::Panel::Closed;
            }
        }
        _ => return false,
    }
    true
}

fn main() {
    // Drain queued storage writes through Worker::drop before exiting.
    rune::orderly_exit();
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|a| a == "--apply-update") {
        let result = args
            .get(2)
            .ok_or_else(|| anyhow::anyhow!("Missing installation plan"))
            .and_then(|p| folio::updates::apply(std::path::Path::new(p)));
        if let Err(error) = result {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return;
    }
    if args.get(1).is_some_and(|a| a == "--verify-backup") {
        let result = (|| -> anyhow::Result<()> {
            let path = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Missing backup path"))?;
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(folio::storage::recovery::certify_plain_backup(
                    std::path::Path::new(path),
                ))?;
            println!("Backup restored, reopened and verified in an isolated database.");
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
        return;
    }
    if args.get(1).is_some_and(|a| a == "--release-info") {
        println!(
            "{}",
            serde_json::json!({
                "version": folio::updates::VERSION, "revision": folio::updates::REVISION,
                "dirty": env!("FOLIO_BUILD_DIRTY") == "true",
                "target": folio::updates::config().target,
                "storage_contract": folio::updates::config().storage_contract,
                "rune_revision": serde_json::from_str::<serde_json::Value>(include_str!("../folio-update.json")).expect("valid Rune pin")["rune_revision"],
                "turso": folio::storage::TURSO_VERSION, "rust": env!("FOLIO_RUST_VERSION"),
                "schema_version": folio::storage::SCHEMA_VERSION, "document_version": folio::document::CURRENT_VERSION, "backup_verification": 1, "module_api": folio::modules::API_VERSION,
            })
        );
        return;
    }
    if std::env::args().any(|arg| arg == "--check-update") {
        match folio::updates::check() {
            Ok(remote) => println!(
                "Local {} {} ; GitHub {} {} ; update available: {}",
                folio::updates::VERSION,
                folio::updates::REVISION,
                remote.version,
                remote.revision,
                remote.ahead
            ),
            Err(e) => {
                eprintln!("{e:#}");
                std::process::exit(1);
            }
        }
        return;
    }
    if std::env::args().any(|arg| arg == "--version") {
        println!(
            "Folio {} ({})",
            folio::updates::VERSION,
            folio::updates::REVISION
        );
        return;
    }
    rune::run(AppSpec {
        min_size: Some((920.0, 620.0)),
        setup: Some(setup),
        poll: Some(poll),
        on_key: Some(on_key),
        scenes: &[
            ("settings", |s| s.open_settings()),
            ("library", |s| s.open_library()),
            ("security", |s| {
                s.security_form(crate::settings::SecurityForm::EnableVault)
            }),
            ("locked", |s| {
                s.accept_security(folio::vault::Outcome::Locked)
            }),
            ("workspace", |s| s.demo_workspace()),
            ("calendar", |s| s.demo_calendar()),
            ("lists", |s| s.demo_lists()),
            ("icon-controls", |s| s.demo_control_catalog()),
            ("icon-catalog", |s| s.demo_icon_catalog()),
            ("launch-table", |s| {
                if s.demo {
                    s.select("demo-launch");
                }
            }),
            ("drag-list", |s| s.demo_drag_list()),
            ("saved-views", |s| s.demo_saved_views()),
            ("late-view", |s| {
                s.demo_saved_views();
                if s.demo {
                    s.select_view("demo-view-late");
                }
            }),
            ("tracker", |s| {
                if s.demo {
                    s.select("demo-tracker");
                }
            }),
            ("dashboard", |s| {
                if s.demo {
                    s.select("demo-dashboard");
                }
            }),
            ("tracker-dashboard", |s| {
                if s.demo {
                    s.select("demo-tracker");
                    s.collection_view(folio::collection::View::Dashboard);
                }
            }),
            ("table-custom", |s| {
                if s.demo {
                    s.select("demo-tracker");
                    s.collection_view(folio::collection::View::Table);
                    if let Some(collection) = s
                        .pages
                        .iter_mut()
                        .find(|p| p.id == "demo-tracker")
                        .and_then(|p| p.collection.as_mut())
                    {
                        use folio::collection::BuiltinColumn;
                        collection.hidden = vec![
                            BuiltinColumn::Priority,
                            BuiltinColumn::Due,
                            BuiltinColumn::Tags,
                        ];
                    }
                }
            }),
        ],
        ..AppSpec::new("Folio", (1200.0, 820.0), init, ui::view)
    });
}

#[cfg(test)]
mod block_drag_tests;
#[cfg(test)]
mod collection_tests;
#[cfg(test)]
mod completed_tasks_tests;
#[cfg(test)]
mod dashboard_tests;
#[cfg(test)]
mod feature_tests;
#[cfg(test)]
mod list_tests;
#[cfg(test)]
mod management_layout_tests;
#[cfg(test)]
mod overview_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod workspace_flow_tests;
#[cfg(test)]
mod workspace_tests;

#[cfg(test)]
mod rune_update_tests;

#[cfg(test)]
mod settings_tests;

#[cfg(test)]
mod module_library_tests;
