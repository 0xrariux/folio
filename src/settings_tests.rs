use crate::{
    settings::SecurityForm,
    state::{Phase, State},
    worker::Worker,
};
use rune::{Env, Key, Runtime};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
const PASSWORD: &str = "example master password 2026";
const PIN: &str = "58391726";
fn app(path: &Path) -> Runtime<State> {
    let mut s = State::new();
    s.worker = Some(Worker::start(path.into(), Arc::new(|| {})));
    let mut rt = Runtime::new(s, crate::ui::view, (1200., 900.), Env::new(false, 1.));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.);
    rt
}
fn wait(rt: &mut Runtime<State>, condition: impl Fn(&State) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        rt.update_if(|s, tx| crate::poll(s, 0., tx));
        rt.frame(1. / 60.);
        if condition(&rt.state) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timeout: error={:?}, phase_failed={}, pending={}",
            rt.state.settings.error,
            matches!(rt.state.phase, Phase::Failed(_)),
            rt.state.pending.len()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..35 {
        rt.frame(1. / 60.);
    }
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("missing {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.);
}
fn enter(rt: &mut Runtime<State>, key: &str, value: &str) {
    rt.update(|_, tx| tx.focus(key, false));
    rt.frame(0.);
    rt.key(Key::Text(value.into()).into());
    rt.frame(0.);
}
fn submit(rt: &mut Runtime<State>, value: &str, confirm: bool) {
    enter(rt, "security-secret", value);
    if confirm {
        enter(rt, "security-confirmation", value);
    }
    assert!(
        !rt.accessibility_json().to_string().contains(value),
        "secret exposed in accessibility tree"
    );
    tap(rt, "submit-security");
}
#[test]
fn settings_encrypt_unlock_and_relock_notes_through_the_real_worker_and_restart() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("folio.db");
    let mut rt = app(&path);
    wait(&mut rt, |s| {
        matches!(s.phase, Phase::Ready) && s.workspace.preferences_loaded
    });
    rt.update(|s, _| {
        s.create(false);
        s.title.replace_text("ma-note-intime-xyz");
        s.edit();
    });
    rt.frame(0.);
    let id = rt.state.selected.clone().unwrap();
    rt.update(|s, _| {
        s.editor_mut(&s.active.clone().unwrap())
            .insert("contenu-intime-xyz");
        s.commit_blocks(true);
    });
    rt.frame(0.);
    wait(&mut rt, |s| s.pending.is_empty());
    tap(&mut rt, "settings");
    assert!(rt.find("settings-page").is_some());
    tap(&mut rt, "enable-encryption");
    submit(&mut rt, PASSWORD, true);
    wait(&mut rt, |s| {
        s.settings.encrypted && !s.settings.busy && s.workspace.preferences_loaded
    });
    assert!(!path.exists());
    assert!(folio::security::encrypted_path(&path).is_file());
    rt.update(|s, _| {
        s.select(&id);
        s.protect_current();
    });
    rt.frame(0.);
    submit(&mut rt, PIN, true);
    wait(&mut rt, |s| {
        !s.settings.busy && s.pages.iter().any(|p| p.id == id && p.protected.is_some())
    });
    assert!(rt.find("locked-page").is_some());
    assert!(rt.find("page-title").is_none());
    assert!(
        !rt.accessibility_json()
            .to_string()
            .contains("contenu-intime-xyz")
    );
    tap(&mut rt, "unlock-page");
    submit(&mut rt, "00000000", false);
    wait(&mut rt, |s| !s.settings.busy && s.settings.error.is_some());
    assert!(rt.state.current().unwrap().protected.is_some());
    submit(&mut rt, PIN, false);
    wait(&mut rt, |s| s.settings.unlocked.contains(&id));
    assert_eq!(rt.state.title.text, "ma-note-intime-xyz");
    tap(&mut rt, "library");
    wait(&mut rt, |s| {
        !s.settings.busy && !s.settings.unlocked.contains(&id)
    });
    assert!(rt.find("library-page").is_some());
    assert!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .protected
            .is_some()
    );
    assert!(
        !rt.accessibility_json()
            .to_string()
            .contains("contenu-intime-xyz")
    );
    tap(&mut rt, "library-close");
    tap(&mut rt, "unlock-page");
    submit(&mut rt, PIN, false);
    wait(&mut rt, |s| s.settings.unlocked.contains(&id));
    tap(&mut rt, "settings");
    wait(&mut rt, |s| {
        !s.settings.busy && s.pages.iter().any(|p| p.id == id && p.protected.is_some())
    });
    assert!(
        rt.find("settings-page").is_some(),
        "async relocking must preserve the settings page"
    );
    tap(&mut rt, "close-settings");
    tap(&mut rt, "unlock-page");
    submit(&mut rt, PIN, false);
    wait(&mut rt, |s| s.settings.unlocked.contains(&id));
    rt.update(|s, _| s.create(false));
    rt.frame(0.);
    assert!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .protected
            .is_some(),
        "navigation must hide plaintext before the async lock acknowledgement"
    );
    wait(&mut rt, |s| s.pending.is_empty());
    rt.update(|s, _| s.lock_notebook());
    wait(&mut rt, |s| s.settings.locked && !s.settings.busy);
    assert!(rt.state.pages.is_empty());
    assert!(rt.find("locked-notebook").is_some());
    drop(rt);
    let mut reopened = app(&path);
    wait(&mut reopened, |s| s.settings.locked);
    assert!(reopened.state.pages.is_empty());
    submit(&mut reopened, "mauvais mot de passe", false);
    wait(&mut reopened, |s| {
        !s.settings.busy && s.settings.error.is_some()
    });
    assert!(reopened.state.pages.is_empty());
    submit(&mut reopened, PASSWORD, false);
    wait(&mut reopened, |s| {
        !s.settings.locked && matches!(s.phase, Phase::Ready)
    });
    assert!(
        reopened
            .state
            .pages
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .protected
            .is_some()
    );
    assert!(
        !reopened
            .state
            .pages
            .iter()
            .any(|p| p.plain_text.contains("contenu-intime-xyz"))
    );
}
#[test]
fn settings_and_pin_controls_exist_at_the_minimum_window_size_without_network_requests() {
    let mut s = State::demo();
    s.open_settings();
    let mut rt = Runtime::new(s, crate::ui::view, (920., 620.), Env::new(false, 1.));
    rt.frame(0.);
    for key in ["settings", "settings-page", "check-version"] {
        assert!(
            rt.elements().iter().any(|e| e.key.as_deref() == Some(key)),
            "{key}"
        );
    }
    assert!(!rt.state.settings.checking);
    assert!(rt.state.settings.update_task.is_none());
    rt.update(|s, _| s.security_form(SecurityForm::EnableVault));
    rt.frame(0.);
    assert!(
        rt.elements()
            .iter()
            .any(|e| e.key.as_deref() == Some("security-secret"))
    );
}
#[test]
fn export_destinations_cannot_overwrite_vault_database_configuration_or_wal() {
    let temp = tempfile::tempdir().unwrap();
    let database = temp.path().join("folio.db");
    for path in folio::security::storage_files(&database) {
        assert!(
            crate::workspace_worker::ensure_destination(&database, &path).is_err(),
            "{}",
            path.display()
        );
    }
}

#[test]
fn cancelling_a_security_form_erases_its_inputs() {
    let mut s = State::demo();
    s.security_form(SecurityForm::EnableVault);
    s.settings.secret.replace_text("temporary-secret-value");
    s.settings
        .confirmation
        .replace_text("temporary-secret-value");
    s.cancel_security();
    assert!(s.settings.secret.text.is_empty());
    assert!(s.settings.confirmation.text.is_empty());
    assert!(s.settings.form.is_none());
}

#[test]
fn permanent_deletion_requires_confirmation_in_the_page_menu() {
    let mut s = State::demo();
    s.create(false);
    let id = s.selected.clone().unwrap();
    let mut rt = Runtime::new(s, crate::ui::view, (1200., 900.), Env::new(false, 1.));
    rt.frame(0.);
    tap(&mut rt, "page-menu");
    tap(&mut rt, "archive-page");
    tap(&mut rt, "page-menu");
    tap(&mut rt, "delete-page");
    assert!(rt.state.pages.iter().any(|p| p.id == id));
    tap(&mut rt, "cancel-delete-page");
    assert!(rt.state.pages.iter().any(|p| p.id == id));
    tap(&mut rt, "page-menu");
    tap(&mut rt, "delete-page");
    tap(&mut rt, "confirm-delete-page");
    assert!(!rt.state.pages.iter().any(|p| p.id == id));
}

#[test]
fn navigation_lock_acknowledgements_do_not_clear_another_password_form() {
    let mut s = State::demo();
    s.security_form(SecurityForm::EnableVault);
    s.settings
        .secret
        .replace_text("password currently being submitted");
    s.settings.busy = true;
    s.settings.pending_security = Some(crate::settings::SecurityOperation::Vault);
    let page = folio::model::Page::new(None);
    s.accept_security(folio::vault::Outcome::Page {
        page: Box::new(page),
        unlocked: false,
        warning: None,
    });
    assert!(s.settings.busy);
    assert_eq!(s.settings.secret.text, "password currently being submitted");
    assert!(matches!(s.panel, crate::state::Panel::Security));
    s.accept_security(folio::vault::Outcome::Opened {
        encrypted: true,
        warning: None,
    });
    assert!(!s.settings.busy);
    assert!(s.settings.secret.text.is_empty());
}

#[test]
fn sidebar_and_header_delete_active_pages_only_after_confirmation_and_preserve_children() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("delete.db");
    let mut rt = app(&path);
    wait(&mut rt, |s| {
        matches!(s.phase, Phase::Ready) && s.workspace.preferences_loaded
    });
    rt.update(|s, _| s.create(false));
    let parent = rt.state.selected.clone().unwrap();
    rt.update(|s, _| {
        s.title.replace_text("Disposable parent");
        s.edit();
        s.create(true);
    });
    let child = rt.state.selected.clone().unwrap();
    rt.update(|s, _| s.create(false));
    rt.frame(0.);
    wait(&mut rt, |s| s.pending.is_empty());
    tap(&mut rt, &format!("page-actions-{parent}"));
    assert_eq!(rt.state.selected.as_deref(), Some(parent.as_str()));
    tap(&mut rt, "delete-page");
    assert!(!rt.state.current().unwrap().archived);
    assert!(rt.state.pages.iter().any(|p| p.id == parent));
    tap(&mut rt, "cancel-delete-page");
    assert!(rt.state.pages.iter().any(|p| p.id == parent && !p.archived));
    tap(&mut rt, "page-menu");
    tap(&mut rt, "delete-page");
    tap(&mut rt, "confirm-delete-page");
    rt.update(|s, _| s.confirm_delete()); // A second press while busy cannot enqueue another deletion.
    wait(&mut rt, |s| {
        !s.workspace.busy && s.pending.is_empty() && !s.pages.iter().any(|p| p.id == parent)
    });
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == child)
            .unwrap()
            .parent_id,
        None
    );
    drop(rt);
    let mut reopened = app(&path);
    wait(&mut reopened, |s| {
        matches!(s.phase, Phase::Ready) && s.workspace.preferences_loaded
    });
    assert!(!reopened.state.pages.iter().any(|p| p.id == parent));
    assert_eq!(
        reopened
            .state
            .pages
            .iter()
            .find(|p| p.id == child)
            .unwrap()
            .parent_id,
        None
    );
}

#[test]
fn sidebar_shortcuts_reorder_hide_restore_and_persist_without_changing_notes() {
    use folio::workspace::SidebarShortcut;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("navigation.db");
    let mut rt = app(&path);
    wait(&mut rt, |s| {
        matches!(s.phase, Phase::Ready) && s.workspace.preferences_loaded
    });
    let pages = rt.state.pages.clone();
    tap(&mut rt, "customize-navigation");
    tap(&mut rt, "shortcut-up-favorites");
    assert_eq!(
        rt.state.workspace.preferences.sidebar_shortcuts,
        [
            SidebarShortcut::Overview,
            SidebarShortcut::Favorites,
            SidebarShortcut::AllPages,
            SidebarShortcut::Archives
        ]
    );
    rt.key(Key::Escape.into());
    for _ in 0..35 {
        rt.frame(1. / 60.);
    }
    assert!(rt.find("favorites").unwrap().1 < rt.find("all-pages").unwrap().1);
    tap(&mut rt, "favorites");
    tap(&mut rt, "customize-navigation");
    tap(&mut rt, "shortcut-hide-favorites");
    assert!(matches!(rt.state.filter, crate::state::Filter::All));
    for shortcut in ["overview", "all_pages", "archives"] {
        tap(&mut rt, &format!("shortcut-hide-{shortcut}"));
    }
    assert!(rt.state.workspace.preferences.sidebar_shortcuts.is_empty());
    assert!(rt.find("customize-navigation").is_some());
    assert_eq!(rt.state.pages, pages);
    drop(rt);

    let mut reopened = app(&path);
    wait(&mut reopened, |s| {
        matches!(s.phase, Phase::Ready) && s.workspace.preferences_loaded
    });
    assert!(
        reopened
            .state
            .workspace
            .preferences
            .sidebar_shortcuts
            .is_empty()
    );
    for key in ["open-dashboard", "all-pages", "favorites", "archive-filter"] {
        assert!(
            reopened.find(key).is_none(),
            "{key} should stay hidden after restart"
        );
    }
    tap(&mut reopened, "customize-navigation");
    tap(&mut reopened, "shortcut-add-overview");
    tap(&mut reopened, "shortcut-add-favorites");
    reopened.key(Key::Escape.into());
    for _ in 0..35 {
        reopened.frame(1. / 60.);
    }
    let (x, y) = reopened.find("favorites").unwrap();
    let (dx, dy) = reopened.find("shortcut-before-overview").unwrap();
    reopened.pointer_down(x, y);
    reopened.pointer_move(dx, dy);
    reopened.frame(0.);
    reopened.pointer_up(dx, dy);
    reopened.frame(0.);
    assert_eq!(
        reopened.state.workspace.preferences.sidebar_shortcuts,
        [SidebarShortcut::Favorites, SidebarShortcut::Overview]
    );
    assert_eq!(reopened.state.pages, pages);
    drop(reopened);

    let mut reordered = app(&path);
    wait(&mut reordered, |s| {
        matches!(s.phase, Phase::Ready) && s.workspace.preferences_loaded
    });
    assert_eq!(
        reordered.state.workspace.preferences.sidebar_shortcuts,
        [SidebarShortcut::Favorites, SidebarShortcut::Overview]
    );
    for _ in 0..35 {
        reordered.frame(1. / 60.);
    }
    assert!(reordered.find("favorites").unwrap().1 < reordered.find("open-dashboard").unwrap().1);
    tap(&mut reordered, "customize-navigation");
    tap(&mut reordered, "reset-shortcuts");
    assert_eq!(
        reordered.state.workspace.preferences.sidebar_shortcuts,
        SidebarShortcut::defaults()
    );
}

#[test]
fn a_backup_failure_cannot_be_reversed_by_a_late_success_event() {
    use crate::workspace_worker::Response;
    let mut s = State::new();
    s.settings.update_backup_pending = true;
    s.workspace.busy = true;
    s.accept_workspace(Ok(Response::Failure {
        message: "disk full".into(),
        blocking: true,
    }));
    assert!(!s.settings.update_backup_pending);
    assert!(!s.workspace.busy);
    s.accept_workspace(Ok(Response::UpdateBackupDone("unused.folio".into())));
    assert!(!s.settings.install_when_saved);
    assert!(s.settings.update_backup.is_none());
}
