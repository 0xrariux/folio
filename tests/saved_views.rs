use folio::{
    collection::{Collection, Priority, Sort, Status, View},
    dashboard::Deadline,
    model::Page,
    storage::Store,
};

#[test]
fn views_isolate_settings_validate_names_and_keep_a_last_view() {
    let mut c = Collection::default();
    c.save_view("Urgences", false).unwrap();
    let all = c.views[0].id.clone();
    let urgent = c.active_view.clone().unwrap();
    c.view = View::Dashboard;
    c.sort = Sort::Due;
    c.priority = Some(Priority::High);
    c.deadline = Deadline::Overdue;
    c.sync_view();
    c.select_view(&all);
    assert_eq!(
        (c.view, c.sort, c.priority, c.deadline),
        (View::Table, Sort::Title, None, Deadline::All)
    );
    c.select_view(&urgent);
    assert_eq!(
        (c.view, c.sort, c.priority, c.deadline),
        (
            View::Dashboard,
            Sort::Due,
            Some(Priority::High),
            Deadline::Overdue
        )
    );
    let before = c.clone();
    for name in ["", "   ", "URGENCES", &"é".repeat(41)] {
        assert!(c.save_view(name, false).is_err());
        assert_eq!(c, before);
    }
    c.save_view(" À traiter ", true).unwrap();
    assert_eq!(c.active().unwrap().name, "À traiter");
    assert!(c.remove_view(&urgent));
    assert_eq!(c.active_view, Some(all.clone()));
    assert_eq!(c.view, View::Table);
    assert!(!c.remove_view(&all));
    assert_eq!(c.views.len(), 1);
    for i in 1..20 {
        c.save_view(&format!("Vue {i}"), false).unwrap();
    }
    assert!(c.save_view("Au-delà", false).is_err());
    c.save_view("Renommage à la limite", true).unwrap();
}

#[test]
fn named_views_and_active_selection_survive_turso_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("views.db");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let store = Store::open(&path).await.unwrap();
            let mut page = Page::new(None);
            let mut c = Collection::default();
            c.save_view("Suivi", false).unwrap();
            c.status = Some(Status::InProgress);
            c.view = View::Board;
            c.sync_view();
            page.collection = Some(c);
            store.save(&page).await.unwrap();
            drop(store);
            let reopened = Store::open(&path).await.unwrap();
            assert_eq!(reopened.load().await.unwrap(), vec![page]);
        });
}

#[test]
fn version_three_keeps_its_filters_and_schema_without_inventing_views() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("v3.db");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let store = Store::open(&path).await.unwrap();
            let mut page = Page::new(None);
            page.collection = Some(Collection {
                view: View::Dashboard,
                deadline: Deadline::NextWeek,
                priority: Some(Priority::High),
                ..Default::default()
            });
            store.save(&page).await.unwrap();
            let database = turso::Builder::new_local(path.to_str().unwrap())
                .build()
                .await
                .unwrap();
            let conn = database.connect().unwrap();
            let mut json = serde_json::to_value(page.content()).unwrap();
            json["version"] = 3.into();
            json["collection"].as_object_mut().unwrap().remove("views");
            json["collection"]
                .as_object_mut()
                .unwrap()
                .remove("active_view");
            conn.execute(
                "UPDATE page_content SET content_json=?",
                (json.to_string(),),
            )
            .await
            .unwrap();
            drop(conn);
            drop(database);
            drop(store);
            let store = Store::open(&path).await.unwrap();
            assert_eq!(store.load().await.unwrap(), vec![page.clone()]);
            store.save(&page).await.unwrap();
            drop(store);
            assert_eq!(
                Store::open(&path).await.unwrap().load().await.unwrap(),
                vec![page]
            );
        });
}
