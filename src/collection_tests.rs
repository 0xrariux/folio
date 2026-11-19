use crate::{
    editor::input_key,
    state::{Panel, State},
};
use folio::collection::{Priority, Sort, Status, View};
use rune::{Env, Key, Runtime};

fn app() -> Runtime<State> {
    let mut state = State::demo();
    state.select("demo-tracker");
    let mut rt = Runtime::new(
        state,
        crate::ui::view,
        (1200.0, 820.0),
        Env::new(false, 1.0),
    );
    rt.frame(0.0);
    rt
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    let (x, y) = rt
        .find(key)
        .unwrap_or_else(|| panic!("Missing control: {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
}
fn type_text(rt: &mut Runtime<State>, value: &str) {
    rt.key(Key::Text(value.into()).into());
    rt.frame(0.0);
}
fn replace(rt: &mut Runtime<State>, key: &str, value: &str) {
    tap(rt, key);
    rt.key(Key::Command('a').into());
    type_text(rt, value);
}
fn reveal(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..35 {
        rt.frame(1.0 / 60.0);
    }
}

#[test]
fn table_and_board_show_the_same_pages_and_status_updates() {
    let mut rt = app();
    assert!(rt.find("collection-board").is_some());
    tap(&mut rt, "view-table");
    assert!(rt.find("collection-table").is_some());
    assert_eq!(rt.state.collection_items().len(), 6);
    tap(&mut rt, "status-demo-record-0");
    tap(&mut rt, "set-status-InProgress");
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .properties
            .status,
        Status::InProgress
    );
    tap(&mut rt, "view-board");
    assert!(rt.find("card-demo-record-0").is_some());
    reveal(&mut rt, "board-forward-demo-record-0");
    tap(&mut rt, "board-forward-demo-record-0");
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .properties
            .status,
        Status::Done
    );
    assert_eq!(
        rt.state
            .current()
            .unwrap()
            .collection
            .as_ref()
            .unwrap()
            .view,
        View::Board
    );
}

#[test]
fn records_are_real_editable_pages_and_the_parent_breadcrumb_returns_to_the_database() {
    let mut rt = app();
    tap(&mut rt, "new-record");
    let id = rt.state.selected.clone().unwrap();
    replace(&mut rt, "page-title", "Une fiche avec des notes");
    let key = input_key(&rt.state.blocks[0].data.id);
    tap(&mut rt, &key);
    type_text(&mut rt, "Le contenu de cette fiche.");
    assert!(rt.find("record-properties").is_some());
    tap(&mut rt, "breadcrumb-demo-tracker");
    assert_eq!(rt.state.selected.as_deref(), Some("demo-tracker"));
    assert_eq!(rt.state.collection_items().len(), 7);
    reveal(&mut rt, &format!("record-{id}"));
    tap(&mut rt, &format!("record-{id}"));
    assert_eq!(rt.state.title.text, "Une fiche avec des notes");
    assert_eq!(rt.state.blocks[0].edit.text, "Le contenu de cette fiche.");
}

#[test]
fn collection_filters_sorts_and_tag_search_apply_to_both_views() {
    let mut rt = app();
    tap(&mut rt, "collection-filter");
    tap(&mut rt, "filter-Done");
    assert_eq!(rt.state.collection_items().len(), 1);
    tap(&mut rt, "view-table");
    assert_eq!(rt.state.collection_items().len(), 1);
    tap(&mut rt, "collection-filter");
    tap(&mut rt, "filter-all");
    tap(&mut rt, "collection-sort");
    tap(&mut rt, "sort-Priority");
    assert_eq!(
        rt.state.collection_items()[0].properties.priority,
        Priority::High
    );
    tap(&mut rt, "collection-sort");
    tap(&mut rt, "sort-Due");
    assert_eq!(rt.state.collection_items()[0].id, "demo-record-5");
    replace(&mut rt, "collection-search", "DESIGN");
    assert_eq!(rt.state.collection_items().len(), 1);
    assert_eq!(rt.state.collection_items()[0].id, "demo-record-1");
    assert_eq!(
        rt.state
            .current()
            .unwrap()
            .collection
            .as_ref()
            .unwrap()
            .sort,
        Sort::Due
    );
}

#[test]
fn property_form_validates_dates_and_cancel_keeps_original_values() {
    let mut rt = app();
    tap(&mut rt, "view-table");
    let original = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-record-0")
        .unwrap()
        .properties
        .clone();
    tap(&mut rt, "details-demo-record-0");
    replace(&mut rt, "property-due", "2026-02-30");
    tap(&mut rt, "save-properties");
    assert!(matches!(&rt.state.panel,Panel::Properties(form) if form.error.is_some()));
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .properties,
        original
    );
    replace(&mut rt, "property-due", "2028-02-29");
    replace(&mut rt, "property-tags", "Rust, rust, Produit");
    tap(&mut rt, "save-properties");
    let saved = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-record-0")
        .unwrap()
        .properties
        .clone();
    assert_eq!(saved.due.as_deref(), Some("2028-02-29"));
    assert_eq!(saved.tags, vec!["Rust", "Produit"]);
    tap(&mut rt, "details-demo-record-0");
    replace(&mut rt, "property-due", "");
    tap(&mut rt, "cancel-properties");
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .properties,
        saved
    );
}

#[test]
fn slash_page_links_navigate_and_create_automatic_backlinks() {
    let mut rt = app();
    tap(&mut rt, "new-page");
    let source = rt.state.selected.clone().unwrap();
    let block = rt.state.blocks[0].data.id.clone();
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    type_text(&mut rt, "/link");
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    assert!(
        rt.find("link-picker").is_some(),
        "text={:?}, slash={}, link={}, candidates={:?}",
        rt.state.blocks[0].edit.text,
        matches!(rt.state.panel, Panel::Slash { .. }),
        matches!(rt.state.panel, Panel::Link(_)),
        rt.state.slash_options()
    );
    type_text(&mut rt, "ideas");
    assert!(
        matches!(&rt.state.panel,Panel::Link(form) if form.typed && form.query.text == "ideas")
    );
    reveal(&mut rt, "link-target-demo-ideas");
    tap(&mut rt, "link-target-demo-ideas");
    assert_eq!(
        rt.state.current().unwrap().blocks[0].target_id.as_deref(),
        Some("demo-ideas"),
        "link panel={}, blocks={:?}",
        matches!(rt.state.panel, Panel::Link(_)),
        rt.state.current().unwrap().blocks
    );
    assert_eq!(rt.state.backlinks("demo-ideas").len(), 1);
    tap(&mut rt, &format!("page-link-{block}"));
    assert_eq!(rt.state.selected.as_deref(), Some("demo-ideas"));
    tap(&mut rt, &format!("backlink-{source}"));
    assert_eq!(rt.state.selected.as_deref(), Some(source.as_str()));
    rt.update(|s, _| s.select("demo-ideas"));
    rt.frame(0.0);
    replace(&mut rt, "page-title", "Idées de demain");
    assert!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == source)
            .unwrap()
            .plain_text
            .contains("Idées de demain")
    );
    rt.update(|s, _| s.select(&source));
    rt.frame(0.0);
    assert_eq!(rt.state.blocks[0].edit.text, "Idées de demain");
}

#[test]
fn new_collection_and_tracker_template_start_without_records() {
    let mut rt = app();
    tap(&mut rt, "sidebar-more");
    tap(&mut rt, "new-collection");
    assert!(rt.state.current().unwrap().collection.is_some());
    assert!(rt.state.collection_items().is_empty());
    tap(&mut rt, "sidebar-more");
    tap(&mut rt, "templates");
    tap(&mut rt, "template-3");
    assert_eq!(rt.state.current().unwrap().title, "Project tracker");
    assert!(rt.state.collection_items().is_empty());
    tap(&mut rt, "add-InProgress");
    assert_eq!(
        rt.state.current().unwrap().properties.status,
        Status::InProgress
    );
}
