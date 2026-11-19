use super::{Palette, action, label};
use crate::{
    module_library::{Confirmation, Job, Tab},
    state::{Panel, State},
};
use folio::{
    icons,
    modules::{self, Action, Candidate, Catalog, Record},
};
use rune::{El, ScrollSpec, TextStyle, col, el, row, text_input};
fn hint(message: impl Into<String>, p: Palette) -> El {
    label(message, 12., p.muted).w_full().wrap()
}
fn card(p: Palette) -> El {
    col()
        .w_full()
        .padding(16.)
        .gap(10.)
        .border(1., p.line)
        .radius(8.)
        .fill(p.bg)
}
fn unavailable(record: &Record) -> Option<String> {
    let m = match record.metadata() {
        Ok(m) => m,
        Err(_) => return Some("Definition not supported. Saved data is kept.".into()),
    };
    if let Err(error) = m.compatibility() {
        return Some(error.to_string());
    }
    if m.data_version != record.data_version {
        return Some("Data migration required".into());
    }
    if m.permissions != record.grants {
        return Some("Permission review required".into());
    }
    None
}
fn available(raw: &serde_json::Value, source: &str, s: &State, p: Palette) -> El {
    let id = raw.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
    let name = raw.get("name").and_then(|v| v.as_str()).unwrap_or(id);
    let version = raw.get("version").and_then(|v| v.as_str()).unwrap_or("?");
    let parsed = Candidate::new(raw.clone());
    let mut issue = parsed.as_ref().err().map(ToString::to_string);
    let previous = s.library.registry.records.get(id);
    let same = previous.is_some_and(|r| r.installed && r.fingerprint == modules::digest(raw));
    if let (Ok(candidate), Some(old)) = (&parsed, previous) {
        if old.data_version != candidate.metadata.data_version {
            issue = Some("Data migration required".into());
        } else if let Ok(old_manifest) = old.metadata() {
            if semver::Version::parse(&candidate.metadata.version).ok()
                < semver::Version::parse(&old_manifest.version).ok()
            {
                issue = Some("An older version".into());
            } else if old_manifest.version == candidate.metadata.version
                && old.fingerprint != candidate.fingerprint
            {
                issue = Some("Version conflict".into());
            }
        } else {
            issue = Some("Retained definition needs review".into());
        }
    }
    let title = if same {
        "Installed"
    } else if previous.is_some_and(|r| r.installed) {
        "Review update"
    } else if previous.is_some() {
        "Reinstall"
    } else {
        "Install"
    };
    let manifest = raw.clone();
    let mut body = card(p).child(
        row()
            .gap(10.)
            .center_y()
            .flex_wrap()
            .child(label(name, 16., p.ink).grow(1.).min_w(0.).wrap())
            .child(label(format!("v{version}"), 11., p.muted))
            .child(
                action(
                    format!("library-install-{source}-{id}"),
                    title,
                    p,
                    move |s, _| s.review_module(manifest.clone()),
                )
                .fill(p.tint)
                .disabled(s.demo || issue.is_some() || same || s.library.pending),
            ),
    );
    if let Some(author) = raw.get("author").and_then(|v| v.as_str()) {
        body = body.child(hint(author, p));
    }
    if let Some(description) = raw
        .get("description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        body = body.child(
            label(description, 12., p.muted)
                .w_full()
                .truncate()
                .help(description),
        );
    }
    if let Some(issue) = issue {
        body = body.child(hint(issue, p));
    }
    body
}
fn installed(id: &str, record: &Record, s: &State, p: Palette) -> El {
    let issue = unavailable(record);
    let version = record
        .manifest
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let status = if !record.installed {
        if record.has_data() {
            "Data retained"
        } else {
            "Not installed"
        }
    } else if issue.is_some() {
        "Unavailable"
    } else if record.active() {
        "Enabled"
    } else {
        "Disabled"
    };
    let mut body = card(p).child(
        row()
            .gap(10.)
            .center_y()
            .flex_wrap()
            .child(label(record.name(), 16., p.ink).grow(1.).min_w(0.).wrap())
            .child(label(format!("v{version} · {status}"), 11., p.muted)),
    );
    let mut buttons = row().gap(10.).flex_wrap();
    if record.installed {
        let toggle_id = id.to_owned();
        let enabled = record.enabled;
        buttons = buttons.child(
            action(
                format!("library-toggle-{id}"),
                if enabled { "Disable" } else { "Enable" },
                p,
                move |s, _| {
                    s.library_job(Job::Apply(if enabled {
                        Action::Disable(toggle_id.clone())
                    } else {
                        Action::Enable(toggle_id.clone())
                    }));
                },
            )
            .fill(p.tint)
            .disabled(s.demo || s.library.pending || (!enabled && issue.is_some())),
        );
        if issue.is_some() && Candidate::new(record.manifest.clone()).is_ok() {
            let manifest = record.manifest.clone();
            buttons = buttons.child(
                action(
                    format!("library-review-{id}"),
                    "Review…",
                    p,
                    move |s, _| s.review_module(manifest.clone()),
                )
                .disabled(s.demo || s.library.pending),
            );
        }
        let remove_id = id.to_owned();
        buttons = buttons.child(
            action(
                format!("library-uninstall-{id}"),
                "Uninstall…",
                p,
                move |s, _| s.confirm_module_action(Confirmation::Uninstall(remove_id.clone())),
            )
            .disabled(s.demo || s.library.pending),
        );
    } else {
        let raw = record.manifest.clone();
        buttons = buttons.child(
            action(
                format!("library-reinstall-{id}"),
                "Reinstall",
                p,
                move |s, _| s.review_module(raw.clone()),
            )
            .fill(p.tint)
            .disabled(
                s.demo || s.library.pending || Candidate::new(record.manifest.clone()).is_err(),
            ),
        );
        let remove_id = id.to_owned();
        buttons = buttons.child(
            action(
                format!("library-purge-{id}"),
                "Delete data…",
                p,
                move |s, _| s.confirm_module_action(Confirmation::Purge(remove_id.clone())),
            )
            .disabled(s.demo || s.library.pending),
        );
    }
    if let Some(issue) = issue {
        body = body.child(hint(issue, p));
    }
    body.child(buttons)
}
pub(super) fn view(s: &State, p: Palette) -> El {
    let blocked = s.demo || !s.library.loaded || s.library.pending || s.settings.busy;
    let mut body = col()
        .w_full()
        .max_w(960.)
        .padding(28.)
        .gap(18.)
        .child(
            row()
                .gap(12.)
                .center_y()
                .flex_wrap()
                .child(
                    rune::icon(&icons::LIBRARY, 26.)
                        .color(p.accent)
                        .a11y_hidden(),
                )
                .child(label("Library", 28., p.ink).role("heading").level(1))
                .child(el().grow(1.))
                .child(
                    action("library-add", "Add module…", p, |s, tx| {
                        s.library_import(false, tx)
                    })
                    .fill(p.tint)
                    .disabled(blocked),
                )
                .child(action("library-close", "Back to notes", p, |s, _| {
                    s.panel = Panel::Closed
                })),
        )
        .child(hint("Add only what you need.", p));
    let mut tabs = row().gap(8.).flex_wrap();
    for (tab, key, name) in [
        (Tab::Discover, "discover", "Discover"),
        (Tab::Installed, "installed", "Installed"),
        (Tab::Retained, "retained", "Retained data"),
    ] {
        tabs = tabs.child(
            action(format!("library-tab-{key}"), name, p, move |s, _| {
                s.library.tab = tab;
                s.library.visible_limit = 0;
            })
            .fill(if s.library.tab == tab { p.tint } else { p.bg }),
        );
    }
    body = body.child(tabs).child(
        text_input(&s.library.search, TextStyle::new(14., 400.))
            .key("library-search")
            .label("Search modules")
            .placeholder("Search modules…", p.faint)
            .single_line()
            .w_full()
            .h(38.)
            .px(10.)
            .py(8.)
            .radius(6.)
            .border(1., p.line)
            .fill(p.sidebar)
            .color(p.ink)
            .on_edit(|s: &mut State| &mut s.library.search),
    );
    if let Some(error) = &s.library.error {
        body = body.child(label(error, 12., p.ink).role("alert").w_full().wrap());
    }
    let query = s.library.search.text.trim().to_lowercase();
    let limit = if query == s.library.visible_query {
        s.library.visible_limit.max(40)
    } else {
        40
    };
    let mut total = 0;
    if !s.library.loaded {
        body = body.child(hint("Loading…", p));
    } else {
        let mut count = 0;
        match s.library.tab {
            Tab::Discover => {
                let builtin = Catalog::built_in();
                for catalog in std::iter::once(&builtin).chain(s.library.registry.catalogs.values())
                {
                    for raw in &catalog.modules {
                        let text = format!(
                            "{} {}",
                            raw.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            raw.get("id").and_then(|v| v.as_str()).unwrap_or("")
                        );
                        if text.to_lowercase().contains(&query) {
                            if count < limit {
                                body = body.child(available(raw, &catalog.id, s, p));
                            }
                            count += 1;
                        }
                    }
                }
                if count == 0 {
                    body = body.child(hint(
                        if query.is_empty() {
                            "No modules published yet."
                        } else {
                            "No matching modules."
                        },
                        p,
                    ));
                }
                body = body.child(
                    action("library-catalogs", "Manage catalogs", p, |s, _| {
                        s.library.sources = !s.library.sources
                    })
                    .justify_start(),
                );
                if s.library.sources {
                    let mut sources = card(p)
                        .child(label("Catalogs", 16., p.ink))
                        .child(hint("Folio Library · built in", p))
                        .child(
                            action("library-add-catalog", "Add catalog…", p, |s, tx| {
                                s.library_import(true, tx)
                            })
                            .fill(p.tint)
                            .disabled(blocked),
                        );
                    for catalog in s.library.registry.catalogs.values() {
                        let id = catalog.id.clone();
                        sources = sources.child(
                            row()
                                .gap(8.)
                                .center_y()
                                .child(hint(&catalog.name, p).grow(1.))
                                .child(
                                    action(
                                        format!("library-remove-catalog-{}", catalog.id),
                                        "Remove",
                                        p,
                                        move |s, _| {
                                            s.library_job(Job::Apply(Action::RemoveCatalog(
                                                id.clone(),
                                            )))
                                        },
                                    )
                                    .disabled(blocked),
                                ),
                        );
                    }
                    body = body.child(sources);
                }
            }
            tab => {
                for (id, record) in &s.library.registry.records {
                    if record.installed != (tab == Tab::Installed)
                        || !format!("{} {id}", record.name())
                            .to_lowercase()
                            .contains(&query)
                    {
                        continue;
                    }
                    if count < limit {
                        body = body.child(installed(id, record, s, p));
                    }
                    count += 1;
                }
                if count == 0 {
                    body = body.child(hint(
                        if !query.is_empty() {
                            "No matching modules."
                        } else if tab == Tab::Installed {
                            "No installed modules."
                        } else {
                            "No retained module data."
                        },
                        p,
                    ));
                }
            }
        }
        total = count;
    }
    if s.library.loaded && total > limit {
        body = body.child(action(
            "library-show-more",
            &format!("Show more · {limit} of {total}"),
            p,
            move |s, _| {
                s.library.visible_limit = limit + 40;
                s.library.visible_query = query.clone();
            },
        ));
    }
    col()
        .key("library-page")
        .grow(1.)
        .min_h(0.)
        .w_full()
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(body)
}
pub(super) fn review(s: &State, p: Palette) -> El {
    let Some(candidate) = &s.library.review else {
        return el();
    };
    let m = &candidate.metadata;
    let mut body = col()
        .gap(12.)
        .w_full()
        .child(label(&m.name, 20., p.ink).w_full().wrap())
        .child(hint(format!("v{} · {}", m.version, m.author), p));
    if !m.description.is_empty() {
        body = body.child(
            col()
                .w_full()
                .max_h(120.)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(hint(&m.description, p)),
        );
    }
    body = body
        .child(hint(
            if m.permissions.is_empty() {
                "No permissions requested."
            } else {
                "Permission: private module storage."
            },
            p,
        ))
        .child(hint("Definition only. No external code will run.", p));
    if s.library.registry.records.contains_key(&m.id) {
        body = body.child(hint(
            "Existing data is kept. The module will be disabled until you enable it.",
            p,
        ));
    }
    if let Some(error) = &s.library.error {
        body = body.child(hint(error, p).role("alert"));
    }
    super::workspace::dialog(
        "Install module",
        p,
        body.child(
            row()
                .gap(10.)
                .flex_wrap()
                .child(
                    action("library-confirm-install", "Install", p, |s, _| {
                        s.install_reviewed_module()
                    })
                    .fill(p.tint)
                    .disabled(s.library.pending),
                )
                .child(
                    action("library-cancel-review", "Cancel", p, |s, _| {
                        s.close_module_dialog()
                    })
                    .disabled(s.library.pending),
                ),
        ),
    )
}
pub(super) fn confirm(s: &State, p: Palette) -> El {
    let purge = matches!(s.library.confirmation, Some(Confirmation::Purge(_)));
    let title = if purge {
        "Delete module data?"
    } else {
        "Uninstall module?"
    };
    let text = if purge {
        "Saved module data will be deleted. A recovery backup is kept."
    } else {
        "Saved data stays in your notebook. You can reinstall later."
    };
    let mut body = col().gap(14.).child(hint(text, p));
    if let Some(error) = &s.library.error {
        body = body.child(hint(error, p).role("alert"));
    }
    super::workspace::dialog(
        title,
        p,
        body.child(
            row()
                .gap(10.)
                .flex_wrap()
                .child(
                    action(
                        "library-confirm-remove",
                        if purge { "Delete data" } else { "Uninstall" },
                        p,
                        |s, _| s.apply_module_confirmation(),
                    )
                    .fill(p.tint)
                    .disabled(s.library.pending),
                )
                .child(
                    action("library-cancel-remove", "Cancel", p, |s, _| {
                        s.close_module_dialog()
                    })
                    .disabled(s.library.pending),
                ),
        ),
    )
}
