use folio::{
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
fn existing_v1_database_migrates_once_without_losing_rich_content() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("v1.db");
    runtime().block_on(async {
        let database=turso::Builder::new_local(path.to_str().unwrap()).build().await.unwrap();
        let connection=database.connect().unwrap();
        connection.execute(include_str!("../migrations/001_initial.sql"),()).await.unwrap();
        let html="<p>Une <b>idée 🦀</b> à garder.</p><ul><li>Premier<ul><li>Enfant <i>important</i></li></ul></li></ul><blockquote>Une citation</blockquote><pre><code>ligne 1\nligne 2</code></pre>";
        connection.execute("INSERT INTO pages VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            ("ancienne-page",Option::<String>::None,"Mon carnet",html,"Texte de recherche",1_i64,0_i64,42_i64)).await.unwrap();
        drop(connection);drop(database);
        let store=Store::open(&path).await.unwrap();let pages=store.load().await.unwrap();
        assert_eq!(pages.len(),1);let page=&pages[0];
        assert_eq!(page.body_html,html);assert_eq!(page.title,"Mon carnet");assert!(page.favorite);assert_eq!(page.updated_at,42);
        assert!(page.blocks.iter().any(|b|b.kind==Kind::Bullet&&b.indent==1&&b.html.contains("<em>important</em>")));
        assert!(page.blocks.iter().any(|b|b.kind==Kind::Code&&b.plain_text()=="ligne 1\nligne 2"));
        assert!(page.blocks.iter().any(|b|b.html.contains("<strong>idée 🦀</strong>")));
        drop(store);
        let store=Store::open(&path).await.unwrap();assert_eq!(store.load().await.unwrap(),pages);
    });
}

#[test]
fn blocks_tasks_and_page_appearance_survive_a_restart() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("blocks.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let mut page = Page::new(None);
        page.emoji = "🌿".into();
        page.cover = 2;
        page.wide = true;
        page.blocks = vec![
            NoteBlock::text(Kind::Heading1, "Un projet"),
            NoteBlock::text(Kind::Todo, "Une tâche"),
            NoteBlock::new(Kind::Divider),
            NoteBlock::text(Kind::Callout, "Un détail"),
        ];
        page.blocks[1].checked = true;
        page.blocks[1].indent = 2;
        page.sync_document();
        store.save(&page).await.unwrap();
        drop(store);
        let reopened = Store::open(&path).await.unwrap();
        assert_eq!(reopened.load().await.unwrap(), vec![page]);
    });
}

#[test]
fn failure_to_save_content_rolls_back_the_page_update() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("atomic.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let mut page = Page::new(None);
        page.title = "Avant".into();
        store.save(&page).await.unwrap();
        let database = turso::Builder::new_local(path.to_str().unwrap())
            .build()
            .await
            .unwrap();
        let connection = database.connect().unwrap();
        connection
            .execute("DROP TABLE page_content", ())
            .await
            .unwrap();
        page.title = "Après".into();
        assert!(store.save(&page).await.is_err());
        let mut rows = connection
            .query("SELECT title FROM pages WHERE id=?", (page.id,))
            .await
            .unwrap();
        assert_eq!(
            rows.next()
                .await
                .unwrap()
                .unwrap()
                .get::<String>(0)
                .unwrap(),
            "Avant"
        );
    });
}

#[test]
fn a_future_document_version_fails_explicitly_instead_of_replacing_notes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("future.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let page = Page::new(None);
        store.save(&page).await.unwrap();
        let database = turso::Builder::new_local(path.to_str().unwrap())
            .build()
            .await
            .unwrap();
        let connection = database.connect().unwrap();
        let mut content = page.content();
        content.version = 99;
        connection
            .execute(
                "UPDATE page_content SET content_json=?",
                (serde_json::to_string(&content).unwrap(),),
            )
            .await
            .unwrap();
        assert!(
            store
                .load_or_seed()
                .await
                .unwrap_err()
                .to_string()
                .contains("newer")
        );
        let mut rows = connection
            .query("SELECT count(*) FROM pages", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            1
        );
    });
}
