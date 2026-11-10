use crate::state::{Panel, State};
use folio::{
    collection::{Status, View},
    dashboard::{Deadline, Summary, field_summaries},
    fields::CellValue,
};
use rune::{Env, Key, Runtime};

fn app(global: bool) -> Runtime<State> {
    let mut state = State::demo();
    state.select(if global {
        "demo-dashboard"
    } else {
        "demo-tracker"
    });
    if !global {
        state.collection_view(View::Dashboard);
    }
    let mut rt = Runtime::new(
        state,
        crate::ui::view,
        (1200.0, 820.0),
        Env::new(false, 1.0),
    );
    rt.frame(0.0);
    rt
}
fn reveal(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..40 {
        rt.frame(1.0 / 60.0);
    }
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    reveal(rt, key);
    let (x, y) = rt
        .find(key)
        .unwrap_or_else(|| panic!("Missing control: {key}"));
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
fn summary(rt: &Runtime<State>) -> Summary {
    Summary::from_pages(&rt.state.tracking_items(), &rt.state.today)
}

#[test]
fn dashboard_metrics_filter_the_same_fiches_as_the_table_and_reset_cleanly() {
    let mut rt = app(false);
    assert!(rt.find("tracking-dashboard").is_some());
    assert_eq!(
        (
            summary(&rt).total,
            summary(&rt).done,
            summary(&rt).overdue,
            summary(&rt).soon
        ),
        (6, 1, 1, 2)
    );
    tap(&mut rt, "metric-late");
    assert_eq!(rt.state.tracking_items().len(), 1);
    assert_eq!(rt.state.tracking_items()[0].id, "demo-record-5");
    tap(&mut rt, "metric-late");
    assert_eq!(rt.state.tracking_items().len(), 6);
    tap(&mut rt, "chart-status-Done");
    assert_eq!(rt.state.tracking_items().len(), 1);
    tap(&mut rt, "view-table");
    assert_eq!(rt.state.collection_items()[0].id, "demo-record-3");
    tap(&mut rt, "view-dashboard");
    tap(&mut rt, "reset-tracking");
    assert_eq!(summary(&rt).total, 6);
}

#[test]
fn custom_number_cells_validate_cancel_rename_and_feed_live_totals() {
    let mut rt = app(false);
    tap(&mut rt, "view-table");
    tap(&mut rt, "collection-columns");
    replace(&mut rt, "column-name", "Score");
    tap(&mut rt, "column-kind-Number");
    tap(&mut rt, "save-column");
    let id = rt
        .state
        .current()
        .unwrap()
        .collection
        .as_ref()
        .unwrap()
        .columns
        .last()
        .unwrap()
        .id
        .clone();
    let cell = format!("cell-demo-record-0-{id}");
    tap(&mut rt, &cell);
    replace(&mut rt, "cell-value", "NaN");
    tap(&mut rt, "save-cell");
    assert!(matches!(&rt.state.panel,Panel::Cell(form)if form.error.is_some()));
    replace(&mut rt, "cell-value", "12,5");
    tap(&mut rt, "save-cell");
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .properties
            .custom
            .get(&id),
        Some(&CellValue::Number("12.5".into()))
    );
    tap(&mut rt, &cell);
    replace(&mut rt, "cell-value", "90");
    tap(&mut rt, "cancel-cell");
    tap(&mut rt, "collection-columns");
    tap(&mut rt, &format!("rename-column-{id}"));
    replace(&mut rt, "column-name", "Score acquis");
    tap(&mut rt, "save-column");
    tap(&mut rt, "view-dashboard");
    let fields = field_summaries(&rt.state.tracking_sources(), &rt.state.tracking_items());
    let field = fields.iter().find(|f| f.column_id == id).unwrap();
    assert_eq!(field.label, "Score acquis");
    assert_eq!(field.sum, Some(12.5));
}

#[test]
fn global_dashboard_scope_is_persistent_and_empty_selection_is_unambiguous() {
    let mut rt = app(true);
    assert_eq!(summary(&rt).total, 9);
    assert_eq!(rt.state.tracking_sources().len(), 2);
    tap(&mut rt, "dashboard-sources");
    tap(&mut rt, "source-demo-launch");
    assert_eq!(rt.state.tracking_sources().len(), 1);
    assert_eq!(summary(&rt).total, 6);
    tap(&mut rt, "source-demo-tracker");
    assert_eq!(summary(&rt).total, 0);
    assert!(rt.state.tracking_sources().is_empty());
    tap(&mut rt, "sources-all");
    tap(&mut rt, "close-sources");
    assert_eq!(summary(&rt).total, 9);
    rt.update(|s, _| {
        s.pages
            .iter_mut()
            .find(|p| p.id == "demo-launch")
            .unwrap()
            .archived = true
    });
    rt.frame(0.0);
    assert_eq!(summary(&rt).total, 6);
}

#[test]
fn changing_values_status_and_the_day_updates_metrics_without_a_cached_snapshot() {
    let mut rt = app(false);
    tap(&mut rt, "view-table");
    tap(&mut rt, "cell-demo-record-0-demo-approved");
    let fields = field_summaries(&rt.state.tracking_sources(), &rt.state.tracking_items());
    assert_eq!(
        fields
            .iter()
            .find(|f| f.column_id == "demo-approved")
            .unwrap()
            .checked,
        2
    );
    tap(&mut rt, "status-demo-record-5");
    tap(&mut rt, "set-status-Done");
    tap(&mut rt, "view-dashboard");
    assert_eq!(summary(&rt).done, 2);
    assert_eq!(summary(&rt).overdue, 0);
    rt.update(|s, _| {
        s.set_status("demo-record-5", Status::Todo);
        s.today = "2026-11-15".into();
    });
    rt.frame(0.0);
    assert_eq!(summary(&rt).overdue, 2);
}

#[test]
fn hiding_a_column_keeps_its_values_and_combined_filters_work() {
    let mut rt = app(false);
    tap(&mut rt, "collection-columns");
    tap(&mut rt, "toggle-column-demo-hours");
    tap(&mut rt, "close-columns");
    assert!(
        field_summaries(&rt.state.tracking_sources(), &rt.state.tracking_items())
            .iter()
            .all(|f| f.column_id != "demo-hours")
    );
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .properties
            .custom
            .get("demo-hours"),
        Some(&CellValue::Number("8".into()))
    );
    rt.update(|s, _| {
        s.tracking_status(Some(Status::Todo));
        s.tracking_deadline(Deadline::Overdue);
    });
    rt.frame(0.0);
    assert_eq!(summary(&rt).total, 1);
    tap(&mut rt, "collection-columns");
    tap(&mut rt, "toggle-column-demo-hours");
    tap(&mut rt, "close-columns");
    assert_eq!(
        field_summaries(&rt.state.tracking_sources(), &rt.state.tracking_items())
            .iter()
            .find(|f| f.column_id == "demo-hours")
            .unwrap()
            .sum,
        Some(4.0)
    );
}
