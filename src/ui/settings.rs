use super::{Palette, action, label};
use crate::{
    settings::SecurityForm,
    state::{Panel, State},
};
use folio::{icons::controls as g, updates};
use rune::{El, ScrollSpec, TextStyle, col, el, row, text_input};
fn section(title: &str, p: Palette) -> El {
    col()
        .w_full()
        .gap(12.)
        .padding(18.)
        .radius(8.)
        .border(1., p.line)
        .fill(p.bg)
        .child(
            label(title, 17., p.ink)
                .role("heading")
                .level(2)
                .label(title),
        )
}
fn paragraph(value: impl Into<String>, p: Palette) -> El {
    label(value, 12., p.muted).w_full().wrap()
}
pub(super) fn view(s: &State, p: Palette) -> El {
    let current_revision = &updates::REVISION[..updates::REVISION.len().min(12)];
    let build = if env!("FOLIO_BUILD_DIRTY") == "true" {
        " · Local build"
    } else {
        ""
    };
    let mut versions = section("Updates", p).child(
        paragraph(format!("Folio {}{build}", updates::VERSION), p)
            .help(format!("Git revision: {current_revision}")),
    );
    if let Some(remote) = &s.settings.remote {
        versions = versions.child(paragraph(format!("GitHub {}", remote.version), p));
    }
    if !s.settings.update_status.is_empty() {
        versions = versions.child(paragraph(&s.settings.update_status, p));
    }
    let mut controls = row().gap(10.).flex_wrap().child(
        action("check-version", "Check for updates", p, |s, _| {
            s.check_version()
        })
        .fill(p.tint)
        .disabled(s.settings.checking || s.settings.update_task.is_some()),
    );
    if s.settings.remote.as_ref().is_some_and(|r| r.ahead) && s.settings.ready.is_none() {
        controls = controls.child(
            action("prepare-update", "Update", p, |s, _| s.prepare_update())
                .help("Download a signed release. Your notebook is backed up before installation.")
                .fill(p.tint)
                .disabled(s.demo || s.settings.locked || s.settings.update_task.is_some()),
        );
    }
    if let Some(remote) = &s.settings.remote {
        let url = format!(
            "https://github.com/{}/releases/tag/v{}",
            updates::config().repository,
            remote.version
        );
        controls = controls.child(action("release-notes", "What's new", p, move |s, tx| {
            if let Err(error) = tx.open_url(&url) {
                s.settings.update_status = format!("Cannot open release notes: {error}");
            }
        }));
    }
    versions = versions.child(controls);
    if s.settings.ready.is_some() {
        versions = versions.child(
            action("install-update", "Install and restart", p, |s, tx| {
                s.install_update(tx)
            })
            .fill(p.tint),
        );
    }
    let database = folio::storage::database_path();
    let database = if s.settings.encrypted {
        folio::security::encrypted_path(&database)
    } else {
        database
    };
    let data = section("Data", p)
        .child(paragraph("Stored on this device", p).help(database.display().to_string()))
        .child(
            row()
                .gap(10.)
                .flex_wrap()
                .child(
                    action("settings-backup", "Back up", p, |s, tx| s.backup_dialog(tx))
                        .disabled(s.demo || s.settings.locked),
                )
                .child(
                    action("settings-restore", "Restore…", p, |s, tx| {
                        s.restore_dialog(tx)
                    })
                    .disabled(s.demo || s.settings.locked),
                ),
        );
    let mut security = section("Security", p);
    security = if s.settings.encrypted {
        security
            .child(paragraph(
                if s.settings.locked {
                    "Encrypted · Locked"
                } else {
                    "Encrypted"
                },
                p,
            ))
            .child(
                action("lock-notebook", "Lock notebook", p, |s, _| {
                    s.lock_notebook()
                })
                .fill(p.tint)
                .disabled(s.settings.locked || s.settings.busy),
            )
    } else {
        security.child(paragraph("Encryption off", p)).child(
            action("enable-encryption", "Enable encryption…", p, |s, tx| {
                s.security_form(SecurityForm::EnableVault);
                tx.focus("security-secret", false);
            })
            .fill(p.tint)
            .disabled(s.demo || s.settings.busy),
        )
    };
    security = security.child(paragraph("Experimental. Attachments are not encrypted.", p));
    if let Some(error) = &s.settings.error {
        security = security.child(label(error, 12., p.ink).role("alert").w_full().wrap());
    }
    if s.settings.busy {
        security = security.child(paragraph("Please wait…", p));
    }
    col()
        .key("settings-page")
        .grow(1.)
        .min_h(0.)
        .w_full()
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(
            col()
                .w_full()
                .max_w(900.)
                .padding(28.)
                .gap(18.)
                .child(
                    row()
                        .center_y()
                        .gap(12.)
                        .child(rune::icon(&g::SETTINGS, 26.).color(p.accent).a11y_hidden())
                        .child(
                            label("Settings", 28., p.ink)
                                .role("heading")
                                .level(1)
                                .label("Settings"),
                        )
                        .child(el().grow(1.))
                        .child(action(
                            "close-settings",
                            if s.settings.locked {
                                "Unlock notebook"
                            } else {
                                "Back to notes"
                            },
                            p,
                            |s, _| s.panel = Panel::Closed,
                        )),
                )
                .child(versions)
                .child(data)
                .child(security),
        )
}
fn form(s: &State, p: Palette) -> El {
    let mode = s.settings.form.as_ref();
    let confirm = matches!(
        mode,
        Some(SecurityForm::EnableVault | SecurityForm::ProtectPage(_))
    );
    let pin = matches!(
        mode,
        Some(SecurityForm::ProtectPage(_) | SecurityForm::UnlockPage(_))
    );
    let hint = match mode {
        Some(SecurityForm::EnableVault) => {
            Some("12+ characters. No password recovery. Close other Folio windows first.")
        }
        Some(SecurityForm::ProtectPage(_)) => Some("6+ digits. Longer is safer. No PIN recovery."),
        Some(SecurityForm::UnlockBackup(_)) => Some("Use this backup’s password."),
        _ => None,
    };
    let mut body = col().w_full().gap(12.);
    if let Some(hint) = hint {
        body = body.child(paragraph(hint, p));
    }
    body = body
        .child(label(
            if pin { "PIN" } else { "Master password" },
            12.,
            p.muted,
        ))
        .child(
            text_input(&s.settings.secret, TextStyle::new(16., 400.))
                .key("security-secret")
                .label(if pin { "PIN" } else { "Master password" })
                .secure()
                .single_line()
                .w_full()
                .h(42.)
                .px(10.)
                .radius(5.)
                .border(1., p.line)
                .color(p.ink)
                .fill(p.sidebar)
                .on_edit(|s: &mut State| &mut s.settings.secret)
                .on_submit(|s: &mut State, _| s.submit_security())
                .disabled(s.settings.busy),
        );
    if confirm {
        body = body
            .child(label(
                if pin {
                    "Confirm PIN"
                } else {
                    "Confirm password"
                },
                12.,
                p.muted,
            ))
            .child(
                text_input(&s.settings.confirmation, TextStyle::new(16., 400.))
                    .key("security-confirmation")
                    .label("Confirm code")
                    .secure()
                    .single_line()
                    .w_full()
                    .h(42.)
                    .px(10.)
                    .radius(5.)
                    .border(1., p.line)
                    .color(p.ink)
                    .fill(p.sidebar)
                    .on_edit(|s: &mut State| &mut s.settings.confirmation)
                    .on_submit(|s: &mut State, _| s.submit_security())
                    .disabled(s.settings.busy),
            );
    }
    if let Some(error) = &s.settings.error {
        body = body.child(label(error, 12., p.ink).role("alert").w_full().wrap());
    }
    body.child(
        action(
            "submit-security",
            if s.settings.busy {
                "Please wait…"
            } else if confirm {
                "Enable encryption"
            } else {
                "Unlock"
            },
            p,
            |s, _| s.submit_security(),
        )
        .fill(p.tint)
        .disabled(s.settings.busy),
    )
}
pub(super) fn security_dialog(s: &State, p: Palette) -> El {
    let title = match s.settings.form {
        Some(SecurityForm::ProtectPage(_)) => "Protect this page",
        Some(SecurityForm::UnlockPage(_)) => "Unlock this page",
        Some(SecurityForm::UnlockBackup(_)) => "Open encrypted backup",
        _ => "Notebook protection",
    };
    super::workspace::dialog(title, p, form(s, p))
}
pub(super) fn locked(s: &State, p: Palette) -> El {
    col()
        .key("locked-notebook")
        .w_full()
        .h_full()
        .fill(p.bg)
        .center()
        .padding(24.)
        .child(
            col()
                .w(530.)
                .max_w(530.)
                .gap(18.)
                .padding(24.)
                .fill(p.bg)
                .border(1., p.line)
                .radius(10.)
                .child(rune::icon(&g::LOCK, 30.).color(p.accent).a11y_hidden())
                .child(
                    label("Your notebook is locked", 24., p.ink)
                        .role("heading")
                        .level(1)
                        .label("Your notebook is locked"),
                )
                .child(form(s, p))
                .child(action(
                    "locked-settings",
                    "Settings and version",
                    p,
                    |s, _| s.open_settings(),
                )),
        )
}
pub(super) fn locked_page(_s: &State, page: &folio::model::Page, p: Palette) -> El {
    let id = page.id.clone();
    col()
        .key("locked-page")
        .grow(1.)
        .center()
        .gap(16.)
        .padding(24.)
        .child(rune::icon(&g::LOCK, 32.).color(p.accent).a11y_hidden())
        .child(
            label("Protected page", 24., p.ink)
                .role("heading")
                .level(1)
                .label("Protected page"),
        )
        .child(paragraph("Enter the PIN to access this page.", p))
        .child(
            action("unlock-page", "Unlock this page", p, move |s, tx| {
                s.security_form(SecurityForm::UnlockPage(id.clone()));
                tx.focus("security-secret", false);
            })
            .fill(p.tint),
        )
        .child(action("locked-page-menu", "Page options", p, |s, _| {
            s.panel = Panel::Page
        }))
}
