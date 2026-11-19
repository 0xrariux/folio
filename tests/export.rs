use folio::{
    collection::Collection,
    document::{Kind, NoteBlock},
    export::{csv, file_name, markdown, write_atomic},
    fields::{CellValue, Column, ColumnKind},
    model::Page,
};

#[test]
fn markdown_preserves_blocks_unicode_formatting_code_and_link_identity() {
    let mut page = Page::new(None);
    page.title = "Carnet [été]".into();
    let mut rich = NoteBlock::text(Kind::Text, "");
    rich.html = "<p>Une <strong>idée 🦀</strong> et <u>soulignée</u> &amp; *texte*.</p>".into();
    let mut todo = NoteBlock::text(Kind::Todo, "Étape");
    todo.checked = true;
    todo.indent = 1;
    let code = NoteBlock::text(Kind::Code, "let x = \"```\";\n🦀");
    let mut target = Page::new(None);
    target.id = "cible-été".into();
    target.title = "Titre actualisé".into();
    let mut link = NoteBlock::text(Kind::PageLink, "Ancien titre");
    link.target_id = Some(target.id.clone());
    page.blocks = vec![
        rich,
        todo,
        code,
        link,
        NoteBlock::new(Kind::Divider),
        NoteBlock::text(Kind::Quote, "Ligne 1\nLigne 2"),
    ];
    let result = markdown(&page, &[target]);
    assert!(result.contents.starts_with("# Carnet \\[été\\]\n"));
    assert!(result.contents.contains("<strong>idée 🦀</strong>"));
    assert!(
        result
            .contents
            .contains("<u>soulignée</u> &amp; \\*texte\\*\\.")
    );
    assert!(result.contents.contains("    - [x] Étape"));
    assert!(result.contents.contains("````\nlet x = \"```\";\n🦀\n````"));
    assert!(
        result
            .contents
            .contains("[Titre actualisé](folio://page/cible-%C3%A9t%C3%A9)")
    );
    assert!(result.contents.contains("> Ligne 1<br>\n> Ligne 2"));
    assert_eq!(file_name("../été:/\\", "md"), "_été___.md");
}

#[test]
fn csv_quotes_multiline_cells_and_keeps_hidden_columns_and_full_numbers() {
    let mut base = Page::new(None);
    base.title = "Suivi".into();
    let mut number = Column::new("Budget", ColumnKind::Number);
    number.visible = false;
    let comment = Column::new("Retour", ColumnKind::Text);
    let check = Column::new("Approved", ColumnKind::Checkbox);
    base.collection = Some(Collection {
        columns: vec![number.clone(), comment.clone(), check.clone()],
        ..Default::default()
    });
    let mut page = Page::new(Some(base.id.clone()));
    page.id = "record".into();
    page.title = "Titre, avec \"guillemets\" 🦀".into();
    page.properties
        .custom
        .insert(number.id, CellValue::Number("123.456789012345".into()));
    page.properties
        .custom
        .insert(comment.id, CellValue::Text("Deux\nlignes, \"oui\"".into()));
    page.properties
        .custom
        .insert(check.id, CellValue::Checkbox(true));
    let export = csv("Mon suivi", &[&base], &[&page]);
    assert!(export.contents.starts_with("\"ID\",\"Database\",\"Name\""));
    assert!(
        export
            .contents
            .contains("\"Budget\",\"Retour\",\"Approved\"\r\n")
    );
    assert!(
        export
            .contents
            .contains("\"Titre, avec \"\"guillemets\"\" 🦀\"")
    );
    assert!(
        export
            .contents
            .ends_with("\"123.456789012345\",\"Deux\nlignes, \"\"oui\"\"\",\"true\"\r\n")
    );
    let note = markdown(&page, &[base]);
    assert!(note.contents.contains("**Budget** : 123\\.456789012345"));
}

#[test]
fn csv_separates_columns_with_identical_ids_across_sources() {
    let mut a = Page::new(None);
    a.title = "A".into();
    let mut b = Page::new(None);
    b.title = "B".into();
    let mut column = Column::new("Points", ColumnKind::Number);
    column.id = "same-id".into();
    a.collection = Some(Collection {
        columns: vec![column.clone()],
        ..Default::default()
    });
    b.collection = a.collection.clone();
    let mut row = Page::new(Some(a.id.clone()));
    row.properties
        .custom
        .insert(column.id, CellValue::Number("42".into()));
    let export = csv("Global", &[&a, &b], &[&row]);
    assert!(
        export
            .contents
            .contains("\"A / Points\",\"B / Points\"\r\n")
    );
    assert!(export.contents.ends_with("\"42\",\"\"\r\n"));
}

#[test]
fn atomic_exports_replace_complete_files_and_leave_no_partial_files_on_failure() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("note.md");
    std::fs::write(&path, "Ancien").unwrap();
    write_atomic(&path, "Nouveau 🦀").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "Nouveau 🦀");
    let directory = temp.path().join("folder");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("keep"), "Conserver").unwrap();
    assert!(write_atomic(&directory, "Échec").is_err());
    assert_eq!(
        std::fs::read_to_string(directory.join("keep")).unwrap(),
        "Conserver"
    );
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
}
