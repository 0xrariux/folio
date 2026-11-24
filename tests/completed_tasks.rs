use folio::{
    document::{Kind, NoteBlock},
    model::Page,
    outline,
    storage::Store,
};

fn task(checked: bool, indent: u8) -> NoteBlock {
    let mut b = NoteBlock::text(Kind::Todo, "Task 🦀");
    b.checked = checked;
    b.indent = indent;
    b
}

#[test]
fn completion_uses_every_leaf_including_folded_tasks_and_keeps_non_task_context() {
    let mut blocks = vec![
        NoteBlock::text(Kind::ToggleHeading2, "Project"),
        task(true, 1),
        task(false, 1),
        NoteBlock::text(Kind::Text, "Notes"),
        task(true, 0),
    ];
    blocks[0].collapsed = true;
    // A stale parent flag must never hide an unfinished descendant.
    blocks[0].checked = true;
    blocks[3].checked = true;
    let sections = outline::task_sections(&blocks);
    assert_eq!(sections.active, [0, 1, 2, 3]);
    assert_eq!(sections.completed, [4]);
    blocks[2].checked = true;
    let sections = outline::task_sections(&blocks);
    assert_eq!(sections.active, [3]);
    assert_eq!(sections.completed, [0, 1, 2, 4]);
    assert_eq!(sections.completed_groups, 2);
    assert!(outline::task_sections([]).order().next().is_none());
}

#[test]
fn completion_sections_are_reconstructed_after_turso_reopen_without_changing_documents() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("completed.db");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut page = Page::new(None);
            page.blocks = vec![task(true, 0), task(true, 1), task(false, 0)];
            page.blocks[0].collapsed = true;
            page.blocks[1].html = "<p><strong>Preserved 🦀</strong></p>".into();
            page.sync_document();
            let original = serde_json::to_value(page.content()).unwrap();
            let store = Store::open(&path).await.unwrap();
            store.save(&page).await.unwrap();
            drop(store);
            let store = Store::open(&path).await.unwrap();
            let pages = store.load().await.unwrap();
            assert_eq!(pages, [page]);
            assert_eq!(serde_json::to_value(pages[0].content()).unwrap(), original);
            let sections = outline::task_sections(&pages[0].blocks);
            assert_eq!(sections.active, [2]);
            assert_eq!(sections.completed, [0, 1]);
            assert_eq!(sections.completed_groups, 1);
            assert_eq!(original["version"], 7);
        });
}
