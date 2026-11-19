use folio::{
    document::{Kind, NoteBlock},
    model::Page,
    security::{self, Sealed},
    storage::Store,
    vault::{Action, Outcome, Vault},
    workspace,
};
use zeroize::Zeroizing;
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
fn secret(value: &str) -> Zeroizing<String> {
    Zeroizing::new(value.into())
}
const PASSWORD: &str = "principal de test unique 2026";
const PIN: &str = "73916284";
#[test]
fn vault_and_pin_protect_notes_history_backups_and_reopen_without_plaintext() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    runtime().block_on(async {
        let mut vault = Vault::start(path.clone()).await.unwrap();
        let mut page = Page::new(None);
        page.title = "titre-prive-xyz".into();
        page.emoji = "folio:heart".into();
        page.icon_color = folio::icons::IconColor::Rose;
        page.blocks = vec![NoteBlock::text(Kind::Text, "contenu-prive-abc")];
        page.sync_document();
        vault.save(&page).await.unwrap();
        vault.store().unwrap().checkpoint(&page).await.unwrap();
        let backups = path.with_extension("backups");
        std::fs::create_dir_all(&backups).unwrap();
        let backup = backups.join("ancien.folio");
        vault
            .store()
            .unwrap()
            .notebook(&workspace::asset_dir(&path))
            .await
            .unwrap()
            .write(&backup)
            .unwrap();
        assert!(matches!(
            vault
                .action(Action::Enable(secret(PASSWORD)))
                .await
                .unwrap(),
            Outcome::Opened {
                encrypted: true,
                ..
            }
        ));
        assert!(
            !path.exists(),
            "the plaintext database must be removed after a verified migration"
        );
        assert!(security::encrypted_path(&path).is_file());
        for file in [
            security::encrypted_path(&path),
            std::path::PathBuf::from(format!("{}-wal", security::encrypted_path(&path).display())),
        ] {
            if let Ok(bytes) = std::fs::read(file) {
                assert!(
                    !bytes
                        .windows(b"contenu-prive-abc".len())
                        .any(|b| b == b"contenu-prive-abc")
                );
            }
        }
        // Check the native database itself, independently of per-page PIN encryption.
        drop(vault.store.take());
        assert!(Store::open(&security::encrypted_path(&path)).await.is_err());
        vault
            .action(Action::Unlock(secret(PASSWORD)))
            .await
            .unwrap();

        let key = vault.key.as_ref().unwrap().clone();
        assert_eq!(
            security::read_backup(&backup, Some(&key), None)
                .unwrap()
                .pages[0],
            page
        );
        vault
            .action(Action::Protect {
                id: page.id.clone(),
                pin: secret(PIN),
            })
            .await
            .unwrap();
        let locked = vault
            .store()
            .unwrap()
            .load_one(&page.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(locked.title, "Protected page");
        assert!(!locked.matches("contenu-prive"));
        assert!(locked.protected.is_some());
        assert!(security::unlock_page(&locked, "11111111").is_err());
        assert_eq!(security::unlock_page(&locked, PIN).unwrap().0, page);
        let history = vault
            .store()
            .unwrap()
            .history(Some(&page.id))
            .await
            .unwrap();
        assert!(!history.is_empty());
        assert!(history.iter().all(|h| h.page.protected.is_some()));
        let backed = security::read_backup(&backup, Some(&key), None).unwrap();
        assert!(backed.pages[0].protected.is_some());
        assert!(backed.history.iter().all(|h| h.page.protected.is_some()));
        assert!(
            !folio::export::markdown(&locked, std::slice::from_ref(&locked))
                .contents
                .contains("contenu-prive")
        );
        let Outcome::Page {
            mut page,
            unlocked: true,
            ..
        } = vault
            .action(Action::UnlockPage {
                id: page.id.clone(),
                pin: secret(PIN),
            })
            .await
            .unwrap()
        else {
            panic!("unlocked page expected")
        };
        page.blocks[0] = NoteBlock::text(Kind::Text, "nouvelle-version-secrete");
        page.sync_document();
        vault.save(&page).await.unwrap();
        vault
            .action(Action::LockPage(page.id.clone()))
            .await
            .unwrap();
        assert!(
            vault.save(&page).await.is_err(),
            "no plaintext save may overwrite a locked note"
        );
        vault.action(Action::Lock).await.unwrap();
        assert!(vault.key.is_none());
        assert!(vault.store.is_none());
        drop(vault);
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                let bytes = std::fs::read(entry.path()).unwrap();
                for marker in [
                    "titre-prive-xyz",
                    "contenu-prive-abc",
                    "nouvelle-version-secrete",
                ] {
                    assert!(
                        !bytes.windows(marker.len()).any(|b| b == marker.as_bytes()),
                        "plaintext leaked in {:?}",
                        entry.path()
                    );
                }
            }
        }
        // A legacy application could recreate this name after encryption. Preserve it.
        std::fs::write(&path, b"separate legacy notebook").unwrap();
        let mut reopened = Vault::start(path.clone()).await.unwrap();
        assert!(reopened.store.is_none());
        assert!(
            reopened
                .action(Action::Unlock(secret("mot de passe incorrect")))
                .await
                .is_err()
        );
        let outcome = reopened
            .action(Action::Unlock(secret(PASSWORD)))
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            Outcome::Opened {
                warning: Some(_),
                ..
            }
        ));
        assert_eq!(std::fs::read(&path).unwrap(), b"separate legacy notebook");
        let saved = reopened
            .store()
            .unwrap()
            .load_one(&page.id)
            .await
            .unwrap()
            .unwrap();
        assert!(saved.protected.is_some());
        assert_eq!(
            security::unlock_page(&saved, PIN).unwrap().0,
            page.as_ref().clone()
        );
        // A portable encrypted backup carries its own salt and opens with the original password.
        assert!(security::read_backup(&backup, None, Some(PASSWORD)).is_ok());
        assert!(security::read_backup(&backup, None, Some("wrong")).is_err());
    });
}
#[test]
fn authenticated_envelopes_reject_tampering_transplantation_and_short_pins() {
    assert!(security::new_page_key("1234").is_err());
    let keys = security::new_page_key(PIN).unwrap();
    let mut page = Page::new(None);
    page.title = "secret".into();
    let mut locked = security::seal_page(&page, &keys).unwrap();
    locked.id = "another-page".into();
    assert!(security::open_page(&locked, &keys).is_err());
    let mut locked = security::seal_page(&page, &keys).unwrap();
    let sealed = locked.protected.as_mut().unwrap();
    sealed.ciphertext.replace_range(0..1, "!");
    assert!(security::open_page(&locked, &keys).is_err());
    let config = security::VaultConfig::create(PASSWORD).unwrap().0;
    let encoded = serde_json::to_string(&config).unwrap();
    assert!(!encoded.contains(PASSWORD));
    let mut sealed: Sealed =
        serde_json::from_str(&serde_json::to_string(&config.verifier).unwrap()).unwrap();
    sealed.version = 999;
    assert!(security::open(&sealed, &keys.key, b"test").is_err());
}
#[test]
fn an_intentionally_empty_notebook_stays_empty_and_a_missing_vault_never_gets_seeded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        store.load_or_seed().await.unwrap();
        let mut notebook = store.notebook(&workspace::asset_dir(&path)).await.unwrap();
        notebook.pages.clear();
        notebook.history.clear();
        store
            .restore_notebook(&notebook, &workspace::asset_dir(&path))
            .await
            .unwrap();
        drop(store);
        assert!(
            Store::open(&path)
                .await
                .unwrap()
                .load_or_seed()
                .await
                .unwrap()
                .is_empty()
        );
        let config = security::VaultConfig::create(PASSWORD).unwrap().0;
        config.write(&path).unwrap();
        assert!(Vault::start(path.clone()).await.is_err());
        assert!(!security::encrypted_path(&path).exists());
    });
}

#[test]
fn deleting_the_last_archived_page_does_not_recreate_welcome_and_preserves_children() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("notes.db");
    runtime().block_on(async {
        let store = Store::open(&path).await.unwrap();
        let mut page = store.load_or_seed().await.unwrap().remove(0);
        assert!(store.delete_archived(&page.id).await.is_err());
        page.archived = true;
        store.save(&page).await.unwrap();
        let child = Page::new(Some(page.id.clone()));
        store.save(&child).await.unwrap();
        store.delete_archived(&page.id).await.unwrap();
        let mut child = store.load_one(&child.id).await.unwrap().unwrap();
        assert!(child.parent_id.is_none());
        child.archived = true;
        store.save(&child).await.unwrap();
        store.delete_archived(&child.id).await.unwrap();
        drop(store);
        assert!(
            Store::open(&path)
                .await
                .unwrap()
                .load_or_seed()
                .await
                .unwrap()
                .is_empty()
        );
    });
}
#[test]
fn lost_vault_configuration_does_not_create_a_plain_empty_notebook() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("notes.db");
    std::fs::write(security::encrypted_path(&path), b"existing vault").unwrap();
    runtime().block_on(async {
        assert!(Vault::start(path.clone()).await.is_err());
    });
    assert!(!path.exists());
    assert_eq!(
        std::fs::read(security::encrypted_path(&path)).unwrap(),
        b"existing vault"
    );
}
