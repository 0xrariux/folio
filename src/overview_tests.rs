use crate::{
    editor::input_key,
    state::{Phase, State},
};
use folio::{
    document::{Kind, NoteBlock},
    model::Page,
};
use rune::{Env, Key, Runtime, TextEdit};

fn note(id: &str, title: &str, blocks: Vec<NoteBlock>) -> Page {
    let mut page = Page::new(None);
    page.id = id.into();
    page.title = title.into();
    page.blocks = blocks;
    for (i, b) in page.blocks.iter_mut().enumerate() {
        b.id = format!("{id}-{i}");
    }
    page.sync_document();
    page
}
fn app(pages: Vec<Page>) -> Runtime<State> {
    let mut state = State::new();
    state.demo = true;
    state.phase = Phase::Ready;
    let first = pages[0].id.clone();
    state.pages = pages;
    state.select(&first);
    let mut rt = Runtime::new(state, crate::ui::view, (920., 620.), Env::new(false, 1.));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.);
    rt
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..35 {
        rt.frame(1. / 60.);
    }
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("Missing {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.);
}
fn pages() -> Vec<Page> {
    vec![
        note(
            "work",
            "Work",
            vec![
                NoteBlock::text(Kind::Todo, "Send the draft"),
                NoteBlock::text(Kind::Bullet, "A useful idea"),
            ],
        ),
        note(
            "home",
            "Home",
            vec![
                NoteBlock::text(Kind::Todo, "Buy groceries"),
                NoteBlock::text(Kind::Numbered, "Prepare dinner"),
            ],
        ),
    ]
}

#[test]
fn overview_is_a_shortcut_view_that_never_adds_a_page_and_returns_to_notes() {
    let mut rt = app(pages());
    let before = rt.state.pages.clone();
    let history = rt.state.history.clone();
    for _ in 0..3 {
        tap(&mut rt, "open-dashboard");
        assert!(rt.state.overview.open);
        assert!(rt.state.selected.is_none());
        assert!(rt.find("notebook-overview").is_some());
        assert_eq!(rt.state.pages, before);
        assert_eq!(rt.state.notebook_overview().tasks.len(), 2);
        assert_eq!(rt.state.notebook_overview().lists.len(), 2);
    }
    tap(&mut rt, "all-pages");
    assert!(!rt.state.overview.open);
    assert_eq!(rt.state.selected.as_deref(), Some("work"));
    tap(&mut rt, "open-dashboard");
    rt.key(Key::Escape.into());
    rt.frame(0.);
    assert!(
        rt.find("notebook-overview").is_some(),
        "closing a menu does not leave Overview"
    );
    tap(&mut rt, "overview-back");
    assert_eq!(rt.state.pages, before);
    assert_eq!(rt.state.selected.as_deref(), Some("work"));
    assert_eq!(rt.state.history, history);
}

#[test]
fn overview_counts_leaf_tasks_folded_lists_and_excludes_archived_and_pin_unlocked_notes() {
    let mut notes = pages();
    notes[0].blocks[0].collapsed = true;
    let mut child = NoteBlock::text(Kind::Todo, "Nested task");
    child.id = "nested".into();
    child.indent = 1;
    child.checked = true;
    notes[0].blocks.insert(1, child);
    notes[0].sync_document();
    let mut archived = note(
        "archived",
        "Archived",
        vec![NoteBlock::text(Kind::Todo, "Do not show archived")],
    );
    archived.archived = true;
    notes.push(archived);
    notes.push(note(
        "private",
        "Private",
        vec![NoteBlock::text(Kind::Todo, "Hidden private task")],
    ));
    let mut rt = app(notes);
    rt.state.settings.unlocked.insert("private".into());
    let data = rt.state.notebook_overview();
    assert_eq!(data.notes.len(), 2);
    assert_eq!(
        data.tasks.len(),
        2,
        "the nested parent is not counted twice"
    );
    assert_eq!(data.done, 1);
    assert_eq!(data.lists.len(), 2);
    assert_eq!(data.tasks[0].block.id, "nested");
    rt.state.overview.search = TextEdit::new("groceries");
    let data = rt.state.notebook_overview();
    assert_eq!(data.tasks.len(), 1);
    assert_eq!(data.notes.len(), 1);
    assert!(data.lists.is_empty());
    rt.state.settings.locked = true;
    assert!(rt.state.notebook_overview().tasks.is_empty());
    assert!(rt.state.notebook_overview().notes.is_empty());
}

#[test]
fn overview_filters_navigates_to_folded_task_sources_and_refreshes_completion() {
    let mut notes = pages();
    notes[0].blocks[0].kind = Kind::Toggle;
    notes[0].blocks[0].collapsed = true;
    notes[0].blocks[1].kind = Kind::Todo;
    notes[0].blocks[1].indent = 1;
    notes[0].sync_document();
    let mut rt = app(notes);
    tap(&mut rt, "open-dashboard");
    assert!(rt.find("overview-item-work-1").is_some());
    tap(&mut rt, "overview-filter-done");
    assert!(rt.find("overview-item-work-1").is_none());
    tap(&mut rt, "overview-filter-all");
    tap(&mut rt, "overview-item-work-1");
    assert!(!rt.state.overview.open);
    assert_eq!(rt.state.selected.as_deref(), Some("work"));
    assert!(!rt.state.blocks[0].data.collapsed);
    assert_eq!(rt.state.active.as_deref(), Some("work-1"));
    assert!(rt.find(&input_key("work-1")).is_some());
    tap(&mut rt, "check-work-1");
    tap(&mut rt, "open-dashboard");
    assert_eq!(rt.state.notebook_overview().done, 1);
    tap(&mut rt, "overview-filter-done");
    assert!(rt.find("overview-item-work-1").is_some());
    assert!(rt.find("overview-item-home-0").is_none());
}

#[test]
fn legacy_blank_overview_is_hidden_from_the_tree_without_deleting_or_hiding_custom_dashboards() {
    let mut legacy = Page::new(None);
    legacy.id = "legacy".into();
    legacy.title = "Overview".into();
    legacy.emoji = "📊".into();
    legacy.wide = true;
    legacy.dashboard = Some(Default::default());
    legacy.dashboard.as_mut().unwrap().filter.status = Some(folio::collection::Status::Done);
    let mut custom = folio::templates::Template::Dashboard.create();
    custom.id = "custom".into();
    let mut populated = legacy.clone();
    populated.id = "populated".into();
    populated.replace_html("<p>Keep this content</p>");
    let mut notes = pages();
    notes.extend([legacy.clone(), custom, populated]);
    let mut rt = app(notes);
    let original = rt.state.pages.clone();
    assert!(rt.find("page-label-legacy").is_none());
    assert!(
        rt.state
            .sidebar_pages()
            .iter()
            .any(|(p, _)| p.id == "custom")
    );
    assert!(
        rt.state
            .sidebar_pages()
            .iter()
            .any(|(p, _)| p.id == "populated")
    );
    tap(&mut rt, "open-dashboard");
    assert_eq!(rt.state.pages, original);
    rt.state.search = TextEdit::new("Overview");
    assert!(
        rt.state
            .sidebar_pages()
            .iter()
            .any(|(p, _)| p.id == "legacy")
    );
    assert_eq!(
        rt.state.pages.iter().find(|p| p.id == "legacy"),
        Some(&legacy)
    );
}

#[test]
fn overview_batched_results_can_be_searched_and_loaded_without_creating_pages() {
    let blocks = (0..65)
        .map(|i| NoteBlock::text(Kind::Todo, &format!("Task number {i}")))
        .collect();
    let mut rt = app(vec![note("large", "Large notebook", blocks)]);
    tap(&mut rt, "open-dashboard");
    assert!(rt.find("overview-item-large-39").is_some());
    assert!(rt.find("overview-item-large-40").is_none());
    tap(&mut rt, "overview-more");
    assert!(rt.find("overview-item-large-64").is_some());
    tap(&mut rt, "overview-search");
    rt.key(Key::Text("number 64".into()).into());
    rt.frame(0.);
    assert_eq!(rt.state.overview.limit, 40);
    assert!(rt.find("overview-item-large-64").is_some());
    assert!(rt.find("overview-item-large-0").is_none());
    assert_eq!(rt.state.pages.len(), 1);
}

#[test]
fn startup_and_late_preferences_do_not_reopen_a_legacy_stub_or_interrupt_overview() {
    let mut legacy = Page::new(None);
    legacy.id = "legacy".into();
    legacy.title = "Overview".into();
    legacy.emoji = "📊".into();
    legacy.wide = true;
    legacy.dashboard = Some(Default::default());
    let mut source = vec![legacy];
    source.extend(pages());
    let mut state = State::new();
    state.demo = true;
    state.accept(crate::worker::Event::Loaded(source.clone()));
    assert_eq!(state.selected.as_deref(), Some("work"));
    let mut prefs = folio::workspace::Preferences {
        last_page: Some("legacy".into()),
        ..Default::default()
    };
    state.accept_workspace(Ok(crate::workspace_worker::Response::Preferences(
        prefs.clone(),
    )));
    assert_eq!(state.selected.as_deref(), Some("work"));
    state.open_dashboard();
    prefs.last_page = Some("home".into());
    state.accept_workspace(Ok(crate::workspace_worker::Response::Preferences(prefs)));
    assert!(state.overview.open);
    state.select("missing");
    assert!(state.overview.open);
    assert_eq!(state.pages, source);
}
