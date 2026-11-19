use rune::{Env, Key, KeyEvent, Mark, Runtime};

use crate::{
    state::{Filter, Pending, State},
    worker::Event,
};

fn runtime() -> Runtime<State> {
    let mut rt = Runtime::new(
        State::demo(),
        crate::ui::view,
        (1200.0, 820.0),
        Env::new(false, 1.0),
    );
    rt.frame(0.0);
    rt
}

fn tap(rt: &mut Runtime<State>, key: &str) {
    // The notebook scrolls independently of the fixed Library/Settings footer.
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..35 {
        rt.frame(1.0 / 60.0);
    }
    let (x, y) = rt
        .find(key)
        .unwrap_or_else(|| panic!("Missing control: {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
}

#[test]
fn create_edit_format_and_switch_pages_through_the_real_ui() {
    let mut rt = runtime();
    let original = rt.state.selected.clone().unwrap();
    let count = rt.state.pages.len();
    tap(&mut rt, "new-page");
    assert_eq!(rt.state.pages.len(), count + 1);
    let created = rt.state.selected.clone().unwrap();
    assert_ne!(created, original);
    // Route typing through the focused editor, as a native key event does.
    rt.key(KeyEvent::from(Key::Command('a')));
    rt.key(KeyEvent::from(Key::Text("Mes notes".into())));
    rt.frame(0.0);
    assert_eq!(rt.state.current().unwrap().title, "Mes notes");
    let first = crate::editor::input_key(&rt.state.blocks[0].data.id);
    tap(&mut rt, &first);
    rt.key(KeyEvent::from(Key::Text("Une idée".into())));
    rt.key(KeyEvent::from(Key::Enter));
    rt.frame(0.0);
    rt.key(KeyEvent::from(Key::Text("Une autre".into())));
    rt.frame(0.0);
    assert!(
        rt.state
            .current()
            .unwrap()
            .plain_text
            .contains("Une idée\nUne autre")
    );
    rt.key(KeyEvent::from(Key::Command('a')));
    tap(&mut rt, "bold");
    assert!(rt.state.current().unwrap().body_html.contains("<strong>"));
    tap(&mut rt, &format!("page-{original}"));
    tap(&mut rt, &format!("page-{created}"));
    assert_eq!(rt.state.title.text, "Mes notes");
    assert!(rt.state.current().unwrap().body_html.contains("<strong>"));
}

#[test]
fn archived_parent_does_not_hide_its_children_and_search_reads_content() {
    let mut state = State::demo();
    let parent = state.pages[1].id.clone();
    let child = state.pages[2].id.clone();
    state.select(&parent);
    state.archive();
    assert!(
        state
            .sidebar_pages()
            .iter()
            .any(|(page, depth)| page.id == child && *depth == 0)
    );
    state.filter = Filter::Archive;
    assert_eq!(state.sidebar_pages().len(), 1);
    state.archive();
    state.filter = Filter::All;
    state.search.text = "EXPLORING".into();
    assert_eq!(
        state
            .sidebar_pages()
            .iter()
            .map(|(page, _)| &page.id)
            .collect::<Vec<_>>(),
        vec![&parent]
    );
}

#[test]
fn late_acknowledgements_do_not_claim_newer_edits_are_saved() {
    let mut state = State::demo();
    let id = state.selected.clone().unwrap();
    state.pending.insert(
        id.clone(),
        Pending {
            revision: 2,
            error: None,
        },
    );
    state.accept(Event::Saved {
        id: id.clone(),
        revision: 1,
    });
    assert!(state.pending.contains_key(&id));
    state.accept(Event::SaveFailed {
        id: id.clone(),
        revision: 2,
        message: "Disque plein".into(),
    });
    assert_eq!(state.pending[&id].error.as_deref(), Some("Disque plein"));
    state.accept(Event::Saved {
        id: id.clone(),
        revision: 2,
    });
    assert!(!state.pending.contains_key(&id));
}

#[test]
fn worker_drains_pending_writes_before_shutdown() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("notes.db");
    let worker = crate::worker::Worker::start(path.clone(), std::sync::Arc::new(|| {}));
    assert!(matches!(
        worker
            .events
            .recv_timeout(std::time::Duration::from_secs(15))
            .unwrap(),
        Event::Loaded(_)
    ));
    let mut page = folio::model::Page::new(None);
    for index in 0..20 {
        page.title = format!("Révision {index}");
        worker.save(page.clone(), index).unwrap();
    }
    drop(worker);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let store = folio::storage::Store::open(&path).await.unwrap();
        let pages = store.load().await.unwrap();
        assert_eq!(
            pages.iter().find(|p| p.id == page.id).unwrap().title,
            "Révision 19"
        );
    });
}

#[test]
fn formatted_documents_remain_searchable() {
    let mut state = State::demo();
    let id = state.blocks[0].data.id.clone();
    let edit = state.editor_mut(&id);
    *edit = rune::TextEdit::new("Une note").rich();
    edit.set_caret(0);
    edit.toggle_mark(Mark::Italic);
    edit.insert("Idée : ");
    state.block_changed(&id);
    let page = state.current().unwrap();
    assert!(page.body_html.contains("<em>"));
    assert!(page.matches("IDÉE"));
}

#[test]
fn typing_in_the_app_is_saved_and_restored_by_turso() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("app.db");
    let mut state = State::new();
    state.worker = Some(crate::worker::Worker::start(
        path.clone(),
        std::sync::Arc::new(|| {}),
    ));
    let mut rt = Runtime::new(
        state,
        crate::ui::view,
        (1200.0, 820.0),
        Env::new(false, 1.0),
    );
    rt.frame(0.0);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while matches!(rt.state.phase, crate::state::Phase::Loading) {
        assert!(
            std::time::Instant::now() < deadline,
            "The database did not open"
        );
        rt.update_if(|state, tx| crate::poll(state, 0.0, tx));
        rt.frame(0.0);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(matches!(rt.state.phase, crate::state::Phase::Ready));
    tap(&mut rt, "new-page");
    rt.key(Key::Command('a').into());
    rt.key(Key::Text("Un carnet durable".into()).into());
    let first = crate::editor::input_key(&rt.state.blocks[0].data.id);
    tap(&mut rt, &first);
    rt.key(Key::Text("Retrouver cette idée demain.".into()).into());
    rt.frame(0.0);
    let saved = rt.state.current().unwrap().clone();
    assert!(!rt.state.pending.is_empty());
    drop(rt);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let store = folio::storage::Store::open(&path).await.unwrap();
        let pages = store.load().await.unwrap();
        assert_eq!(pages.iter().find(|page| page.id == saved.id), Some(&saved));
        assert_eq!(saved.plain_text, "Retrouver cette idée demain.");
    });
}
