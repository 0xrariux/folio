use super::{Palette, label};
use crate::state::{Filter, Panel, State};
use folio::icons::{self as glyph, controls as g};
use folio::workspace::SidebarShortcut;
use rune::{Align, El, ScrollSpec, Side, TextStyle, col, el, row, text, text_input};

fn sidebar_action(
    key: &str,
    title: &str,
    glyph: &'static rune::Icon,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    super::button(key, title, p, f)
        .w_full()
        .h(34.0)
        .px(10.0)
        .gap(10.0)
        .justify_start()
        .child(rune::icon(glyph, 16.0).color(p.muted).a11y_hidden())
        .child(label(title, 12.0, p.muted).min_w(0.0).truncate())
}

pub(super) fn more_menu(s: &State, p: Palette) -> El {
    use super::panels::icon_item;
    super::panels::surface(
        "sidebar-more-menu",
        "sidebar-more",
        s.workspace.preferences.sidebar_width as f32 - 32.0,
        p,
    )
    .anchored_fit("sidebar-more", Side::Top, Align::Start, 5.0)
    .child(icon_item(
        "new-page-bottom",
        "New page",
        &g::PLUS,
        p,
        |s, tx| {
            s.create(false);
            tx.focus("page-title", false);
        },
    ))
    .child(icon_item(
        "new-collection",
        "New database",
        &glyph::DATABASE,
        p,
        |s, tx| {
            s.create_collection();
            tx.focus("page-title", false);
        },
    ))
    .child(icon_item(
        "templates",
        "Templates",
        &g::TEMPLATES,
        p,
        |s, _| s.panel = Panel::Templates,
    ))
    .child(el().h(1.0).fill(p.line))
    .child(icon_item(
        "workspace-menu",
        "Your notebook",
        &glyph::FOLDER,
        p,
        |s, _| s.panel = Panel::Workspace,
    ))
}

fn nav(shortcut: SidebarShortcut, s: &State, p: Palette) -> El {
    use crate::entity_drag::Kind;
    let key = match shortcut {
        SidebarShortcut::Overview => "open-dashboard",
        SidebarShortcut::AllPages => "all-pages",
        SidebarShortcut::Favorites => "favorites",
        SidebarShortcut::Archives => "archive-filter",
    };
    let selected = matches!(
        (shortcut, s.filter),
        (SidebarShortcut::AllPages, Filter::All)
            | (SidebarShortcut::Favorites, Filter::Favorites)
            | (SidebarShortcut::Archives, Filter::Archive)
    );
    let source = format!("shortcut:{}", shortcut.key());
    let click_id = source.clone();
    let dragging = s
        .workspace
        .entity_drag
        .as_ref()
        .is_some_and(|d| d.kind == Kind::Shortcut && d.active);
    let moving = dragging
        && s.workspace
            .entity_drag
            .as_ref()
            .is_some_and(|d| d.id == source);
    let before_key = format!("shortcut-before-{}", shortcut.key());
    let after_key = format!("shortcut-after-{}", shortcut.key());
    let before = dragging && super::entity_target(s, &before_key);
    let after = dragging && super::entity_target(s, &after_key);
    row()
        .key(key)
        .role("button")
        .label(shortcut.name())
        .focusable(true)
        .h(34.0)
        .w_full()
        .px(10.0)
        .gap(10.0)
        .center_y()
        .radius(6.0)
        .fill(if selected || moving {
            p.tint
        } else {
            p.sidebar
        })
        .opacity(if moving { 0.55 } else { 1.0 })
        .hover(p.ink.alpha(0.04))
        .cursor(if moving {
            rune::Cursor::Grabbing
        } else {
            rune::Cursor::Grab
        })
        .on_drag(move |s: &mut State, drag, tx| s.drag_entity(&source, Kind::Shortcut, drag, tx))
        .on_press_tx(move |s: &mut State, tx| {
            if s.entity_click(&click_id) {
                return;
            }
            match shortcut {
                SidebarShortcut::Overview => {
                    s.open_dashboard();
                    tx.scroll_to_y("document-scroll", 0.0);
                }
                SidebarShortcut::AllPages => s.filter = Filter::All,
                SidebarShortcut::Favorites => s.filter = Filter::Favorites,
                SidebarShortcut::Archives => s.filter = Filter::Archive,
            }
            s.panel = Panel::Closed;
        })
        .child(
            rune::icon(shortcut_icon(shortcut), 16.0)
                .color(if selected { p.accent } else { p.faint })
                .a11y_hidden(),
        )
        .child(label(
            shortcut.name(),
            12.0,
            if selected { p.accent } else { p.muted },
        ))
        .child(if before {
            el().key("shortcut-drop-indicator")
                .absolute(0.0, 0.0)
                .w_full()
                .h(2.0)
                .radius(1.0)
                .fill(p.accent)
                .a11y_hidden()
        } else if after {
            el().key("shortcut-drop-indicator")
                .absolute(0.0, 32.0)
                .w_full()
                .h(2.0)
                .radius(1.0)
                .fill(p.accent)
                .a11y_hidden()
        } else {
            el()
        })
        .child(
            col()
                .absolute(0.0, 0.0)
                .w_full()
                .h_full()
                .a11y_hidden()
                .child(el().key(before_key).w_full().h(17.0))
                .child(el().key(after_key).w_full().h(17.0)),
        )
}

fn shortcut_icon(shortcut: SidebarShortcut) -> &'static rune::Icon {
    match shortcut {
        SidebarShortcut::Overview => &glyph::DASHBOARD,
        SidebarShortcut::AllPages => &g::TEXT,
        SidebarShortcut::Favorites => &g::STAR,
        SidebarShortcut::Archives => &glyph::ARCHIVE,
    }
}

fn navigation(s: &State, p: Palette) -> El {
    let mut shortcuts = col().w_full().gap(2.0).child(
        row()
            .h(22.0)
            .px(10.0)
            .center_y()
            .child(label("SHORTCUTS", 9.0, p.faint).grow(1.0))
            .child(
                super::icons::control(
                    "customize-navigation",
                    "Customize shortcuts",
                    &g::MORE,
                    p,
                    |s, _| s.panel = Panel::SidebarNavigation,
                )
                .size(24.0, 22.0)
                .px(4.0)
                .disabled(
                    s.workspace.busy
                        || s.settings.busy
                        || s.settings.locked
                        || (!s.demo && !s.workspace.preferences_loaded),
                ),
            ),
    );
    for &shortcut in &s.workspace.preferences.sidebar_shortcuts {
        shortcuts = shortcuts.child(nav(shortcut, s, p));
    }
    shortcuts
}

pub(super) fn navigation_menu(s: &State, p: Palette) -> El {
    let visible = &s.workspace.preferences.sidebar_shortcuts;
    let mut body = col().gap(4.0).child(
        row()
            .center_y()
            .px(8.0)
            .child(label("Shortcuts", 12.0, p.muted).grow(1.0))
            .child(super::icons::control(
                "reset-shortcuts",
                "Reset shortcuts",
                &g::UNDO,
                p,
                |s, _| s.reset_sidebar_shortcuts(),
            )),
    );
    for (index, &shortcut) in visible.iter().enumerate() {
        body = body.child(
            row()
                .h(34.0)
                .gap(6.0)
                .px(8.0)
                .center_y()
                .child(
                    rune::icon(shortcut_icon(shortcut), 16.0)
                        .color(p.muted)
                        .a11y_hidden(),
                )
                .child(
                    label(shortcut.name(), 12.0, p.ink)
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                )
                .child(
                    super::icons::control(
                        format!("shortcut-up-{}", shortcut.key()),
                        &format!("Move {} up", shortcut.name()),
                        &g::ARROW_UP,
                        p,
                        move |s, _| s.move_sidebar_shortcut(shortcut, true),
                    )
                    .size(24.0, 26.0)
                    .px(4.0)
                    .disabled(index == 0),
                )
                .child(
                    super::icons::control(
                        format!("shortcut-down-{}", shortcut.key()),
                        &format!("Move {} down", shortcut.name()),
                        &g::ARROW_DOWN,
                        p,
                        move |s, _| s.move_sidebar_shortcut(shortcut, false),
                    )
                    .size(24.0, 26.0)
                    .px(4.0)
                    .disabled(index + 1 == visible.len()),
                )
                .child(
                    super::icons::control(
                        format!("shortcut-hide-{}", shortcut.key()),
                        &format!("Hide {}", shortcut.name()),
                        &g::CLOSE,
                        p,
                        move |s, _| s.show_sidebar_shortcut(shortcut, false),
                    )
                    .size(24.0, 26.0)
                    .px(4.0),
                ),
        );
    }
    let hidden: Vec<_> = SidebarShortcut::ALL
        .into_iter()
        .filter(|shortcut| !visible.contains(shortcut))
        .collect();
    if !hidden.is_empty() {
        body = body.child(label("Hidden", 9.0, p.faint).padding(8.0));
        for shortcut in hidden {
            body = body.child(
                super::panels::icon_item(
                    format!("shortcut-add-{}", shortcut.key()),
                    shortcut.name(),
                    &g::PLUS,
                    p,
                    move |s, _| s.show_sidebar_shortcut(shortcut, true),
                )
                .label(format!("Show {}", shortcut.name())),
            );
        }
    }
    super::panels::surface("sidebar-navigation-menu", "customize-navigation", 300.0, p).child(body)
}

pub(super) fn view(s: &State, p: Palette) -> El {
    let rows = s.sidebar_pages();
    let dragged_page = s
        .workspace
        .entity_drag
        .as_ref()
        .filter(|drag| drag.active && drag.kind == crate::entity_drag::Kind::Page)
        .and_then(|drag| {
            s.pages
                .iter()
                .find(|page| page.id == drag.id && !page.archived)
        });
    let mut pages = col().w_full().gap(2.0);
    for (page, depth) in &rows {
        let id = page.id.clone();
        let collapse_id = id.clone();
        let drag_id = id.clone();
        let zone_id = id.clone();
        let menu_id = id.clone();
        let selected = s.selected.as_ref() == Some(&id);
        let moving = dragged_page.is_some_and(|source| source.id == page.id);
        let allowed = !page.archived
            && dragged_page.is_some_and(|source| {
                source.id != page.id && !s.ancestors(&page.id).iter().any(|id| id == &source.id)
            });
        let before = allowed && super::entity_target(s, &format!("tree-before-{zone_id}"));
        let after = allowed && super::entity_target(s, &format!("tree-after-{zone_id}"));
        let inside = allowed && super::entity_target(s, &format!("tree-inside-{zone_id}"));
        let has_children = s
            .pages
            .iter()
            .any(|child| child.parent_id.as_deref() == Some(&id) && !child.archived);
        let toggle = if has_children {
            super::icons::control(
                format!("fold-{id}"),
                if s.collapsed.contains(&id) {
                    "Expand page"
                } else {
                    "Collapse page"
                },
                if s.collapsed.contains(&id) {
                    &g::CHEVRON_RIGHT
                } else {
                    &g::CHEVRON_DOWN
                },
                p,
                move |s, _| {
                    if !s.collapsed.remove(&collapse_id) {
                        s.collapsed.insert(collapse_id.clone());
                    }
                },
            )
            .w(17.0)
            .px(0.0)
        } else {
            el().w(17.0)
        };
        pages = pages.child(
            row()
                .key(format!("page-row-{id}"))
                .h(34.0)
                .w_full()
                .px(4.0 + (*depth).min(6) as f32 * 12.0)
                .center_y()
                .radius(6.0)
                .hover(p.ink.alpha(0.045))
                .hover_group()
                .fill(if selected || moving || before || after || inside {
                    p.tint
                } else {
                    p.sidebar
                })
                .opacity(if moving { 0.65 } else { 1.0 })
                .border(if inside { 1.0 } else { 0.0 }, p.accent)
                .child(toggle)
                .child(
                    row()
                        .key(format!("page-{id}"))
                        .role("button")
                        .label(page.display_title())
                        .focusable(true)
                        .h_full()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(7.0)
                        .center_y()
                        .cursor(if moving {
                            rune::Cursor::Grabbing
                        } else {
                            rune::Cursor::Grab
                        })
                        .on_drag(move |s: &mut State, d, tx| {
                            s.drag_entity(&drag_id, crate::entity_drag::Kind::Page, d, tx)
                        })
                        .on_press_tx(move |s: &mut State, tx| {
                            if s.entity_click(&id) {
                                return;
                            }
                            s.select(&id);
                            tx.scroll_to_y("document-scroll", 0.0);
                            tx.instant();
                        })
                        .child(super::icons::page_icon(
                            page,
                            16.0,
                            if selected { p.accent } else { p.muted },
                            p,
                        ))
                        .child(
                            label(
                                page.display_title(),
                                12.0,
                                if selected { p.ink } else { p.muted },
                            )
                            .grow(1.0)
                            .min_w(0.0)
                            .truncate(),
                        )
                        .child(if page.favorite {
                            rune::icon(&g::STAR, 12.0).color(p.faint).a11y_hidden()
                        } else {
                            el()
                        }),
                )
                .child(
                    super::icons::control(
                        format!("page-actions-{menu_id}"),
                        "Page actions",
                        &g::MORE,
                        p,
                        move |s, _| {
                            s.select(&menu_id);
                            s.panel = Panel::SidebarPage(menu_id.clone());
                        },
                    )
                    .size(24.0, 26.0)
                    .px(4.0)
                    .group_opacity(0.35, 1.0)
                    .disabled(s.workspace.busy || s.settings.busy),
                )
                .child(if before || after {
                    el().key("page-drop-indicator")
                        .absolute(0.0, if before { 0.0 } else { 32.0 })
                        .w_full()
                        .h(2.0)
                        .radius(1.0)
                        .fill(p.accent)
                        .a11y_hidden()
                } else {
                    el()
                })
                .child(
                    col()
                        .absolute(0.0, 0.0)
                        .w_full()
                        .h_full()
                        .child(el().key(format!("tree-before-{zone_id}")).w_full().h(8.0))
                        .child(
                            el().key(format!("tree-inside-{zone_id}"))
                                .w_full()
                                .grow(1.0),
                        )
                        .child(el().key(format!("tree-after-{zone_id}")).w_full().h(8.0)),
                ),
        );
    }
    pages = pages.child(
        el().key("tree-root")
            .w_full()
            .h(
                if s.workspace
                    .entity_drag
                    .as_ref()
                    .is_some_and(|d| d.active && d.kind == crate::entity_drag::Kind::Page)
                {
                    28.0
                } else {
                    6.0
                },
            )
            .fill(
                if dragged_page.is_some() && super::entity_target(s, "tree-root") {
                    p.tint
                } else {
                    rune::Color::TRANSPARENT
                },
            )
            .help("Drop here to move the page to the root"),
    );
    if rows.is_empty() {
        pages = pages.child(label("No pages here.", 12.0, p.faint).padding(12.0));
    }
    col()
        .key("sidebar")
        .role("navigation")
        .label("Notebook navigation")
        .w(s.workspace.preferences.sidebar_width as f32)
        .h_full()
        .fill(p.sidebar)
        .border_sides([0.0, 1.0, 0.0, 0.0], p.line)
        .child(el().h(40.0).window_drag())
        .child(
            col()
                .px(16.0)
                .gap(20.0)
                .child(
                    row()
                        .gap(9.0)
                        .center_y()
                        .child(
                            el().size(32.0, 32.0)
                                .radius(9.0)
                                .fill(p.accent)
                                .center()
                                .child(
                                    rune::icon(&folio::icons::FOLIO, 28.0)
                                        .color(p.bg)
                                        .a11y_hidden(),
                                ),
                        )
                        .child(text("folio").font(TextStyle::new(22.0, 600.0)).color(p.ink))
                        .child(el().grow(1.0))
                        .child(label("WORKSPACE", 9.0, p.faint)),
                )
                .child(
                    row()
                        .h(34.0)
                        .w_full()
                        .px(10.0)
                        .gap(8.0)
                        .center_y()
                        .radius(6.0)
                        .border(1.0, p.line)
                        .fill(p.bg)
                        .child(rune::icon(&g::SEARCH, 18.0).color(p.faint).a11y_hidden())
                        .child(
                            text_input(&s.search, TextStyle::new(12.0, 400.0))
                                .key("search")
                                .label("Search pages")
                                .single_line()
                                .grow(1.0)
                                .min_w(0.0)
                                .color(p.ink)
                                .caret(p.accent, p.tint)
                                .placeholder("Search…", p.faint)
                                .on_edit(|s: &mut State| &mut s.search),
                        )
                        .child(label("⌘K", 10.0, p.faint)),
                )
                .child(navigation(s, p)),
        )
        .child(
            row()
                .px(22.0)
                .py(12.0)
                .center_y()
                .child(label("YOUR NOTEBOOK", 9.0, p.faint))
                .child(el().grow(1.0))
                .child(super::icons::control(
                    "new-page",
                    "New page",
                    &g::PLUS,
                    p,
                    |s, tx| {
                        s.create(false);
                        tx.focus("page-title", false);
                    },
                )),
        )
        .child(super::entity_scroll(
            col()
                .key("pages-scroll")
                .grow(1.0)
                .min_h(0.0)
                .px(10.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(pages),
            s,
            "pages-scroll",
        ))
        .child(
            col()
                .padding(16.0)
                .gap(6.0)
                .child(
                    sidebar_action("library", "Library", &glyph::LIBRARY, p, |s, _| {
                        s.open_library()
                    })
                    .fill(
                        if matches!(
                            s.panel,
                            Panel::Library | Panel::ModuleReview | Panel::ModuleConfirm
                        ) {
                            p.tint
                        } else {
                            p.sidebar
                        },
                    ),
                )
                .child(sidebar_action(
                    "settings",
                    if s.settings.remote.as_ref().is_some_and(|r| r.ahead) {
                        "Settings · Update available"
                    } else {
                        "Settings"
                    },
                    &g::SETTINGS,
                    p,
                    |s, _| s.open_settings(),
                ))
                .child(
                    sidebar_action("sidebar-more", "More", &g::MORE, p, |s, _| {
                        s.panel = if matches!(s.panel, Panel::SidebarMore) {
                            Panel::Closed
                        } else {
                            Panel::SidebarMore
                        };
                    })
                    .w_full()
                    .h(34.0)
                    .justify_start()
                    .fill(if matches!(s.panel, Panel::SidebarMore) {
                        p.tint
                    } else {
                        p.sidebar
                    })
                    .child(el().grow(1.0))
                    .child(
                        rune::icon(&g::CHEVRON_DOWN, 14.0)
                            .color(p.faint)
                            .a11y_hidden(),
                    ),
                )
                .child(el().h(1.0).fill(p.line))
                .child(
                    row()
                        .gap(5.0)
                        .py(10.0)
                        .center_y()
                        .child(
                            rune::icon(&g::STATUS_DOT, 12.0)
                                .color(p.faint)
                                .a11y_hidden(),
                        )
                        .child(label("Your notebook, on this device", 10.0, p.faint)),
                ),
        )
}
