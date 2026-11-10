use folio::{
    document::{Kind, NoteBlock, PageContent},
    model::Page,
    storage::Store,
};

#[test]
fn toggle_trees_restart_points_and_closed_state_survive_turso_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("lists.db");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut page = folio::templates::Template::Outline.create();
            page.blocks[1].collapsed = true;
            page.blocks[3].number_start = Some(7);
            page.blocks[4].html = "<p>Un détail <strong>important 🦀</strong></p>".into();
            page.sync_document();
            assert!(
                page.matches("important"),
                "collapsed content stays searchable"
            );
            let store = Store::open(&path).await.unwrap();
            store.save(&page).await.unwrap();
            drop(store);
            let store = Store::open(&path).await.unwrap();
            assert_eq!(store.load().await.unwrap(), vec![page.clone()]);
            assert_eq!(page.content().version, 6);
            assert!(page.body_html.contains("<details><summary><h2>"));
            assert!(page.body_html.contains("<ol start=\"7\">"));
        });
}

#[test]
fn version_four_lists_load_with_safe_defaults_and_keep_ids() {
    let mut page = Page::new(None);
    page.blocks = vec![
        NoteBlock::text(Kind::Numbered, "Première"),
        NoteBlock::text(Kind::Bullet, "Enfant"),
    ];
    page.blocks[1].indent = 1;
    let mut old = serde_json::to_value(page.content()).unwrap();
    old["version"] = 4.into();
    for b in old["blocks"].as_array_mut().unwrap() {
        b.as_object_mut().unwrap().remove("collapsed");
        b.as_object_mut().unwrap().remove("number_start");
    }
    let loaded: PageContent = serde_json::from_value(old).unwrap();
    assert_eq!(loaded.blocks, page.blocks);
    assert!(!loaded.blocks[0].collapsed);
    assert!(loaded.blocks[0].number_start.is_none());
}

#[test]
fn markdown_export_keeps_toggle_hierarchy_rich_text_tasks_and_six_levels() {
    let mut page = folio::templates::Template::Outline.create();
    page.blocks[4].html = "<p>Une <strong>idée 🦀</strong></p>".into();
    let mut deep = NoteBlock::text(Kind::Todo, "Profondeur six");
    deep.indent = 6;
    let mut numbered = NoteBlock::text(Kind::Numbered, "Séquence à cinq");
    numbered.number_start = Some(5);
    page.blocks
        .extend([deep, numbered, NoteBlock::text(Kind::Numbered, "Suite")]);
    let result = folio::export::markdown(&page, &[]).contents;
    assert!(
        result.contains("<details open><summary><h2>Un projet, étape par étape</h2></summary>")
    );
    assert!(result.contains("<strong>idée 🦀</strong>"));
    assert!(result.contains("<input type=\"checkbox\" disabled checked> Poser les fondations"));
    assert_eq!(result.matches("<details").count(), 3);
    assert_eq!(result.matches("</details>").count(), 3);
    assert!(result.contains(&format!("{}- [ ] Profondeur six", "    ".repeat(6))));
    assert!(result.contains("5. Séquence à cinq"));
    assert!(result.contains("6. Suite"));
    assert_eq!(result.matches("Rassembler les idées").count(), 0);
    assert_eq!(result.matches("Construire et partager").count(), 1);
}
