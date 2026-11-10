use folio::{
    collection::{Collection, Priority, Sort, Status, View, parse_tags, valid_date},
    document::{Kind, NoteBlock},
    model::Page,
    storage::Store,
};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn collection_settings_properties_and_page_links_survive_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("collection.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let mut parent = Page::new(None);
        parent.title = "Projet".into();
        parent.collection = Some(Collection {
            view: View::Board,
            sort: Sort::Due,
            status: Some(Status::InProgress),
            ..Default::default()
        });
        let mut record = Page::new(Some(parent.id.clone()));
        record.title = "Une fiche".into();
        record.properties.status = Status::InProgress;
        record.properties.priority = Priority::High;
        record.properties.due = Some("2028-02-29".into());
        record.properties.tags = vec!["Rust".into(), "Folio".into()];
        let mut source = Page::new(None);
        let mut link = NoteBlock::text(Kind::PageLink, "Une fiche");
        link.target_id = Some(record.id.clone());
        source.blocks = vec![link];
        source.sync_document();
        for page in [&parent, &record, &source] {
            store.save(page).await.unwrap();
        }
        drop(store);
        let reopened = Store::open(&path).await.unwrap();
        let pages = reopened.load().await.unwrap();
        for expected in [parent, record, source] {
            assert_eq!(pages.iter().find(|p| p.id == expected.id), Some(&expected));
        }
    });
}

#[test]
fn version_one_documents_get_safe_defaults_without_changing_block_ids() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("v2.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let page = Page::new(None);
        store.save(&page).await.unwrap();
        let database = turso::Builder::new_local(path.to_str().unwrap())
            .build()
            .await
            .unwrap();
        let connection = database.connect().unwrap();
        let mut json = serde_json::to_value(page.content()).unwrap();
        let object = json.as_object_mut().unwrap();
        object.insert("version".into(), 1.into());
        object.remove("collection");
        object.remove("properties");
        for block in object["blocks"].as_array_mut().unwrap() {
            block.as_object_mut().unwrap().remove("target_id");
        }
        connection
            .execute(
                "UPDATE page_content SET content_json=?",
                (serde_json::to_string(&json).unwrap(),),
            )
            .await
            .unwrap();
        drop(connection);
        drop(database);
        drop(store);
        let reopened = Store::open(&path).await.unwrap();
        let loaded = reopened.load().await.unwrap();
        assert_eq!(loaded, vec![page]);
    });
}

#[test]
fn dates_and_tags_are_validated_without_silent_truncation() {
    for date in ["2028-02-29", "2000-02-29", "2026-11-10"] {
        assert!(valid_date(date));
    }
    for date in [
        "1900-02-29",
        "2026-02-29",
        "2026-04-31",
        "2026-1-01",
        "0000-01-01",
        "2026-12-00",
        "ééé-11-10",
    ] {
        assert!(!valid_date(date), "{date}");
    }
    assert_eq!(
        parse_tags("Rust, rust, Produit, , produit").unwrap(),
        vec!["Rust", "Produit"]
    );
    assert!(parse_tags(&"a".repeat(41)).is_err());
    assert!(parse_tags(&(0..13).map(|i| i.to_string()).collect::<Vec<_>>().join(",")).is_err());
}
