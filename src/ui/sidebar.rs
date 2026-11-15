use super::{Palette, label};
use crate::state::{Filter, Panel, State};
use folio::icons::{self as glyph, controls as g};
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
        "Nouvelle page",
        &g::PLUS,
        p,
        |s, tx| {
            s.create(false);
            tx.focus("page-title", false);
        },
    ))
    .child(icon_item(
        "new-collection",
        "Nouvelle base",
        &glyph::DATABASE,
        p,
        |s, tx| {
            s.create_collection();
            tx.focus("page-title", false);
        },
    ))
    .child(icon_item(
        "templates",
        "Modèles",
        &g::TEMPLATES,
        p,
        |s, _| s.panel = Panel::Templates,
    ))
    .child(el().h(1.0).fill(p.line))
    .child(icon_item(
        "workspace-menu",
        "Votre carnet",
        &glyph::FOLDER,
        p,
        |s, _| s.panel = Panel::Workspace,
    ))
}

fn nav(
    key: &str,
    title: &str,
    glyph: &'static rune::Icon,
    filter: Filter,
    s: &State,
    p: Palette,
) -> El {
    let selected = s.filter == filter;
    row()
        .key(key)
        .role("button")
        .label(title)
        .focusable(true)
        .h(34.0)
        .px(10.0)
        .gap(10.0)
        .center_y()
        .radius(6.0)
        .fill(if selected { p.tint } else { p.sidebar })
        .hover(p.ink.alpha(0.04))
        .on_press(move |s: &mut State| {
            s.filter = filter;
            s.panel = Panel::Closed;
        })
        .child(
            rune::icon(glyph, 16.0)
                .color(if selected { p.accent } else { p.faint })
                .a11y_hidden(),
        )
        .child(label(
            title,
            12.0,
            if selected { p.accent } else { p.muted },
        ))
}

pub(super) fn view(s: &State, p: Palette) -> El {
    let rows = s.sidebar_pages();
    let mut pages = col().w_full().gap(2.0);
    for (page, depth) in &rows {
        let id = page.id.clone();
        let collapse_id = id.clone();
        let drag_id = id.clone();
        let zone_id = id.clone();
        let selected = s.selected.as_ref() == Some(&id);
        let has_children = s
            .pages
            .iter()
            .any(|child| child.parent_id.as_deref() == Some(&id) && !child.archived);
        let toggle = if has_children {
            super::icons::control(
                format!("fold-{id}"),
                if s.collapsed.contains(&id) {
                    "Déplier la page"
                } else {
                    "Replier la page"
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
                .fill(
                    if selected || super::entity_target(s, &format!("tree-inside-{zone_id}")) {
                        p.tint
                    } else {
                        p.sidebar
                    },
                )
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
                        .hover(p.ink.alpha(0.045))
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
                        .child(super::icons::page(
                            &page.emoji,
                            16.0,
                            if selected { p.accent } else { p.muted },
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
            .fill(if super::entity_target(s, "tree-root") {
                p.tint
            } else {
                rune::Color::TRANSPARENT
            })
            .help("Déposer ici pour déplacer la page à la racine"),
    );
    if rows.is_empty() {
        pages = pages.child(label("Aucune page ici.", 12.0, p.faint).padding(12.0));
    }
    col()
        .key("sidebar")
        .role("navigation")
        .label("Navigation du carnet")
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
                        .child(label("PERSONNEL", 9.0, p.faint)),
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
                                .label("Rechercher dans les pages")
                                .single_line()
                                .grow(1.0)
                                .min_w(0.0)
                                .color(p.ink)
                                .caret(p.accent, p.tint)
                                .placeholder("Rechercher…", p.faint)
                                .on_edit(|s: &mut State| &mut s.search),
                        )
                        .child(label("⌘K", 10.0, p.faint)),
                )
                .child(
                    col()
                        .gap(2.0)
                        .child(sidebar_action(
                            "open-dashboard",
                            "Vue d’ensemble",
                            &glyph::DASHBOARD,
                            p,
                            |s, tx| {
                                s.open_dashboard();
                                tx.scroll_to_y("document-scroll", 0.0);
                            },
                        ))
                        .child(nav(
                            "all-pages",
                            "Toutes les pages",
                            &g::TEXT,
                            Filter::All,
                            s,
                            p,
                        ))
                        .child(nav(
                            "favorites",
                            "Favoris",
                            &g::STAR,
                            Filter::Favorites,
                            s,
                            p,
                        ))
                        .child(nav(
                            "archive-filter",
                            "Archives",
                            &glyph::ARCHIVE,
                            Filter::Archive,
                            s,
                            p,
                        )),
                ),
        )
        .child(
            row()
                .px(22.0)
                .py(12.0)
                .center_y()
                .child(label("VOTRE CARNET", 9.0, p.faint))
                .child(el().grow(1.0))
                .child(super::icons::control(
                    "new-page",
                    "Nouvelle page",
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
                    sidebar_action("sidebar-more", "Plus", &g::MORE, p, |s, _| {
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
                        .child(label("Votre carnet, sur cet appareil", 10.0, p.faint)),
                ),
        )
}
