use super::{Palette, action, label, management as layout};
use crate::{
    module_library::{Confirmation, Job, Tab},
    state::{Panel, State},
};
use folio::{
    icons,
    modules::{self, Action, Candidate, Catalog, Record},
};
use rune::{Align, El, ScrollSpec, TextStyle, col, el, row, text_input};
fn hint(message: impl Into<String>, p: Palette) -> El {
    label(message, 12., p.muted).w_full().wrap()
}
fn card(p: Palette) -> El {
    layout::surface(p)
}
fn empty(title: &str, message: &str, p: Palette) -> El {
    card(p)
        .key("library-empty")
        .min_h(300.)
        .center()
        .gap(14.)
        .child(
            col()
                .size(64., 64.)
                .radius(20.)
                .fill(p.tint)
                .center()
                .child(
                    rune::icon(&icons::PUZZLE, 32.)
                        .color(p.accent)
                        .a11y_hidden(),
                ),
        )
        .child(
            label(title, 20., p.ink)
                .font(TextStyle::new(20., 600.))
                .w_full()
                .wrap()
                .align(Align::Center),
        )
        .child(hint(message, p).max_w(380.).align(Align::Center))
}
fn context(s: &State, blocked: bool, p: Palette) -> El {
    let records = &s.library.registry.records;
    let installed = records.values().filter(|r| r.installed).count();
    let enabled = records.values().filter(|r| r.active()).count();
    let retained = records.values().filter(|r| !r.installed).count();
    let mut summary = card(p).key("library-summary").child(layout::heading(
        "Your library",
        "Your modules at a glance.",
        &icons::LIBRARY,
        p,
    ));
    for (title, count) in [
        ("Installed", installed),
        ("Enabled", enabled),
        ("Retained", retained),
    ] {
        summary = summary.child(
            row()
                .w_full()
                .center_y()
                .child(label(title, 12., p.muted).grow(1.))
                .child(layout::badge(
                    count.to_string(),
                    title == "Enabled" && count > 0,
                    p,
                )),
        );
    }
    let mut sources = card(p)
        .key("library-sources")
        .child(layout::heading(
            "Module sources",
            "Choose where your modules come from.",
            &icons::FOLDER,
            p,
        ))
        .child(hint(
            "Add a local catalog to discover modules, or import a module directly.",
            p,
        ))
        .child(
            action(
                "library-catalogs",
                if s.library.sources {
                    "Hide catalogs"
                } else {
                    "Manage catalogs"
                },
                p,
                |s, _| s.library.sources = !s.library.sources,
            )
            .h(34.)
            .border(1., p.line)
            .justify_start(),
        );
    if s.library.sources {
        sources = sources.child(hint("Folio Library · built in", p)).child(
            action("library-add-catalog", "Add catalog…", p, |s, tx| {
                s.library_import(true, tx)
            })
            .h(34.)
            .fill(p.tint)
            .disabled(blocked),
        );
        for catalog in s.library.registry.catalogs.values() {
            let id = catalog.id.clone();
            sources = sources.child(
                col()
                    .gap(6.)
                    .w_full()
                    .min_w(0.)
                    .child(hint(&catalog.name, p))
                    .child(
                        action(
                            format!("library-remove-catalog-{}", catalog.id),
                            "Remove",
                            p,
                            move |s, _| {
                                s.library_job(Job::Apply(Action::RemoveCatalog(id.clone())))
                            },
                        )
                        .disabled(blocked),
                    ),
            );
        }
    }
    col()
        .key("library-context")
        .w(260.)
        .grow(0.8)
        .min_w(240.)
        .gap(20.)
        .child(summary)
        .child(sources)
        .child(
            row()
                .gap(8.)
                .center_y()
                .px(4.)
                .child(
                    rune::icon(&icons::controls::LOCK, 13.)
                        .color(p.faint)
                        .a11y_hidden(),
                )
                .child(
                    label("Managed in your notebook", 11., p.faint)
                        .grow(1.)
                        .min_w(0.)
                        .wrap(),
                ),
        )
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
                .wrap()
                .max_lines(3)
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
    let header = layout::header(
        "Library",
        "Discover and manage your notebook's modules.",
        &icons::LIBRARY,
        row()
            .gap(8.)
            .center_y()
            .child(
                action("library-add", "Add module…", p, |s, tx| {
                    s.library_import(false, tx)
                })
                .h(34.)
                .fill(p.tint)
                .disabled(blocked),
            )
            .child(
                action("library-close", "Back to notes", p, |s, _| {
                    s.panel = Panel::Closed
                })
                .h(34.)
                .border(1., p.line)
                .radius(7.),
            ),
        p,
    );
    let mut body = col()
        .key("library-modules")
        .w(520.)
        .grow(2.)
        .min_w(420.)
        .gap(16.);
    let mut tabs = row()
        .gap(4.)
        .flex_wrap()
        .w_full()
        .pb(12.)
        .border_sides([0., 0., 1., 0.], p.line);
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
            .h(36.)
            .px(12.)
            .radius(7.)
            .checked(s.library.tab == tab)
            .fill(if s.library.tab == tab { p.tint } else { p.bg }),
        );
    }
    body = body.child(tabs).child(
        row()
            .w_full()
            .min_w(0.)
            .px(12.)
            .gap(9.)
            .center_y()
            .radius(9.)
            .border(1., p.line)
            .fill(p.bg)
            .child(
                rune::icon(&icons::controls::SEARCH, 17.)
                    .color(p.faint)
                    .a11y_hidden(),
            )
            .child(
                text_input(&s.library.search, TextStyle::new(13., 400.))
                    .key("library-search")
                    .label("Search modules")
                    .placeholder("Search your modules…", p.faint)
                    .single_line()
                    .grow(1.)
                    .min_w(0.)
                    .h(42.)
                    .py(10.)
                    .color(p.ink)
                    .caret(p.accent, p.tint)
                    .on_edit(|s: &mut State| &mut s.library.search),
            ),
    );
    if let Some(error) = &s.library.error {
        body = body.child(
            label(error, 12., p.ink)
                .role("alert")
                .w_full()
                .wrap()
                .padding(12.)
                .radius(8.)
                .fill(p.tint),
        );
    }
    let query = s.library.search.text.trim().to_lowercase();
    let limit = if query == s.library.visible_query {
        s.library.visible_limit.max(40)
    } else {
        40
    };
    let mut total = 0;
    let mut results = row().w_full().gap(16.).flex_wrap().items_start();
    if s.library.loaded {
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
                            if total < limit {
                                results = results.child(
                                    available(raw, &catalog.id, s, p)
                                        .w(310.)
                                        .grow(1.)
                                        .min_w(280.),
                                );
                            }
                            total += 1;
                        }
                    }
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
                    if total < limit {
                        results =
                            results.child(installed(id, record, s, p).w(310.).grow(1.).min_w(280.));
                    }
                    total += 1;
                }
            }
        }
    }
    body = body.child(if !s.library.loaded {
        empty("Opening your library", "Your notebook's modules will appear here.", p)
    } else if total == 0 {
        let (title, description) = if !query.is_empty() {
            ("No matching modules", "Try another name, or clear your search to see the full collection.")
        } else { match s.library.tab {
            Tab::Discover => ("Your library is ready", "No modules published yet. Import a local module or catalog to get started."),
            Tab::Installed => ("No modules installed", "No installed modules. Discover additions or import a module when you're ready."),
            Tab::Retained => ("No retained data", "No retained module data. Data kept after uninstalling a module will appear here."),
        }};
        empty(title, description, p)
    } else { results });
    if s.library.loaded && total > limit {
        body = body.child(
            action(
                "library-show-more",
                &format!("Show more · {limit} of {total}"),
                p,
                move |s, _| {
                    s.library.visible_limit = limit + 40;
                    s.library.visible_query = query.clone();
                },
            )
            .h(36.)
            .fill(p.tint),
        );
    }
    layout::page(
        "library-page",
        header,
        row()
            .gap(20.)
            .flex_wrap()
            .items_start()
            .child(body)
            .child(context(s, blocked, p)),
        p,
    )
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
