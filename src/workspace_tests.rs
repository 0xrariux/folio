use crate::state::{Panel, State};
use folio::collection::{Sort, Status, View};
use rune::{Env, FileDialogKind, Key, Runtime};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn app() -> Runtime<State> {
    let mut s = State::demo();
    s.select("demo-tracker");
    s.collection_view(View::Table);
    let c = s
        .pages
        .iter_mut()
        .find(|p| p.id == "demo-tracker")
        .unwrap()
        .collection
        .as_mut()
        .unwrap();
    c.views.clear();
    c.active_view = None;
    let mut rt = Runtime::new(s, crate::ui::view, (1200.0, 820.0), Env::new(false, 1.0));
    rt.frame(0.0);
    rt
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..40 {
        rt.frame(1.0 / 60.0);
    }
    let (x, y) = rt
        .find(key)
        .unwrap_or_else(|| panic!("Missing control {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
}
fn replace(rt: &mut Runtime<State>, key: &str, value: &str) {
    tap(rt, key);
    rt.key(Key::Command('a').into());
    rt.key(Key::Text(value.into()).into());
    rt.frame(0.0);
}
fn choose_export(rt: &mut Runtime<State>, path: &Path) {
    assert_eq!(rt.file_dialog().unwrap().0, FileDialogKind::Save);
    rt.resolve_file_dialog(Some(path.into())).unwrap();
    rt.frame(0.0);
    let start = Instant::now();
    while rt.state.export_busy && start.elapsed() < Duration::from_secs(3) {
        rt.update(|s, _| {
            s.poll_export();
        });
        rt.frame(0.0);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!rt.state.export_busy, "export timed out");
}

#[test]
fn named_views_switch_independent_filters_and_layouts_through_the_ui() {
    let mut rt = app();
    tap(&mut rt, "collection-views");
    replace(&mut rt, "view-name", "En mouvement");
    tap(&mut rt, "save-view");
    let c = rt.state.current().unwrap().collection.as_ref().unwrap();
    let all = c.views[0].id.clone();
    let moving = c.active_view.clone().unwrap();
    tap(&mut rt, "collection-filter");
    tap(&mut rt, "filter-InProgress");
    tap(&mut rt, "view-board");
    tap(&mut rt, "collection-sort");
    tap(&mut rt, "sort-Due");
    assert_eq!(rt.state.collection_items().len(), 2);
    tap(&mut rt, "collection-views");
    tap(&mut rt, &format!("saved-view-{all}"));
    assert_eq!(rt.state.collection_items().len(), 6);
    assert!(rt.find("collection-table").is_some());
    tap(&mut rt, "collection-views");
    tap(&mut rt, &format!("saved-view-{moving}"));
    let c = rt.state.current().unwrap().collection.as_ref().unwrap();
    assert_eq!(
        (c.view, c.sort, c.status),
        (View::Board, Sort::Due, Some(Status::InProgress))
    );
    assert_eq!(rt.state.collection_items().len(), 2);
    assert!(rt.find("collection-board").is_some());
    rt.update(|s, _| {
        s.select("demo-home");
        s.select("demo-tracker");
    });
    rt.frame(0.0);
    assert_eq!(
        rt.state
            .current()
            .unwrap()
            .collection
            .as_ref()
            .unwrap()
            .active_view
            .as_deref(),
        Some(moving.as_str())
    );
}

#[test]
fn view_form_validates_renames_and_removes_configuration_without_records() {
    let mut rt = app();
    let records = rt.state.pages.len();
    tap(&mut rt, "collection-views");
    tap(&mut rt, "save-view");
    assert!(matches!(&rt.state.panel, Panel::Views(f) if f.error.is_some()));
    replace(&mut rt, "view-name", "Ma vue");
    tap(&mut rt, "save-view");
    tap(&mut rt, "collection-views");
    tap(&mut rt, "rename-view");
    replace(&mut rt, "view-name", "Main view");
    tap(&mut rt, "save-view");
    assert!(matches!(&rt.state.panel, Panel::Views(f) if f.error.is_some()));
    replace(&mut rt, "view-name", "Mon suivi");
    tap(&mut rt, "save-view");
    assert_eq!(
        rt.state
            .current()
            .unwrap()
            .collection
            .as_ref()
            .unwrap()
            .active()
            .unwrap()
            .name,
        "Mon suivi"
    );
    tap(&mut rt, "collection-views");
    tap(&mut rt, "remove-view");
    assert_eq!(rt.state.pages.len(), records);
    assert_eq!(rt.state.collection_items().len(), 6);
    tap(&mut rt, "collection-views");
    assert!(rt.find("remove-view").is_none());
}

#[test]
fn saved_view_actions_stay_reachable_at_the_minimum_window_size() {
    let mut s = State::demo();
    s.demo_saved_views();
    let mut rt = Runtime::new(s, crate::ui::view, (920.0, 620.0), Env::new(false, 1.0));
    rt.frame(0.0);
    tap(&mut rt, "collection-views");
    replace(&mut rt, "view-name", "Petite fenêtre");
    tap(&mut rt, "save-view");
    assert!(matches!(rt.state.panel, Panel::Closed));
    assert_eq!(
        rt.state
            .current()
            .unwrap()
            .collection
            .as_ref()
            .unwrap()
            .active()
            .unwrap()
            .name,
        "Petite fenêtre"
    );
}

#[test]
fn markdown_export_cancels_cleanly_and_keeps_the_original_page_after_navigation() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("note.md");
    std::fs::write(&path, "Avant").unwrap();
    let mut rt = app();
    rt.update(|s, _| s.select("demo-home"));
    rt.frame(0.0);
    tap(&mut rt, "page-menu");
    tap(&mut rt, "export-markdown");
    assert_eq!(
        rt.file_dialog().unwrap().1.file_name.as_deref(),
        Some("Room for your ideas.md")
    );
    rt.resolve_file_dialog(None).unwrap();
    rt.frame(0.0);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "Avant");
    assert!(!rt.state.export_busy);
    tap(&mut rt, "page-menu");
    tap(&mut rt, "export-markdown");
    rt.update(|s, _| s.select("demo-ideas"));
    rt.frame(0.0);
    choose_export(&mut rt, &path);
    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.starts_with("# Room for your ideas"));
    assert!(!contents.starts_with("# Ideas"));
    assert!(
        rt.state
            .export_message
            .as_ref()
            .unwrap()
            .contains("Export saved")
    );
    assert!(rt.find("export-notice").is_some());
    tap(&mut rt, "dismiss-export");
    assert!(rt.state.export_message.is_none());
}

#[test]
fn csv_exports_follow_filters_and_all_records_respect_dashboard_sources_and_archives() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("export.csv");
    let mut rt = app();
    tap(&mut rt, "collection-filter");
    tap(&mut rt, "filter-Done");
    tap(&mut rt, "page-menu");
    tap(&mut rt, "export-csv-view");
    choose_export(&mut rt, &path);
    let filtered = std::fs::read_to_string(&path).unwrap();
    assert!(filtered.contains("demo-record-3"));
    assert!(!filtered.contains("demo-record-0"));
    tap(&mut rt, "page-menu");
    tap(&mut rt, "export-csv-all");
    choose_export(&mut rt, &path);
    let all = std::fs::read_to_string(&path).unwrap();
    for i in 0..6 {
        assert!(all.contains(&format!("demo-record-{i}")));
    }
    rt.update(|s, _| {
        s.select("demo-dashboard");
        s.dashboard_scope(Some("demo-launch"));
        s.pages
            .iter_mut()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .archived = true;
        s.tracking_status(Some(Status::Done));
    });
    rt.frame(0.0);
    tap(&mut rt, "page-menu");
    tap(&mut rt, "export-csv-all");
    choose_export(&mut rt, &path);
    let scoped = std::fs::read_to_string(&path).unwrap();
    assert!(scoped.contains("demo-record-1"));
    assert!(!scoped.contains("demo-record-0"));
    assert!(!scoped.contains("demo-launch-record"));
}

#[test]
fn failed_exports_report_the_error_and_allow_retry() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("missing/export.csv");
    let mut rt = app();
    tap(&mut rt, "page-menu");
    tap(&mut rt, "export-csv-all");
    choose_export(&mut rt, &path);
    assert!(
        rt.state
            .export_message
            .as_ref()
            .unwrap()
            .starts_with("Export failed")
    );
    let valid = temp.path().join("valid.csv");
    tap(&mut rt, "page-menu");
    tap(&mut rt, "export-csv-all");
    choose_export(&mut rt, &valid);
    assert!(valid.exists());
    assert!(
        rt.state
            .export_message
            .as_ref()
            .unwrap()
            .starts_with("Export saved")
    );
}
