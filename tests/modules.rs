//! Only transient definitions are used here. No functional module is shipped in the app.
use folio::{
    modules::{self, Action, Candidate, Catalog, Registry},
    security,
    storage::{Store, recovery},
    vault::{Action as SecurityAction, Vault},
    workspace,
};
use serde_json::{Value, json};
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
fn manifest(id: &str, version: &str) -> Value {
    json!({"format":1,"id":id,"name":"Transient test definition","version":version,"author":"Folio test suite","description":"Metadata only; no implementation","folio":">=0.13.0","api":1,"runtime":"manifest","data_version":1,"permissions":["storage"]})
}
fn install(raw: Value) -> Action {
    let c = Candidate::new(raw).unwrap();
    Action::Install {
        manifest: c.manifest,
        fingerprint: c.fingerprint,
    }
}
#[test]
fn complete_lifecycle_retains_data_and_revokes_old_sessions() {
    let id = "org.example.lifecycle";
    let raw = manifest(id, "1.0.0");
    let mut r = Registry::default();
    r.apply(install(raw.clone())).unwrap();
    assert!(!r.records[id].active());
    assert!(r.session(id).is_err());
    r.apply(Action::Enable(id.into())).unwrap();
    let session = r.session(id).unwrap();
    let data = json!({"opaque":{"future_fields":[1,"two",{"keep":true}]}});
    r.write_data(&session, data.clone()).unwrap();
    // Duplicate installation is idempotent, including an active session.
    r.apply(install(raw.clone())).unwrap();
    assert_eq!(r.read_data(&session).unwrap(), data);
    r.apply(Action::Disable(id.into())).unwrap();
    assert!(r.read_data(&session).is_err());
    assert_eq!(r.records[id].data, data);
    r.apply(Action::Enable(id.into())).unwrap();
    assert!(r.write_data(&session, json!({})).is_err());
    let current = r.session(id).unwrap();
    r.apply(Action::Uninstall(id.into())).unwrap();
    assert!(r.read_data(&current).is_err());
    assert!(!r.records[id].installed);
    assert!(r.records[id].grants.is_empty());
    r.apply(install(raw)).unwrap();
    assert_eq!(r.records[id].data, data);
    assert!(!r.records[id].active());
    r.apply(Action::Enable(id.into())).unwrap();
    assert_eq!(r.read_data(&r.session(id).unwrap()).unwrap(), data);
    assert!(r.apply(Action::Purge(id.into())).is_err());
    r.apply(Action::Uninstall(id.into())).unwrap();
    r.apply(Action::Purge(id.into())).unwrap();
    assert!(r.is_empty());
    r.apply(install(manifest(id, "1.0.0"))).unwrap();
    r.apply(Action::Enable(id.into())).unwrap();
    assert!(
        r.read_data(&session).is_err(),
        "Recreating a deleted module must not revive its original session"
    );
    assert_eq!(r.read_data(&r.session(id).unwrap()).unwrap(), json!({}));
}
#[test]
fn updates_are_reviewed_atomic_and_never_guess_a_data_migration() {
    let id = "org.example.versions";
    let mut r = Registry::default();
    r.apply(install(manifest(id, "1.0.0"))).unwrap();
    r.apply(Action::Enable(id.into())).unwrap();
    let session = r.session(id).unwrap();
    r.write_data(&session, json!({"precious":"kept"})).unwrap();
    r.apply(install(manifest(id, "1.1.0"))).unwrap();
    assert!(!r.records[id].enabled);
    assert!(r.read_data(&session).is_err());
    assert_eq!(r.records[id].data["precious"], "kept");
    let before = r.clone();
    assert!(r.apply(install(manifest(id, "1.0.0"))).is_err());
    assert_eq!(r, before);
    let mut changed = manifest(id, "1.1.0");
    changed["author"] = "Different bytes for the same version".into();
    assert!(r.apply(install(changed)).is_err());
    assert_eq!(r, before);
    let mut newer = manifest(id, "2.0.0");
    newer["data_version"] = 2.into();
    assert!(r.apply(install(newer)).is_err());
    assert_eq!(r, before);
    let candidate = Candidate::new(manifest(id, "1.2.0")).unwrap();
    assert!(
        r.apply(Action::Install {
            manifest: candidate.manifest,
            fingerprint: "wrong".into()
        })
        .is_err()
    );
    assert_eq!(r, before);
}
#[test]
fn unsupported_code_permissions_api_and_app_versions_are_never_activated() {
    for (field, value) in [
        ("runtime", json!("native")),
        ("runtime", json!("wasm")),
        ("permissions", json!(["network"])),
        ("api", json!(999)),
        ("format", json!(999)),
        ("folio", json!(">=999.0.0")),
        ("command", json!("echo must never execute")),
    ] {
        let mut m = manifest("org.example.unsupported", "1.0.0");
        m[field] = value;
        assert!(Candidate::new(m).is_err(), "{field}");
    }
    for id in [
        "../escape",
        "org..empty",
        "/tmp/module",
        "UPPER.name",
        "single",
    ] {
        assert!(!modules::valid_id(id));
    }
    let mut no_storage = manifest("org.example.noaccess", "1.0.0");
    no_storage["permissions"] = json!([]);
    let mut r = Registry::default();
    r.apply(install(no_storage)).unwrap();
    r.apply(Action::Enable("org.example.noaccess".into()))
        .unwrap();
    assert!(r.session("org.example.noaccess").is_err());
}
#[test]
fn catalogs_can_be_added_replaced_removed_without_removing_installed_modules() {
    let id = "org.example.catalogtest";
    let mut r = Registry::default();
    let mut catalog = Catalog {
        format: 1,
        id: "org.example.catalog".into(),
        name: "Transient test catalog".into(),
        modules: vec![manifest(id, "1.0.0")],
    };
    r.apply(Action::Catalog(catalog.clone())).unwrap();
    r.apply(install(catalog.modules[0].clone())).unwrap();
    catalog.modules[0]["version"] = "1.1.0".into();
    r.apply(Action::Catalog(catalog.clone())).unwrap();
    assert_eq!(r.catalogs[&catalog.id].modules[0]["version"], "1.1.0");
    r.apply(Action::RemoveCatalog(catalog.id)).unwrap();
    assert!(r.records[id].installed);
    assert!(r.catalogs.is_empty());
    assert!(
        Catalog::built_in().modules.is_empty(),
        "No actual module should be shipped yet"
    );
}
#[test]
fn unavailable_future_definitions_remain_opaque_and_do_not_block_the_notebook() {
    runtime().block_on(async {
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("opaque.db");let store=Store::open(&path).await.unwrap();let id="org.example.future";
        store.module_action(install(manifest(id,"1.0.0"))).await.unwrap();let mut notebook=store.notebook(&workspace::asset_dir(&path)).await.unwrap();
        let record=notebook.modules.records.get_mut(id).unwrap();record.manifest["format"]=99.into();record.manifest["future_extension"]=json!({"keep":[1,2,3]});record.fingerprint=modules::digest(&record.manifest);record.enabled=true;record.grants=vec!["future_capability".into()];record.data=json!({"unknown_schema":{"do_not_drop":true}});
        notebook.validate().unwrap();assert!(!record_active(&notebook,id));
        store.restore_notebook(&notebook,&workspace::asset_dir(&path)).await.unwrap();store.checkpoint_wal().await.unwrap();drop(store);
        let reopened=Store::open(&path).await.unwrap();assert_eq!(reopened.notebook(&workspace::asset_dir(&path)).await.unwrap(),notebook);assert!(reopened.module_session(id).await.is_err());
        reopened.module_action(Action::Uninstall(id.into())).await.unwrap();assert_eq!(reopened.module_registry().await.unwrap().records[id].data["unknown_schema"]["do_not_drop"],true);
    });
}
fn record_active(n: &workspace::Notebook, id: &str) -> bool {
    n.modules.records[id].active()
}
#[test]
fn durable_namespaces_survive_reopen_and_cannot_be_used_in_another_or_restored_profile() {
    runtime().block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("modules.db");
        let store = Store::open(&path).await.unwrap();
        for id in ["org.example.alpha", "org.example.beta"] {
            store
                .module_action(install(manifest(id, "1.0.0")))
                .await
                .unwrap();
            store
                .module_action(Action::Enable(id.into()))
                .await
                .unwrap();
        }
        let a = store.module_session("org.example.alpha").await.unwrap();
        let b = store.module_session("org.example.beta").await.unwrap();
        store
            .write_module_data(&a, json!({"alpha":"only here"}))
            .await
            .unwrap();
        assert_eq!(store.read_module_data(&b).await.unwrap(), json!({}));
        let expected = store.notebook(&workspace::asset_dir(&path)).await.unwrap();
        assert_eq!(expected.format, 2);
        let other_path = dir.path().join("other.db");
        let other = Store::open(&other_path).await.unwrap();
        other
            .restore_notebook(&expected, &workspace::asset_dir(&other_path))
            .await
            .unwrap();
        assert!(
            other
                .write_module_data(&a, json!({"wrong":"profile"}))
                .await
                .is_err()
        );
        store
            .restore_notebook(&expected, &workspace::asset_dir(&path))
            .await
            .unwrap();
        assert!(store.read_module_data(&a).await.is_err());
        store.checkpoint_wal().await.unwrap();
        drop(store);
        let reopened = Store::open(&path).await.unwrap();
        assert_eq!(
            reopened
                .notebook(&workspace::asset_dir(&path))
                .await
                .unwrap(),
            expected
        );
        assert!(reopened.read_module_data(&a).await.is_err());
        let fresh = reopened.module_session("org.example.alpha").await.unwrap();
        assert_eq!(
            reopened.read_module_data(&fresh).await.unwrap()["alpha"],
            "only here"
        );
    });
}
#[test]
fn module_data_is_encrypted_backed_up_rehearsed_and_recovered_after_deletion() {
    runtime().block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let mut vault = Vault::start(path.clone()).await.unwrap();
        let id = "org.example.encrypted";
        vault
            .action(SecurityAction::Enable(zeroize::Zeroizing::new(
                "module fixture password 2026".into(),
            )))
            .await
            .unwrap();
        let store = vault.store().unwrap();
        store
            .module_action(install(manifest(id, "1.0.0")))
            .await
            .unwrap();
        store
            .module_action(Action::Enable(id.into()))
            .await
            .unwrap();
        let token = store.module_session(id).await.unwrap();
        store
            .write_module_data(&token, json!({"private":"module-private-fixture-xyz"}))
            .await
            .unwrap();
        store
            .module_action(Action::Uninstall(id.into()))
            .await
            .unwrap();
        let expected = store.notebook(&workspace::asset_dir(&path)).await.unwrap();
        let backup = dir.path().join("before-purge.folio");
        recovery::write_backup(
            store,
            &path,
            &backup,
            vault.config.as_ref().zip(vault.key.as_ref()),
            true,
        )
        .await
        .unwrap();
        recovery::BackupProof::verify(&backup).unwrap();
        assert!(
            !std::fs::read_to_string(&backup)
                .unwrap()
                .contains("module-private-fixture-xyz")
        );
        assert!(store.module_action(Action::Purge(id.into())).await.is_err());
        store.purge_module(id.into(), &backup).await.unwrap();
        assert!(store.module_registry().await.unwrap().records.is_empty());
        let saved =
            security::read_backup(&backup, None, Some("module fixture password 2026")).unwrap();
        assert_eq!(saved, expected);
        store
            .restore_notebook(&saved, &workspace::asset_dir(&path))
            .await
            .unwrap();
        assert_eq!(store.module_registry().await.unwrap(), expected.modules);
        let mut downgraded = saved;
        downgraded.format = 1;
        assert!(downgraded.validate().is_err());
    });
}
#[test]
fn bad_or_missing_recovery_receipts_never_authorize_data_deletion() {
    runtime().block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.db");
        let store = Store::open(&path).await.unwrap();
        let id = "org.example.retained";
        store
            .module_action(install(manifest(id, "1.0.0")))
            .await
            .unwrap();
        store
            .module_action(Action::Uninstall(id.into()))
            .await
            .unwrap();
        assert!(
            store
                .purge_module(id.into(), &dir.path().join("missing.folio"))
                .await
                .is_err()
        );
        assert!(
            store
                .module_registry()
                .await
                .unwrap()
                .records
                .contains_key(id)
        );
    });
}
