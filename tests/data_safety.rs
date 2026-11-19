//! All profiles here are synthetic and isolated. No personal database is ever opened.
use folio::{
    document::{Kind, NoteBlock},
    model::Page,
    storage::{
        APPLICATION_ID, SCHEMA_VERSION, Store,
        recovery::{self, BackupProof},
    },
    workspace::{self, Notebook},
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
async fn raw(path: &Path) -> (turso::Database, turso::Connection) {
    let db = turso::Builder::new_local(path.to_str().unwrap())
        .build()
        .await
        .unwrap();
    let c = db.connect().unwrap();
    (db, c)
}
async fn flush(c: &turso::Connection) {
    let mut rows = c
        .query("PRAGMA wal_checkpoint(TRUNCATE)", ())
        .await
        .unwrap();
    while rows.next().await.unwrap().is_some() {}
}
async fn seeded(path: &Path) -> Page {
    let store = Store::open(path).await.unwrap();
    let mut p = Page::new(None);
    p.title = "Never lose this fixture 🦀".into();
    store.save(&p).await.unwrap();
    store.checkpoint(&p).await.unwrap();
    store.checkpoint_wal().await.unwrap();
    p
}
#[test]
fn native_sqlite_v1_migration_keeps_raw_recovery_copy_and_is_repeatable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.db");
    let original = include_bytes!("fixtures/legacy-v1.sqlite.fixture");
    fs::write(&path, original).unwrap();
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        store.verify_storage().await.unwrap();
        assert_eq!(
            store.scalar("PRAGMA user_version").await.unwrap(),
            SCHEMA_VERSION
        );
        assert_eq!(
            store.scalar("PRAGMA application_id").await.unwrap(),
            APPLICATION_ID
        );
        assert_eq!(store.scalar("PRAGMA synchronous").await.unwrap(), 2);
        let pages = store.load().await.unwrap();
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].title, "Legacy notebook 🦀");
        assert!(pages[0].favorite);
        assert_eq!(pages[1].parent_id.as_deref(), Some("legacy-root"));
        assert!(pages[1].archived);
        let copies: Vec<_> = fs::read_dir(path.with_extension("recovery"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(copies.len(), 1);
        assert!(fs::read(copies[0].join("database")).unwrap() == original);
        assert!(copies[0].join("manifest.json").is_file());
        store.checkpoint_wal().await.unwrap();
        drop(store);
        let next = Store::open(&path).await.unwrap();
        assert_eq!(next.load().await.unwrap(), pages);
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(copies[0].join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["pinned"], true);
        assert!(fs::read(copies[0].join("database")).unwrap() == original);
    });
}
#[test]
fn unknown_schema_and_future_data_fail_without_rewriting_the_database() {
    runtime().block_on(async {
        for case in [
            "schema",
            "identity",
            "column",
            "table",
            "version",
            "document",
            "block",
            "properties",
            "history",
            "preferences",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("future.db");
            let page = seeded(&path).await;
            let (db, c) = raw(&path).await;
            match case {
                "schema" => {
                    c.execute("PRAGMA user_version=999", ()).await.unwrap();
                }
                "identity" => {
                    c.execute("PRAGMA application_id=42", ()).await.unwrap();
                }
                "column" => {
                    c.execute("ALTER TABLE pages ADD COLUMN future TEXT", ())
                        .await
                        .unwrap();
                }
                "table" => {
                    c.execute("CREATE TABLE future_data(payload TEXT)", ())
                        .await
                        .unwrap();
                }
                "history" => {
                    c.execute("UPDATE page_history SET content_json='broken JSON'", ())
                        .await
                        .unwrap();
                }
                "preferences" => {
                    c.execute(
                        "INSERT INTO preferences VALUES('app', '{\"new_setting\":true}')",
                        (),
                    )
                    .await
                    .unwrap();
                }
                _ => {
                    let mut json = serde_json::to_value(page.content()).unwrap();
                    match case {
                        "version" => json["version"] = 99.into(),
                        "document" => json["future_content"] = true.into(),
                        "block" => json["blocks"][0]["future_style"] = true.into(),
                        "properties" => json["properties"]["future_property"] = true.into(),
                        _ => unreachable!(),
                    }
                    c.execute(
                        "UPDATE page_content SET content_json=?",
                        (json.to_string(),),
                    )
                    .await
                    .unwrap();
                }
            }
            flush(&c).await;
            drop(c);
            drop(db);
            let before = fs::read(&path).unwrap();
            assert!(Store::open(&path).await.is_err(), "{case} was accepted");
            assert_eq!(
                fs::read(&path).unwrap(),
                before,
                "{case} modified the original database"
            );
        }
    });
}
#[test]
fn missing_documents_and_orphaned_history_never_trigger_html_repair_or_seeding() {
    runtime().block_on(async {
        for sql in [
            "DELETE FROM page_content",
            "DROP TABLE page_content",
            "UPDATE page_history SET page_id='missing-page'",
            "DELETE FROM pages",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("damaged.db");
            seeded(&path).await;
            let (db, c) = raw(&path).await;
            c.execute(sql, ()).await.unwrap();
            flush(&c).await;
            drop(c);
            drop(db);
            let before = fs::read(&path).unwrap();
            assert!(Store::open(&path).await.is_err(), "{sql}");
            assert!(fs::read(&path).unwrap() == before);
        }
    });
}
#[test]
fn empty_truncated_and_foreign_files_are_preserved_instead_of_replaced() {
    runtime().block_on(async {
        let fixture = include_bytes!("fixtures/legacy-v1.sqlite.fixture");
        for bytes in [
            &b""[..],
            &b"not a database"[..],
            &fixture[..40],
            &fixture[..fixture.len() / 2],
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("bad.db");
            fs::write(&path, bytes).unwrap();
            assert!(Store::open(&path).await.is_err());
            assert!(fs::read(&path).unwrap() == bytes);
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("foreign.db");
        let (db, c) = raw(&path).await;
        c.execute("CREATE TABLE unrelated(value TEXT)", ())
            .await
            .unwrap();
        flush(&c).await;
        drop(c);
        drop(db);
        let before = fs::read(&path).unwrap();
        assert!(Store::open(&path).await.is_err());
        assert!(fs::read(&path).unwrap() == before);
    });
}
#[test]
fn damaged_and_uncertified_backups_cannot_authorize_an_update() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("source.db");
    runtime().block_on(async {
        seeded(&path).await;
        let store = Store::open(&path).await.unwrap();
        let backup = dir.path().join("before-update.folio");
        recovery::write_backup(&store, &path, &backup, None, true)
            .await
            .unwrap();
        BackupProof::verify(&backup).unwrap();
        let original = fs::read(&backup).unwrap();
        fs::write(&backup, &original[..original.len() / 2]).unwrap();
        assert!(BackupProof::verify(&backup).is_err());
        assert!(recovery::certify_plain_backup(&backup).await.is_err());
        fs::write(&backup, &original).unwrap();
        BackupProof::verify(&backup).unwrap();
        fs::remove_file(BackupProof::path(&backup)).unwrap();
        assert!(BackupProof::verify(&backup).is_err());
        recovery::certify_plain_backup(&backup).await.unwrap();
        BackupProof::verify(&backup).unwrap();
        let mut proof: serde_json::Value =
            serde_json::from_slice(&fs::read(BackupProof::path(&backup)).unwrap()).unwrap();
        proof["storage_contract"] = "future-contract".into();
        fs::write(BackupProof::path(&backup), proof.to_string()).unwrap();
        assert!(BackupProof::verify(&backup).is_err());
    });
}
#[test]
fn restored_backup_must_preserve_attachment_bytes_and_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("attachments.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let source = dir.path().join("fixture.bin");
        fs::write(&source, b"exact fixture bytes\0\xff").unwrap();
        let asset = workspace::import_asset(&source, &workspace::asset_dir(&path)).unwrap();
        let mut page = Page::new(None);
        let mut block = NoteBlock::new(Kind::Attachment);
        block.asset = Some(asset.clone());
        page.blocks.push(block);
        page.sync_document();
        store.save(&page).await.unwrap();
        let backup = dir.path().join("complete.folio");
        recovery::write_backup(&store, &path, &backup, None, true)
            .await
            .unwrap();
        let mut notebook = Notebook::read(&backup).unwrap();
        notebook.pages[0].blocks[1].asset.as_mut().unwrap().size += 1;
        assert!(notebook.validate().is_err());
        fs::remove_file(workspace::asset_path(&workspace::asset_dir(&path), &asset.id).unwrap())
            .unwrap();
        assert!(
            recovery::write_backup(
                &store,
                &path,
                &dir.path().join("incomplete.folio"),
                None,
                true
            )
            .await
            .is_err()
        );
        assert!(!dir.path().join("incomplete.verification.json").exists());
    });
}
#[test]
#[ignore = "Invoked only by the crash-recovery parent with an isolated profile"]
fn crash_writer_child() {
    let Some(base) = std::env::var_os("FOLIO_CRASH_TEST_DIRECTORY") else {
        panic!("isolated test path required")
    };
    let base = std::path::PathBuf::from(base);
    let path = base.join("crash.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        for i in 0..40 {
            let mut p = Page::new(None);
            p.id = format!("committed-{i:03}");
            p.title = format!("Durable note {i} 🦀");
            p.updated_at = i;
            store.save(&p).await.unwrap();
            store.checkpoint(&p).await.unwrap();
        }
        store
            .save_preferences(&workspace::Preferences {
                dark: false,
                sidebar_width: 311,
                ..Default::default()
            })
            .await
            .unwrap();
        fs::write(base.join("committed"), b"all transactions committed").unwrap();
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}
#[test]
fn committed_wal_survives_an_actual_process_kill_and_is_preserved_before_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_writer_child", "--ignored", "--nocapture"])
        .env("FOLIO_CRASH_TEST_DIRECTORY", dir.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !dir.path().join("committed").exists() {
        if child.try_wait().unwrap().is_some() {
            panic!("crash writer exited before commit");
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("crash writer timeout");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let path = dir.path().join("crash.db");
    let wal = fs::read(dir.path().join("crash.db-wal")).unwrap();
    assert!(wal.len() > 32);
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        store.verify_storage().await.unwrap();
        let pages = store.load().await.unwrap();
        assert_eq!(pages.len(), 40);
        for (i, p) in pages.iter().enumerate() {
            assert_eq!(p.id, format!("committed-{i:03}"));
            assert_eq!(p.title, format!("Durable note {i} 🦀"));
        }
        assert_eq!(store.history(None).await.unwrap().len(), 40);
        assert_eq!(store.preferences().await.unwrap().sidebar_width, 311);
        let copies: Vec<_> = fs::read_dir(path.with_extension("recovery"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert!(
            copies
                .iter()
                .any(|p| fs::read(p.join("database-wal")).is_ok_and(|bytes| bytes == wal))
        );
    });
}

#[test]
fn frozen_v7_notebook_preserves_every_element_across_restore_reopen_and_encryption() {
    use folio::{
        security,
        vault::{Action, Vault},
    };
    let fixture: Notebook =
        serde_json::from_str(include_str!("fixtures/notebook-v7.json")).unwrap();
    fixture.validate().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("complete.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        store
            .restore_notebook(&fixture, &workspace::asset_dir(&path))
            .await
            .unwrap();
        let backup = dir.path().join("before-update.folio");
        recovery::write_backup(&store, &path, &backup, None, true)
            .await
            .unwrap();
        assert_eq!(Notebook::read(&backup).unwrap(), fixture);
        store.checkpoint_wal().await.unwrap();
        drop(store);
        let mut vault = Vault::start(path.clone()).await.unwrap();
        assert_eq!(
            vault
                .store()
                .unwrap()
                .notebook(&workspace::asset_dir(&path))
                .await
                .unwrap(),
            fixture
        );
        vault
            .action(Action::Enable(zeroize::Zeroizing::new(
                "fixture master password only".into(),
            )))
            .await
            .unwrap();
        assert_eq!(
            vault
                .store()
                .unwrap()
                .notebook(&workspace::asset_dir(&path))
                .await
                .unwrap(),
            fixture
        );
        let encrypted_backup = dir.path().join("encrypted-before-update.folio");
        recovery::write_backup(
            vault.store().unwrap(),
            &path,
            &encrypted_backup,
            vault.config.as_ref().zip(vault.key.as_ref()),
            true,
        )
        .await
        .unwrap();
        BackupProof::verify(&encrypted_backup).unwrap();
        assert_eq!(
            security::read_backup(
                &encrypted_backup,
                None,
                Some("fixture master password only")
            )
            .unwrap(),
            fixture
        );
        // Reserved update/recovery subfolders are encrypted too; old automatic copies
        // must not retain plaintext after enabling the vault.
        let recovery_dir = path.with_extension("backups").join("recovery");
        let saved = fs::read_dir(recovery_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert!(security::read_backup(&saved, None, None).is_err());
    });
}

#[test]
fn all_supported_document_versions_upgrade_without_losing_legacy_block_ids() {
    runtime().block_on(async {
        for version in 1..=7 {
            let dir=tempfile::tempdir().unwrap(); let path=dir.path().join("version.db");
            fs::write(&path,include_bytes!("fixtures/legacy-v1.sqlite.fixture")).unwrap();
            let (db,c)=raw(&path).await;
            c.execute(include_str!("../migrations/002_blocks.sql"),()).await.unwrap();
            for id in ["legacy-root","legacy-child"] {
                let content=serde_json::json!({"version":version,"blocks":[{"id":format!("stable-{id}"),"kind":"Text","html":"<p>Keep this original block</p>"}],"emoji":"📄","cover":0,"wide":false});
                c.execute("INSERT INTO page_content VALUES(?,?)",(id,content.to_string())).await.unwrap();
            }
            flush(&c).await; drop(c);drop(db);
            let store=Store::open(&path).await.unwrap();
            let pages=store.load().await.unwrap(); assert_eq!(pages.len(),2);
            for p in pages { assert_eq!(p.blocks[0].id,format!("stable-{}",p.id)); assert_eq!(p.blocks[0].html,"<p>Keep this original block</p>"); }
            assert!(fs::read_dir(path.with_extension("recovery")).unwrap().next().is_some());
        }
    });
}

#[test]
fn database_written_by_turso_0_7_2_matches_the_frozen_complete_notebook() {
    let fixture: Notebook =
        serde_json::from_str(include_str!("fixtures/notebook-v7.json")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("older-engine.db");
    fs::write(
        &path,
        include_bytes!("fixtures/turso-0.7.2-v7.sqlite.fixture"),
    )
    .unwrap();
    fixture
        .install_assets(&workspace::asset_dir(&path))
        .unwrap();
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        store.verify_storage().await.unwrap();
        assert_eq!(
            store.notebook(&workspace::asset_dir(&path)).await.unwrap(),
            fixture
        );
        let backup = dir.path().join("older-engine.folio");
        recovery::write_backup(&store, &path, &backup, None, true)
            .await
            .unwrap();
        BackupProof::verify(&backup).unwrap();
        assert_eq!(Notebook::read(&backup).unwrap(), fixture);
    });
}

#[test]
#[ignore = "Manual-only producer for a new immutable format fixture, never normal CI"]
fn produce_encrypted_fixture() {
    use folio::{
        security,
        vault::{Action, Vault},
    };
    let output = std::env::var_os("FOLIO_FIXTURE_OUTPUT_DIRECTORY")
        .expect("explicit fixture output directory required");
    let output = std::path::PathBuf::from(output);
    fs::create_dir_all(&output).unwrap();
    for name in [
        "turso-0.8.3-encrypted.db.fixture",
        "vault-v1.security.fixture",
        "encrypted-notebook-v1.folio.fixture",
        "pin-page-clear.json",
    ] {
        assert!(!output.join(name).exists(), "fixture already exists");
    }
    let fixture: Notebook =
        serde_json::from_str(include_str!("fixtures/notebook-v7.json")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.db");
    runtime().block_on(async {
        let mut vault = Vault::start(path.clone()).await.unwrap();
        vault
            .store()
            .unwrap()
            .restore_notebook(&fixture, &workspace::asset_dir(&path))
            .await
            .unwrap();
        let mut page = Page::new(None);
        page.id = "fixture-pin".into();
        page.title = "Frozen PIN page".into();
        page.updated_at = 6;
        page.blocks = vec![NoteBlock::text(Kind::Text, "Frozen PIN content 🦀")];
        page.blocks[0].id = "fixture-pin-block".into();
        page.sync_document();
        vault.save(&page).await.unwrap();
        vault.store().unwrap().checkpoint(&page).await.unwrap();
        vault
            .action(Action::Enable(zeroize::Zeroizing::new(
                "Folio fixture master password 2026".into(),
            )))
            .await
            .unwrap();
        vault
            .action(Action::Protect {
                id: page.id.clone(),
                pin: zeroize::Zeroizing::new("58391726".into()),
            })
            .await
            .unwrap();
        let notebook = vault
            .store()
            .unwrap()
            .notebook(&workspace::asset_dir(&path))
            .await
            .unwrap();
        security::write_backup(
            &notebook,
            &output.join("encrypted-notebook-v1.folio.fixture"),
            vault.config.as_ref().unwrap(),
            vault.key.as_ref().unwrap(),
        )
        .unwrap();
        vault.store().unwrap().checkpoint_wal().await.unwrap();
        drop(vault);
        fs::copy(
            security::encrypted_path(&path),
            output.join("turso-0.8.3-encrypted.db.fixture"),
        )
        .unwrap();
        fs::copy(
            security::config_path(&path),
            output.join("vault-v1.security.fixture"),
        )
        .unwrap();
        fs::write(
            output.join("pin-page-clear.json"),
            serde_json::to_vec_pretty(&page).unwrap(),
        )
        .unwrap();
    });
}

#[test]
fn frozen_encrypted_database_backup_and_pin_remain_readable_with_their_original_credentials() {
    use folio::{
        security,
        vault::{Action, Vault},
    };
    const PASSWORD: &str = "Folio fixture master password 2026";
    let fixture: Notebook =
        serde_json::from_str(include_str!("fixtures/notebook-v7.json")).unwrap();
    let clear: Page = serde_json::from_str(include_str!("fixtures/pin-page-clear.json")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old-vault.db");
    fs::write(
        security::encrypted_path(&path),
        include_bytes!("fixtures/turso-0.8.3-encrypted.db.fixture"),
    )
    .unwrap();
    fs::write(
        security::config_path(&path),
        include_bytes!("fixtures/vault-v1.security.fixture"),
    )
    .unwrap();
    fixture
        .install_assets(&workspace::asset_dir(&path))
        .unwrap();
    let backup = dir.path().join("frozen-backup.folio");
    fs::write(
        &backup,
        include_bytes!("fixtures/encrypted-notebook-v1.folio.fixture"),
    )
    .unwrap();
    assert!(security::read_backup(&backup, None, Some("wrong fixture password")).is_err());
    let saved = security::read_backup(&backup, None, Some(PASSWORD)).unwrap();
    runtime().block_on(async {
        let mut vault = Vault::start(path.clone()).await.unwrap();
        assert!(vault.store.is_none());
        vault
            .action(Action::Unlock(zeroize::Zeroizing::new(PASSWORD.into())))
            .await
            .unwrap();
        let all = vault
            .store()
            .unwrap()
            .notebook(&workspace::asset_dir(&path))
            .await
            .unwrap();
        assert_eq!(all, saved);
        assert_eq!(
            all.pages
                .iter()
                .filter(|p| p.id != "fixture-pin")
                .cloned()
                .collect::<Vec<_>>(),
            fixture.pages
        );
        let protected = all.pages.iter().find(|p| p.id == "fixture-pin").unwrap();
        assert!(security::unlock_page(protected, "000000").is_err());
        assert_eq!(
            security::unlock_page(protected, "58391726").unwrap().0,
            clear
        );
        let generated = dir.path().join("new-verified.folio");
        recovery::write_backup(
            vault.store().unwrap(),
            &path,
            &generated,
            vault.config.as_ref().zip(vault.key.as_ref()),
            true,
        )
        .await
        .unwrap();
        BackupProof::verify(&generated).unwrap();
    });
}

#[test]
fn encrypted_fixture_damage_and_future_backup_fields_are_not_accepted() {
    use folio::security;
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("damaged.folio");
    let mut sealed: serde_json::Value = serde_json::from_slice(include_bytes!(
        "fixtures/encrypted-notebook-v1.folio.fixture"
    ))
    .unwrap();
    let cipher = sealed["ciphertext"].as_str().unwrap_or_default();
    // Change an authenticated field without needing the decryption key.
    if cipher.is_empty() {
        sealed["version"] = 99.into();
    } else {
        let mut changed = cipher.as_bytes().to_vec();
        changed[10] = if changed[10] == b'A' { b'B' } else { b'A' };
        sealed["ciphertext"] = String::from_utf8(changed).unwrap().into();
    }
    fs::write(&backup, sealed.to_string()).unwrap();
    assert!(
        security::read_backup(&backup, None, Some("Folio fixture master password 2026")).is_err()
    );
    let mut plain: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/notebook-v7.json")).unwrap();
    plain["future_notebook_metadata"] = true.into();
    fs::write(&backup, plain.to_string()).unwrap();
    assert!(Notebook::read(&backup).is_err());
}

#[test]
fn compatibility_fixtures_match_their_reviewed_byte_fingerprints() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/manifest.json")).unwrap();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for fixture in manifest["fixtures"].as_array().unwrap() {
        let path = root.join(fixture["file"].as_str().unwrap());
        assert_eq!(
            path.metadata().unwrap().len(),
            fixture["size"].as_u64().unwrap()
        );
        assert_eq!(
            folio::updates::hash_file(&path).unwrap(),
            fixture["sha256"].as_str().unwrap()
        );
    }
}

#[test]
fn missing_plain_database_with_existing_profile_files_never_starts_a_blank_notebook() {
    runtime().block_on(async {
        for extension in ["backups", "recovery", "assets"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("missing.db");
            fs::create_dir(path.with_extension(extension)).unwrap();
            assert!(Store::open(&path).await.is_err());
            assert!(!path.exists());
        }
    });
}
#[test]
fn malformed_page_writes_and_checkpoints_keep_the_last_saved_revision() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("writes.db");
    runtime().block_on(async {
        let original = seeded(&path).await;
        let store = Store::open(&path).await.unwrap();
        let mut invalid = original.clone();
        invalid.title = "Should not replace the original".into();
        invalid.blocks.clear();
        assert!(store.save(&invalid).await.is_err());
        assert!(store.checkpoint(&invalid).await.is_err());
        invalid.blocks = vec![original.blocks[0].clone(), original.blocks[0].clone()];
        assert!(store.save(&invalid).await.is_err());
        assert_eq!(store.load_one(&original.id).await.unwrap(), Some(original));
    });
}
#[test]
fn routine_snapshot_rotation_keeps_migration_and_failed_check_copies() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rotation.db");
    fs::write(&path, include_bytes!("fixtures/legacy-v1.sqlite.fixture")).unwrap();
    runtime().block_on(async {
        for index in 0..7 {
            let store = Store::open(&path).await.unwrap();
            let mut page = store.load().await.unwrap().remove(0);
            page.title = format!("Iteration {index}");
            store.save(&page).await.unwrap();
            store.checkpoint_wal().await.unwrap();
        }
        let copies: Vec<_> = fs::read_dir(path.with_extension("recovery"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        let manifests: Vec<serde_json::Value> = copies
            .iter()
            .map(|p| serde_json::from_slice(&fs::read(p.join("manifest.json")).unwrap()).unwrap())
            .collect();
        assert!(manifests.iter().filter(|m| m["pinned"] == false).count() <= 3);
        assert!(copies.iter().any(|p| {
            fs::read(p.join("database"))
                .is_ok_and(|bytes| bytes == include_bytes!("fixtures/legacy-v1.sqlite.fixture"))
        }));
        // A failed preflight gets a retained unverified copy even if later valid opens rotate.
        let (db, c) = raw(&path).await;
        c.execute("PRAGMA user_version=999", ()).await.unwrap();
        flush(&c).await;
        drop(c);
        drop(db);
        let failed = fs::read(&path).unwrap();
        assert!(Store::open(&path).await.is_err());
        let (db, c) = raw(&path).await;
        c.execute("PRAGMA user_version=3", ()).await.unwrap();
        flush(&c).await;
        drop(c);
        drop(db);
        for index in 0..5 {
            let store = Store::open(&path).await.unwrap();
            let mut p = store.load().await.unwrap().remove(0);
            p.title = format!("Valid {index}");
            store.save(&p).await.unwrap();
            store.checkpoint_wal().await.unwrap();
        }
        assert!(
            fs::read_dir(path.with_extension("recovery"))
                .unwrap()
                .filter_map(Result::ok)
                .any(|e| fs::read(e.path().join("database")).is_ok_and(|bytes| bytes == failed))
        );
    });
}

#[test]
fn a_real_second_statement_constraint_failure_rolls_back_title_blocks_and_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atomic-writes.db");
    runtime().block_on(async {
        let original=seeded(&path).await;
        let store=Store::open(&path).await.unwrap();let history=store.history(None).await.unwrap();
        let (db,c)=raw(&path).await;
        c.execute("ALTER TABLE page_content RENAME TO saved_content",()).await.unwrap();
        c.execute("CREATE TABLE page_content(page_id TEXT PRIMARY KEY NOT NULL, content_json TEXT NOT NULL CHECK(content_json NOT LIKE '%reject-content%'))",()).await.unwrap();
        c.execute("INSERT INTO page_content SELECT * FROM saved_content",()).await.unwrap();
        c.execute("DROP TABLE saved_content",()).await.unwrap();
        let mut changed=original.clone();changed.title="Must roll back".into();changed.blocks[0].html="<p>reject-content</p>".into();changed.sync_document();
        let error=store.save(&changed).await.unwrap_err().to_string();assert!(error.contains("constraint")||error.contains("CHECK"),"{error}");
        assert_eq!(store.load_one(&original.id).await.unwrap(),Some(original.clone()));
        assert_eq!(store.history(None).await.unwrap(),history);
        store.checkpoint_wal().await.unwrap();drop(store);drop(c);drop(db);
        let reopened=Store::open(&path).await.unwrap();assert_eq!(reopened.load_one(&original.id).await.unwrap(),Some(original));
    });
}

#[test]
fn history_insert_failure_rolls_back_both_document_writes_and_preserves_prior_revisions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("history-rollback.db");
    runtime().block_on(async {
        let original=seeded(&path).await;let store=Store::open(&path).await.unwrap();let (db,c)=raw(&path).await;
        c.execute("UPDATE page_history SET created_at=0",()).await.unwrap();
        c.execute("ALTER TABLE page_history RENAME TO saved_history",()).await.unwrap();
        c.execute("CREATE TABLE page_history(id TEXT PRIMARY KEY, page_id TEXT NOT NULL, created_at INTEGER NOT NULL CHECK(created_at=0), content_json TEXT NOT NULL)",()).await.unwrap();
        c.execute("INSERT INTO page_history SELECT * FROM saved_history",()).await.unwrap();c.execute("DROP TABLE saved_history",()).await.unwrap();
        c.execute("CREATE INDEX history_page ON page_history(page_id,created_at DESC)",()).await.unwrap();
        let history=store.history(None).await.unwrap();
        let mut changed=original.clone();changed.title="Must not commit".into();changed.blocks[0].html="<p>new content must roll back</p>".into();changed.sync_document();
        assert!(store.save(&changed).await.is_err());
        assert_eq!(store.load_one(&original.id).await.unwrap(),Some(original.clone()));assert_eq!(store.history(None).await.unwrap(),history);
        store.checkpoint_wal().await.unwrap();drop(store);drop(c);drop(db);
        let reopened=Store::open(&path).await.unwrap();assert_eq!(reopened.load_one(&original.id).await.unwrap(),Some(original));assert_eq!(reopened.history(None).await.unwrap(),history);
    });
}
