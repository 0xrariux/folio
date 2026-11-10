use folio::{model::Page, storage::Store};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn rich_content_hierarchy_and_flags_survive_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("notes.db");
    let runtime = runtime();
    runtime.block_on(async {
        let store = Store::open(&path).await.unwrap();
        let mut parent = Page::new(None);
        parent.title = "L’idée d’aujourd’hui 🦀".into();
        let mut edit =
            rune::TextEdit::from_html("<p>Un <b>vrai</b> carnet.</p><ul><li>Une piste</li></ul>");
        edit.insert("Été ");
        parent.replace_html(&edit.to_html());
        parent.favorite = true;
        let mut child = Page::new(Some(parent.id.clone()));
        child.archived = true;
        store.save(&parent).await.unwrap();
        store.save(&child).await.unwrap();
        // UPSERT updates the same page instead of creating another row.
        parent.title = "Titre modifié avec 'apostrophe' et ?".into();
        store.save(&parent).await.unwrap();
        drop(store);

        let reopened = Store::open(&path).await.unwrap();
        let pages = reopened.load_or_seed().await.unwrap();
        assert_eq!(pages.len(), 2);
        assert_eq!(
            pages.iter().find(|page| page.id == parent.id),
            Some(&parent)
        );
        assert_eq!(pages.iter().find(|page| page.id == child.id), Some(&child));
        let restored = rune::TextEdit::from_html(&parent.body_html);
        assert!(restored.text.contains("vrai"));
        assert_eq!(
            pages.iter().find(|p| p.id == parent.id).unwrap().blocks,
            parent.blocks
        );
    });
}

#[test]
fn welcome_is_seeded_once_even_after_archiving_everything() {
    let temp = tempfile::tempdir().unwrap();
    runtime().block_on(async {
        let store = Store::open(&temp.path().join("notes.db")).await.unwrap();
        let mut pages = store.load_or_seed().await.unwrap();
        assert_eq!(pages.len(), 1);
        pages[0].archived = true;
        store.save(&pages[0]).await.unwrap();
        assert_eq!(store.load_or_seed().await.unwrap(), pages);
    });
}
