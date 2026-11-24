use super::{Palette, action, label, management as layout};
use crate::{
    settings::{SecurityForm, SettingsPage},
    state::{Panel, State},
};
use folio::{
    icons,
    icons::controls as g,
    updates,
    workspace::{DarkPalette, LightPalette},
};
use rune::{El, Key, TextStyle, col, el, row, text_input};

fn paragraph(value: impl Into<String>, p: Palette) -> El {
    label(value, 12., p.muted).w_full().wrap()
}

fn section(key: &str, title: &str, description: &str, p: Palette) -> El {
    col().key(key).w_full().min_w(0.).gap(22.).child(
        col()
            .w_full()
            .gap(8.)
            .child(
                label(title, 28., p.ink)
                    .font(TextStyle::new(28., 600.))
                    .role("heading")
                    .level(1),
            )
            .child(paragraph(description, p)),
    )
}

fn setting_row(title: &str, description: &str, controls: El, p: Palette) -> El {
    row()
        .w_full()
        .min_w(0.)
        .py(22.)
        .gap(24.)
        .flex_wrap()
        .center_y()
        .border_sides([1., 0., 0., 0.], p.line)
        .child(
            col()
                .grow(1.)
                .min_w(240.)
                .gap(7.)
                .child(label(title, 14., p.ink).font(TextStyle::new(14., 550.)))
                .child(paragraph(description, p)),
        )
        .child(controls)
}

pub(super) fn view(s: &State, p: Palette, width: f32) -> El {
    let top = row()
        .w_full()
        .min_w(0.)
        .px(28.)
        .py(20.)
        .gap(12.)
        .center_y()
        .child(
            col()
                .size(36., 36.)
                .radius(11.)
                .fill(p.tint)
                .center()
                .child(rune::icon(&g::SETTINGS, 19.).color(p.accent).a11y_hidden()),
        )
        .child(label("Settings", 18., p.ink).font(TextStyle::new(18., 600.)))
        .child(label("/", 16., p.faint).a11y_hidden())
        .child(
            label(s.settings.page.title(), 12., p.muted)
                .grow(1.)
                .min_w(0.)
                .truncate(),
        )
        .child(
            super::icons::action(
                "close-settings",
                if s.settings.locked {
                    "Unlock notebook"
                } else {
                    "Back to notes"
                },
                &g::CHEVRON_LEFT,
                p,
                |s, _| s.panel = Panel::Closed,
            )
            .h(34.)
            .border(1., p.line)
            .radius(7.),
        );
    let mut navigation = row()
        .key("settings-navigation")
        .role("navigation")
        .label("Settings pages")
        .w_full()
        .min_w(0.)
        .px(28.)
        .py(8.)
        .gap(8.)
        .flex_wrap();
    for page in SettingsPage::ALL {
        let selected = s.settings.page == page;
        let glyph = match page {
            SettingsPage::Notebook => &icons::DATABASE,
            SettingsPage::Security => &g::LOCK,
            SettingsPage::Appearance => &icons::PALETTE,
            SettingsPage::Updates => &g::ARROW_DOWN,
        };
        navigation = navigation.child(
            super::button(
                format!("settings-nav-{}", page.key()),
                page.title(),
                p,
                move |s, tx| {
                    s.settings.page = page;
                    tx.scroll_to_y("settings-page-scroll", 0.);
                },
            )
            .on_key(move |s: &mut State, event, tx| {
                let at = SettingsPage::ALL
                    .iter()
                    .position(|item| *item == page)
                    .unwrap();
                let destination = match event.key {
                    Key::Enter => page,
                    Key::Left => SettingsPage::ALL[(at + 3) % 4],
                    Key::Right => SettingsPage::ALL[(at + 1) % 4],
                    Key::Home => SettingsPage::Notebook,
                    Key::End => SettingsPage::Updates,
                    _ => return false,
                };
                s.settings.page = destination;
                tx.focus(format!("settings-nav-{}", destination.key()), true);
                tx.scroll_to_y("settings-page-scroll", 0.);
                true
            })
            .h(40.)
            .px(14.)
            .gap(8.)
            .radius(6.)
            .checked(selected)
            .fill(if selected {
                p.tint
            } else {
                rune::Color::TRANSPARENT
            })
            .border_sides([0., 0., if selected { 2. } else { 0. }, 0.], p.accent)
            .child(
                rune::icon(glyph, 15.)
                    .color(if selected { p.accent } else { p.muted })
                    .a11y_hidden(),
            )
            .child(label(
                page.title(),
                12.,
                if selected { p.ink } else { p.muted },
            )),
        );
    }
    let header = col()
        .key("settings-header")
        .w_full()
        .min_w(0.)
        .fill(p.bg)
        .border_sides([0., 0., 1., 0.], p.line)
        .child(top)
        .child(navigation);
    let content = match s.settings.page {
        SettingsPage::Notebook => notebook(s, p),
        SettingsPage::Security => security(s, p),
        SettingsPage::Appearance => appearance(s, p, width),
        SettingsPage::Updates => updates(s, p),
    };
    layout::page("settings-page", header, content, p)
}

fn updates(s: &State, p: Palette) -> El {
    let current_revision = &updates::REVISION[..updates::REVISION.len().min(12)];
    let mut versions = section(
        "settings-updates",
        "Updates",
        "The latest improvements, ready for your notebook.",
        p,
    )
    .child(
        row()
            .gap(10.)
            .center_y()
            .child(label("Folio", 22., p.ink).font(TextStyle::new(22., 600.)))
            .child(
                layout::badge(format!("v{}", updates::VERSION), true, p).help(format!(
                    "Git revision: {current_revision}{}",
                    if env!("FOLIO_BUILD_DIRTY") == "true" {
                        " · Local build"
                    } else {
                        ""
                    }
                )),
            ),
    );
    if let Some(remote) = &s.settings.remote {
        versions = versions.child(paragraph(
            format!("Available release · {}", remote.version),
            p,
        ));
    }
    if !s.settings.update_status.is_empty() {
        versions = versions.child(paragraph(&s.settings.update_status, p));
    }
    let mut controls = row().gap(8.).flex_wrap().child(
        action("check-version", "Check for updates", p, |s, _| {
            s.check_version()
        })
        .h(34.)
        .fill(p.tint)
        .disabled(s.settings.checking || s.settings.update_task.is_some()),
    );
    if s.settings.remote.as_ref().is_some_and(|r| r.ahead) && s.settings.ready.is_none() {
        controls = controls.child(
            action("prepare-update", "Update", p, |s, _| s.prepare_update())
                .h(34.)
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
        controls = controls.child(
            action("release-notes", "What's new", p, move |s, tx| {
                if let Err(error) = tx.open_url(&url) {
                    s.settings.update_status = format!("Cannot open release notes: {error}");
                }
            })
            .h(34.),
        );
    }
    versions = versions.child(controls);
    if s.settings.ready.is_some() {
        versions = versions.child(
            action("install-update", "Install and restart", p, |s, tx| {
                s.install_update(tx)
            })
            .h(34.)
            .fill(p.tint),
        );
    }
    versions
}

fn notebook(s: &State, p: Palette) -> El {
    let database = folio::storage::database_path();
    let database = if s.settings.encrypted {
        folio::security::encrypted_path(&database)
    } else {
        database
    };
    section(
        "settings-data",
        "Your notebook",
        "Your work lives here. Keep a safe copy whenever you need one.",
        p,
    )
    .child(setting_row(
        "Notebook storage",
        "Your notebook is saved locally on this device.",
        layout::badge("On this device", false, p).help(database.display().to_string()),
        p,
    ))
    .child(setting_row(
        "Notebook backups",
        "Keep a copy of your notes, page history and module data, or restore a previous backup.",
        row()
            .gap(8.)
            .flex_wrap()
            .child(
                action("settings-backup", "Back up notebook", p, |s, tx| {
                    s.backup_dialog(tx)
                })
                .h(34.)
                .fill(p.tint)
                .disabled(s.demo || s.settings.locked),
            )
            .child(
                action("settings-restore", "Restore a backup…", p, |s, tx| {
                    s.restore_dialog(tx)
                })
                .h(34.)
                .border(1., p.line)
                .disabled(s.demo || s.settings.locked),
            ),
        p,
    ))
}

fn security(s: &State, p: Palette) -> El {
    let mut security = section(
        "settings-security",
        "Privacy & security",
        "Choose how you protect your notebook.",
        p,
    );
    security = if s.settings.encrypted {
        security
            .child(layout::badge(
                if s.settings.locked {
                    "Encrypted · Locked"
                } else {
                    "Encryption enabled"
                },
                true,
                p,
            ))
            .child(paragraph(
                "Your master password protects the notebook. Lock it when you step away.",
                p,
            ))
            .child(
                action("lock-notebook", "Lock notebook", p, |s, _| {
                    s.lock_notebook()
                })
                .h(34.)
                .fill(p.tint)
                .self_start()
                .disabled(s.settings.locked || s.settings.busy),
            )
    } else {
        security.child(layout::badge("Encryption is off", false, p))
            .child(paragraph("Add a master password to protect your notebook and enable PIN protection for individual notes.", p))
            .child(action("enable-encryption", "Enable encryption…", p, |s, tx| {
                s.security_form(SecurityForm::EnableVault); tx.focus("security-secret", false);
            }).h(34.).self_start().fill(p.tint).disabled(s.demo || s.settings.busy))
    };
    security = security.child(paragraph(
        "Experimental encryption. Attachments are not encrypted.",
        p,
    ));
    if let Some(error) = &s.settings.error {
        security = security.child(label(error, 12., p.ink).role("alert").w_full().wrap());
    }
    if s.settings.busy {
        security = security.child(paragraph("Please wait…", p));
    }
    security
}

fn appearance(s: &State, p: Palette, width: f32) -> El {
    let mut themes = row().w_full().max_w(340.).gap(10.);
    for (key, title, dark, icon) in [
        ("light", "Light", false, &g::SUN),
        ("dark", "Dark", true, &g::MOON),
    ] {
        themes = themes.child(
            super::icons::action(
                format!("settings-theme-{key}"),
                title,
                icon,
                p,
                move |s, _| {
                    if s.dark != dark {
                        s.switch_theme();
                    }
                },
            )
            .grow(1.)
            .min_w(0.)
            .h(42.)
            .radius(8.)
            .checked(s.dark == dark)
            .fill(if s.dark == dark { p.tint } else { p.bg })
            .border(1., if s.dark == dark { p.accent } else { p.line }),
        );
    }
    let columns = if width >= 1120. {
        3
    } else if width >= 640. {
        2
    } else {
        1
    };
    let mut palettes = col()
        .key("settings-palette-gallery")
        .w_full()
        .min_w(0.)
        .equal_columns(if s.dark { columns } else { columns.min(2) })
        .gap(16.);
    if s.dark {
        for palette in DarkPalette::ALL {
            palettes = palettes.child(palette_choice(s, palette, p));
        }
    } else {
        for choice in LightPalette::ALL {
            palettes = palettes.child(palette_tile(
                PalettePreview {
                    key: format!("settings-light-palette-{}", choice.key()),
                    name: choice.name(),
                    description: choice.description(),
                    colors: super::light_palette(choice),
                    selected: s.workspace.preferences.light_palette == choice,
                },
                p,
                move |s, _| s.choose_light_palette(choice),
            ));
        }
    }
    section(
        "settings-appearance",
        "Appearance",
        "Make room for your own colors.",
        p,
    )
    .child(
        row()
            .w_full()
            .gap(20.)
            .flex_wrap()
            .center_y()
            .child(
                col()
                    .grow(1.)
                    .min_w(220.)
                    .gap(6.)
                    .child(label("Display mode", 14., p.ink).font(TextStyle::new(14., 550.)))
                    .child(paragraph("Choose a light canvas or a darker workspace.", p)),
            )
            .child(themes),
    )
    .child(
        col()
            .w_full()
            .pt(22.)
            .gap(8.)
            .border_sides([1., 0., 0., 0.], p.line)
            .child(
                label(if s.dark { "Dark palettes" } else { "Light palettes" }, 17., p.ink)
                    .font(TextStyle::new(17., 600.))
                    .role("heading")
                    .level(2),
            )
            .child(paragraph(
                "Choose a preview to apply it. Your choice is saved automatically.",
                p,
            )),
    )
    .child(palettes)
    .child(paragraph(
        if s.dark {
            "Aurum carries Folio’s warm signature. Sage · Original restores the previous dark colors."
        } else {
            "Vanille brings a softer yellow glow. Original restores the previous light colors."
        }, p,
    ))
}

struct PalettePreview {
    key: String,
    name: &'static str,
    description: &'static str,
    colors: Palette,
    selected: bool,
}

fn palette_choice(s: &State, palette: DarkPalette, p: Palette) -> El {
    let description = match palette {
        DarkPalette::Aurum => "Signature · Burnt orange & ivory",
        DarkPalette::Sage => "Original · Soft greens & charcoal",
        _ => palette.description(),
    };
    palette_tile(
        PalettePreview {
            key: format!("settings-palette-{}", palette.key()),
            name: palette.name(),
            description,
            colors: super::palette(true, palette),
            selected: s.workspace.preferences.dark_palette == palette,
        },
        p,
        move |s, _| s.choose_dark_palette(palette),
    )
}

fn palette_tile(
    preview: PalettePreview,
    p: Palette,
    apply: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    let PalettePreview {
        key,
        name,
        description,
        colors,
        selected,
    } = preview;
    let mut sample_sidebar = col()
        .w(42.)
        .h_full()
        .padding(8.)
        .gap(6.)
        .fill(colors.sidebar);
    for (width, color) in [
        (20., colors.accent),
        (26., colors.tint),
        (20., colors.line),
        (24., colors.line),
    ] {
        sample_sidebar = sample_sidebar.child(el().size(width, 4.).radius(2.).fill(color));
    }
    let preview = row()
        .w_full()
        .h(114.)
        .min_w(0.)
        .fill(colors.bg)
        .a11y_hidden()
        .child(sample_sidebar)
        .child(
            col()
                .grow(1.)
                .min_w(0.)
                .padding(14.)
                .gap(9.)
                .child(
                    label("A little room for ideas", 12., colors.ink)
                        .font(TextStyle::new(12., 600.))
                        .truncate(),
                )
                .child(
                    label("Write something worth keeping.", 10., colors.muted)
                        .w_full()
                        .wrap()
                        .max_lines(2),
                )
                .child(
                    row()
                        .w_full()
                        .gap(6.)
                        .center_y()
                        .child(el().size(10., 10.).radius(3.).fill(colors.accent))
                        .child(el().h(4.).grow(1.).radius(2.).fill(colors.line)),
                )
                .child(el().h(4.).w_full().radius(2.).fill(colors.tint)),
        );
    let mut title = row().w_full().min_w(0.).gap(8.).center_y().child(
        label(name, 13., p.ink)
            .font(TextStyle::new(13., 550.))
            .grow(1.)
            .min_w(0.)
            .truncate(),
    );
    if selected {
        title = title.child(rune::icon(&g::CHECK, 15.).color(p.accent).a11y_hidden());
    }
    super::button(key, name, p, apply)
        .w_full()
        .min_w(0.)
        .h(198.)
        .padding(2.)
        .radius(10.)
        .checked(selected)
        .help(description)
        .border(
            if selected { 2. } else { 1. },
            if selected { p.accent } else { p.line },
        )
        .fill(p.bg)
        .child(
            col()
                .w_full()
                .h_full()
                .min_w(0.)
                .clip_rounded(8.)
                .child(preview)
                .child(
                    col()
                        .w_full()
                        .min_w(0.)
                        .padding(14.)
                        .gap(7.)
                        .child(title)
                        .child(
                            label(description, 11., p.muted)
                                .w_full()
                                .wrap()
                                .max_lines(2),
                        ),
                ),
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
