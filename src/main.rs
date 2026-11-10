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
mod navigation;
mod notebook;
mod saved_views;
mod selection;
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
    let mut changed = state.poll_export() || !events.is_empty();
    if !state.demo {
        let today = folio::dashboard::local_today();
        if today != state.today {
            state.today = today;
            changed = true;
        }
        tx.wake_after(std::time::Duration::from_secs(60));
    }
    for event in events {
        state.accept(event);
    }
    if changed {
        tx.instant();
    }
    changed
}

fn on_key(state: &mut State, event: &KeyEvent, tx: &mut Tx) -> bool {
    if state.workspace.busy {
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
        Key::Escape => state.panel = state::Panel::Closed,
        _ => return false,
    }
    true
}

fn main() {
    rune::run(AppSpec {
        min_size: Some((920.0, 620.0)),
        setup: Some(setup),
        poll: Some(poll),
        on_key: Some(on_key),
        scenes: &[
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
mod dashboard_tests;
#[cfg(test)]
mod feature_tests;
#[cfg(test)]
mod list_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod workspace_flow_tests;
#[cfg(test)]
mod workspace_tests;
