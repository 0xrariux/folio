use crate::{
    state::{Panel, Phase, State},
    worker::Worker,
};
use rune::{Env, Key, Runtime};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
fn app(path: &Path) -> Runtime<State> {
    let mut s = State::new();
    s.worker = Some(Worker::start(path.into(), Arc::new(|| {})));
    let mut rt = Runtime::new(s, crate::ui::view, (1100., 820.), Env::new(false, 1.));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.);
    wait(&mut rt, |s| {
        matches!(s.phase, Phase::Ready) && s.library.loaded
    });
    rt
}
fn wait(rt: &mut Runtime<State>, condition: impl Fn(&State) -> bool) {
    let end = Instant::now() + Duration::from_secs(25);
    loop {
        rt.update_if(|s, tx| crate::poll(s, 0., tx));
        rt.frame(1. / 60.);
        if condition(&rt.state) {
            return;
        }
        assert!(
            Instant::now() < end,
            "Library timeout: {:?} / {:?}",
            rt.state.library.error,
            rt.state.export_message
        );
        std::thread::sleep(Duration::from_millis(5));
    }
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
fn raw(id: &str, version: &str) -> serde_json::Value {
    serde_json::json!({"format":1,"id":id,"name":"Lifecycle fixture","version":version,"author":"Tests","folio":">=0.13.0","api":1,"runtime":"manifest","data_version":1,"permissions":["storage"]})
}
#[test]
fn library_installs_disables_uninstalls_reinstalls_and_confirms_data_deletion_through_the_ui() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("folio.db");
    let file = dir.path().join("module.json");
    let id = "org.example.lifecycle";
    std::fs::write(&file, raw(id, "1.0.0").to_string()).unwrap();
    let mut rt = app(&db);
    let original = rt.state.pages.clone();
    tap(&mut rt, "library");
    assert!(rt.find("library-page").is_some());
    assert!(rt.state.library.registry.records.is_empty());
    tap(&mut rt, "library-add");
    assert!(rt.file_dialog().is_some());
    rt.resolve_file_dialog(Some(file.clone())).unwrap();
    rt.frame(0.);
    wait(&mut rt, |s| matches!(s.panel, Panel::ModuleReview));
    assert!(!rt.state.library.registry.records.contains_key(id));
    tap(&mut rt, "library-confirm-install");
    wait(&mut rt, |s| {
        s.library.registry.records.contains_key(id) && !s.library.pending
    });
    tap(&mut rt, "library-tab-installed");
    assert!(!rt.state.library.registry.records[id].active());
    tap(&mut rt, &format!("library-toggle-{id}"));
    wait(&mut rt, |s| s.library.registry.records[id].active());
    tap(&mut rt, &format!("library-toggle-{id}"));
    wait(&mut rt, |s| {
        !s.library.registry.records[id].enabled && !s.library.pending
    });
    tap(&mut rt, &format!("library-uninstall-{id}"));
    tap(&mut rt, "library-cancel-remove");
    assert!(rt.state.library.registry.records[id].installed);
    tap(&mut rt, &format!("library-uninstall-{id}"));
    tap(&mut rt, "library-confirm-remove");
    wait(&mut rt, |s| {
        !s.library.registry.records[id].installed && !s.library.pending
    });
    assert_eq!(rt.state.pages, original);
    drop(rt);
    std::fs::remove_file(&file).unwrap();
    let mut rt = app(&db);
    tap(&mut rt, "library");
    tap(&mut rt, "library-tab-retained");
    tap(&mut rt, &format!("library-reinstall-{id}"));
    tap(&mut rt, "library-confirm-install");
    wait(&mut rt, |s| {
        s.library.registry.records[id].installed && !s.library.pending
    });
    assert!(!rt.state.library.registry.records[id].enabled);
    tap(&mut rt, "library-tab-installed");
    tap(&mut rt, &format!("library-uninstall-{id}"));
    tap(&mut rt, "library-confirm-remove");
    wait(&mut rt, |s| {
        !s.library.registry.records[id].installed && !s.library.pending
    });
    tap(&mut rt, "library-tab-retained");
    tap(&mut rt, &format!("library-purge-{id}"));
    tap(&mut rt, "library-cancel-remove");
    assert!(rt.state.library.registry.records.contains_key(id));
    tap(&mut rt, &format!("library-purge-{id}"));
    tap(&mut rt, "library-confirm-remove");
    wait(&mut rt, |s| {
        !s.library.registry.records.contains_key(id) && !s.library.pending
    });
    let saved = std::fs::read_dir(db.with_extension("backups").join("recovery"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("before-module-delete-")
                && p.extension().is_some_and(|e| e == "folio")
        })
        .unwrap();
    folio::storage::recovery::BackupProof::verify(&saved).unwrap();
    assert!(
        folio::workspace::Notebook::read(&saved)
            .unwrap()
            .modules
            .records
            .contains_key(id)
    );
}
#[test]
fn catalog_import_updates_and_removal_keep_installed_definitions() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("folio.db");
    let file = dir.path().join("catalog.json");
    let id = "org.example.catalogmodule";
    let catalog = "org.example.source";
    let data = serde_json::json!({"format":1,"id":catalog,"name":"Local catalog","modules":[raw(id,"1.0.0")]});
    std::fs::write(&file, data.to_string()).unwrap();
    let mut rt = app(&db);
    tap(&mut rt, "library");
    tap(&mut rt, "library-catalogs");
    tap(&mut rt, "library-add-catalog");
    rt.resolve_file_dialog(Some(file)).unwrap();
    rt.frame(0.);
    wait(&mut rt, |s| {
        s.library.registry.catalogs.contains_key(catalog) && !s.library.pending
    });
    tap(&mut rt, &format!("library-install-{catalog}-{id}"));
    tap(&mut rt, "library-confirm-install");
    wait(&mut rt, |s| {
        s.library.registry.records.contains_key(id) && !s.library.pending
    });
    tap(&mut rt, &format!("library-remove-catalog-{catalog}"));
    wait(&mut rt, |s| {
        s.library.registry.catalogs.is_empty() && !s.library.pending
    });
    assert!(rt.state.library.registry.records[id].installed);
}
#[test]
fn unsupported_packages_leave_the_notebook_unchanged_and_show_a_library_error() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("folio.db");
    let file = dir.path().join("bad.json");
    let mut value = raw("org.example.bad", "1.0.0");
    value["runtime"] = "native".into();
    std::fs::write(&file, value.to_string()).unwrap();
    let mut rt = app(&db);
    let pages = rt.state.pages.clone();
    tap(&mut rt, "library");
    tap(&mut rt, "library-add");
    rt.resolve_file_dialog(Some(file)).unwrap();
    rt.frame(0.);
    wait(&mut rt, |s| s.library.error.is_some() && !s.library.pending);
    assert!(rt.state.library.registry.is_empty());
    assert_eq!(rt.state.pages, pages);
    assert!(matches!(rt.state.panel, Panel::Library));
    assert!(!rt.state.workspace.busy);
}
#[test]
fn library_is_usable_at_minimum_size_and_demo_does_not_ship_fake_modules() {
    let mut state = State::demo();
    state.open_library();
    let mut rt = Runtime::new(state, crate::ui::view, (920., 620.), Env::new(false, 1.));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.);
    for key in [
        "library",
        "library-page",
        "library-add",
        "library-tab-discover",
        "library-tab-installed",
        "library-tab-retained",
        "library-close",
    ] {
        assert!(rt.find(key).is_some(), "{key}");
    }
    assert!(rt.state.library.registry.is_empty());
    assert!(folio::modules::Catalog::built_in().modules.is_empty());
    tap(&mut rt, "library-tab-retained");
    rt.key(Key::Escape.into());
    rt.frame(0.);
    assert!(matches!(rt.state.panel, Panel::Closed));
}

#[test]
fn module_only_changes_are_backed_up_encrypted_and_vault_lock_clears_the_library() {
    use folio::vault::{Action, Vault};
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("folio.db");
    const PASSWORD: &str = "public module test password 2026";
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let key = runtime.block_on(async {
        let mut vault = Vault::start(db.clone()).await.unwrap();
        vault.store().unwrap().load_or_seed().await.unwrap();
        vault
            .action(Action::Enable(zeroize::Zeroizing::new(PASSWORD.into())))
            .await
            .unwrap();
        vault.key.as_ref().unwrap().clone()
    });
    let mut state = State::new();
    state.worker = Some(Worker::start(db.clone(), Arc::new(|| {})));
    let mut rt = Runtime::new(state, crate::ui::view, (1100., 820.), Env::new(false, 1.));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.);
    wait(&mut rt, |s| s.settings.locked);
    let unlock = |rt: &mut Runtime<State>| {
        rt.update(|_, tx| tx.focus("security-secret", false));
        rt.frame(0.);
        rt.key(Key::Text(PASSWORD.into()).into());
        rt.frame(0.);
        tap(rt, "submit-security");
        wait(rt, |s| {
            !s.settings.locked && s.library.loaded && !s.settings.busy
        });
    };
    unlock(&mut rt);
    tap(&mut rt, "library");
    let id = "org.example.encrypted";
    rt.update(|s, _| s.review_module(raw(id, "1.0.0")));
    rt.frame(0.);
    tap(&mut rt, "library-confirm-install");
    wait(&mut rt, |s| {
        s.library.registry.records.contains_key(id) && !s.library.pending
    });
    rt.update(|s, _| s.lock_notebook());
    wait(&mut rt, |s| s.settings.locked && !s.settings.busy);
    assert!(!rt.state.library.loaded);
    assert!(rt.state.library.registry.is_empty());
    assert!(rt.find("library-page").is_none());
    let backups = || {
        std::fs::read_dir(db.with_extension("backups"))
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "folio"))
            .map(|p| folio::security::read_backup(&p, Some(&key), None).unwrap())
            .collect::<Vec<_>>()
    };
    assert!(
        backups().iter().any(|b| b.modules.records.contains_key(id)),
        "module-only changes need an automatic backup before locking"
    );
    unlock(&mut rt);
    assert!(rt.state.library.registry.records[id].installed);
    tap(&mut rt, "library");
    tap(&mut rt, "library-tab-installed");
    tap(&mut rt, &format!("library-uninstall-{id}"));
    tap(&mut rt, "library-confirm-remove");
    wait(&mut rt, |s| {
        !s.library.registry.records[id].installed && !s.library.pending
    });
    drop(rt);
    assert!(
        backups()
            .iter()
            .any(|b| b.modules.records.get(id).is_some_and(|r| !r.installed)),
        "module-only changes need an automatic backup on exit"
    );
}

#[test]
fn large_catalogs_render_in_batches_and_search_resets_the_batch() {
    let mut state = State::demo();
    let catalog = folio::modules::Catalog {
        format: 1,
        id: "org.example.large".into(),
        name: "Large test catalog".into(),
        modules: (0..100)
            .map(|i| {
                let mut value = raw(&format!("org.example.item{i:03}"), "1.0.0");
                value["name"] = format!("Module {i:03}").into();
                value
            })
            .collect(),
    };
    state
        .library
        .registry
        .apply(folio::modules::Action::Catalog(catalog))
        .unwrap();
    state.open_library();
    let mut rt = Runtime::new(state, crate::ui::view, (1100., 820.), Env::new(false, 1.));
    rt.frame(0.);
    let count = |rt: &Runtime<State>| {
        rt.elements()
            .iter()
            .filter(|e| {
                e.key
                    .as_deref()
                    .is_some_and(|k| k.starts_with("library-install-"))
            })
            .count()
    };
    assert_eq!(count(&rt), 40);
    tap(&mut rt, "library-show-more");
    assert_eq!(count(&rt), 80);
    tap(&mut rt, "library-show-more");
    assert_eq!(count(&rt), 100);
    rt.update(|_, tx| {
        tx.scroll_into_view("library-search");
        tx.focus("library-search", false);
    });
    rt.frame(0.);
    rt.key(Key::Text("Module 042".into()).into());
    rt.frame(0.);
    assert_eq!(count(&rt), 1);
    assert!(rt.find("library-show-more").is_none());
}
