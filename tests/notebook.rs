use folio::{
    document::{Kind, NoteBlock},
    fields::{Column, ColumnKind},
    model::Page,
    storage::Store,
    workspace::{self, Notebook, Preferences},
};
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn complete_notebook_roundtrip_keeps_files_history_preferences_order_and_views() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.db");
    let target = temp.path().join("restored.db");
    runtime().block_on(async {
        let store = Store::open(&original).await.unwrap();
        let file = temp.path().join("Résumé.pdf");
        std::fs::write(&file, b"%PDF-1.4\nFolio fixture\n%%EOF").unwrap();
        let asset = workspace::import_asset(&file, &workspace::asset_dir(&original)).unwrap();
        let mut page = Page::new(None);
        page.title = "Avant".into();
        page.order = 2048;
        let mut attachment = NoteBlock::text(Kind::Attachment, &asset.name);
        attachment.asset = Some(asset.clone());
        page.blocks.push(attachment);
        page.sync_document();
        store.save(&page).await.unwrap();
        store.checkpoint(&page).await.unwrap();
        page.title = "Après 🦀".into();
        store.save(&page).await.unwrap();
        let prefs = Preferences {
            dark: false,
            sidebar_width: 312,
            last_page: Some(page.id.clone()),
        };
        store.save_preferences(&prefs).await.unwrap();
        let snapshot = store
            .notebook(&workspace::asset_dir(&original))
            .await
            .unwrap();
        let backup = temp.path().join("carnet.folio");
        snapshot.write(&backup).unwrap();
        let bundle = Notebook::read(&backup).unwrap();
        let destination = Store::open(&target).await.unwrap();
        destination.save(&Page::new(None)).await.unwrap();
        destination
            .restore_notebook(&bundle, &workspace::asset_dir(&target))
            .await
            .unwrap();
        drop(destination);
        let reopened = Store::open(&target).await.unwrap();
        assert_eq!(reopened.load().await.unwrap(), vec![page]);
        assert_eq!(reopened.preferences().await.unwrap(), prefs);
        assert_eq!(reopened.history(None).await.unwrap(), snapshot.history);
        assert_eq!(
            std::fs::read(
                workspace::asset_path(&workspace::asset_dir(&target), &asset.id).unwrap()
            )
            .unwrap(),
            std::fs::read(file).unwrap()
        );
        let exported = temp.path().join("export");
        workspace::export_assets(
            std::slice::from_ref(&asset),
            &workspace::asset_dir(&target),
            &exported,
        )
        .unwrap();
        assert!(
            exported
                .join(workspace::asset_export_name(&asset))
                .is_file()
        );
    });
}

#[test]
fn history_coalesces_rapid_edits_bounds_checkpoints_and_survives_restart() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("history.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let mut page = Page::new(None);
        page.title = "Original".into();
        store.save(&page).await.unwrap();
        for i in 0..8 {
            page.title = format!("Édition {i}");
            store.save(&page).await.unwrap();
        }
        let history = store.history(Some(&page.id)).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].page.title, "Original");
        for i in 0..55 {
            page.title = format!("Version {i}");
            store.checkpoint(&page).await.unwrap();
        }
        drop(store);
        let store = Store::open(&path).await.unwrap();
        let history = store.history(Some(&page.id)).await.unwrap();
        assert_eq!(history.len(), 50);
        assert_eq!(history[0].page.title, "Version 54");
        assert_eq!(history[49].page.title, "Version 5");
        let bundle = store.notebook(&workspace::asset_dir(&path)).await.unwrap();
        let other_path = temp.path().join("history-restored.db");
        let other = Store::open(&other_path).await.unwrap();
        other
            .restore_notebook(&bundle, &workspace::asset_dir(&other_path))
            .await
            .unwrap();
        assert_eq!(other.history(Some(&page.id)).await.unwrap(), history);
    });
}

#[test]
fn invalid_bundle_and_sql_failure_leave_the_current_notebook_intact() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("safe.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let page = Page::new(None);
        store.save(&page).await.unwrap();
        let mut bundle = store.notebook(&workspace::asset_dir(&path)).await.unwrap();
        bundle.pages[0].title = "Importé".into();
        bundle.pages.push(bundle.pages[0].clone());
        assert!(bundle.validate().is_err());
        assert!(
            store
                .restore_notebook(&bundle, &workspace::asset_dir(&path))
                .await
                .is_err()
        );
        assert_eq!(store.load().await.unwrap(), vec![page.clone()]);
        bundle.pages.pop();
        bundle.assets.insert("../../outside".into(), "AA==".into());
        assert!(bundle.validate().is_err());
        bundle.assets.clear();
        let db = turso::Builder::new_local(path.to_str().unwrap())
            .build()
            .await
            .unwrap();
        let conn = db.connect().unwrap();
        conn.execute("DROP TABLE preferences", ()).await.unwrap();
        assert!(
            store
                .restore_notebook(&bundle, &workspace::asset_dir(&path))
                .await
                .is_err()
        );
        assert_eq!(store.load().await.unwrap(), vec![page]);
    });
}

#[test]
fn independent_views_keep_visibility_width_and_column_order_after_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("views.db");
    runtime().block_on(async {
        let mut page = Page::new(None);
        let field = Column::new("Charge", ColumnKind::Number);
        let id = field.id.clone();
        let mut collection = folio::collection::Collection {
            columns: vec![field],
            ..Default::default()
        };
        collection.layout.widths.insert(id.clone(), 160);
        collection.save_view("Complète", false).unwrap();
        let first = collection.active_view.clone().unwrap();
        collection.save_view("Compacte", false).unwrap();
        let second = collection.active_view.clone().unwrap();
        collection.columns[0].visible = false;
        collection.layout.widths.insert(id.clone(), 300);
        collection.layout.order = vec![id.clone(), "Status".into()];
        collection.select_view(&first);
        assert!(collection.columns[0].visible);
        assert_eq!(collection.width(&id, 140.0), 160.0);
        collection.select_view(&second);
        assert!(!collection.columns[0].visible);
        assert_eq!(collection.width(&id, 140.0), 300.0);
        assert_eq!(collection.layout.order[0], id);
        page.collection = Some(collection);
        let store = Store::open(&path).await.unwrap();
        store.save(&page).await.unwrap();
        drop(store);
        assert_eq!(
            Store::open(&path).await.unwrap().load().await.unwrap(),
            vec![page]
        );
    });
}
