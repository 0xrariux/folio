use super::{Palette, action, blocks, collections, label, rgb};
use crate::state::{Panel, Phase, State};
use folio::icons::controls as g;
use rune::{Color, El, ScrollSpec, TextStyle, col, el, row, text_input};

fn cover(style: u8, p: Palette) -> El {
    let (top, bottom, glow) = match style {
        2 => (rgb(183, 209, 218), rgb(101, 143, 162), rgb(227, 238, 231)),
        3 => (rgb(234, 210, 180), rgb(188, 141, 117), rgb(247, 231, 204)),
        _ => (rgb(181, 196, 157), rgb(95, 126, 98), rgb(222, 231, 192)),
    };
    row()
        .key("page-cover")
        .w_full()
        .h(136.0)
        .gradient(top, bottom)
        .padding(26.0)
        .gap(25.0)
        .center_y()
        .child(el().size(78.0, 78.0).radius(39.0).fill(glow.alpha(0.42)))
        .child(el().grow(1.0))
        .child(el().size(130.0, 65.0).radius(40.0).fill(glow.alpha(0.18)))
        .child(
            action("cover-cycle-top", "Changer la couverture", p, |s, _| {
                s.appearance(None, true, false)
            })
            .fill(Color::WHITE.alpha(0.65)),
        )
}

fn header(s: &State, p: Palette) -> El {
    let mut bar = row()
        .h(51.0)
        .px(18.0)
        .gap(3.0)
        .center_y()
        .border_sides([0.0, 0.0, 1.0, 0.0], p.line)
        .child(super::icons::control(
            "back",
            "Page précédente",
            &g::CHEVRON_LEFT,
            p,
            |s, tx| {
                s.travel(false);
                tx.scroll_to_y("document-scroll", 0.0);
            },
        ))
        .child(super::icons::control(
            "forward",
            "Page suivante",
            &g::CHEVRON_RIGHT,
            p,
            |s, tx| {
                s.travel(true);
                tx.scroll_to_y("document-scroll", 0.0);
            },
        ))
        .child(label("Espace personnel", 11.0, p.faint));
    if let Some(page) = s.current() {
        for id in s.ancestors(&page.id) {
            let Some(parent) = s.pages.iter().find(|p| p.id == id) else {
                continue;
            };
            bar = bar.child(label("/", 12.0, p.faint)).child(
                action(
                    format!("breadcrumb-{id}"),
                    parent.display_title(),
                    p,
                    move |s, tx| {
                        s.select(&id);
                        tx.scroll_to_y("document-scroll", 0.0);
                    },
                )
                .max_w(150.0),
            );
        }
        bar = bar
            .child(label("/", 12.0, p.faint))
            .child(
                label(page.display_title(), 12.0, p.muted)
                    .grow(1.0)
                    .min_w(0.0)
                    .truncate(),
            )
            .child(super::icons::control(
                "favorite-page",
                if page.favorite {
                    "Retirer des favoris"
                } else {
                    "Ajouter aux favoris"
                },
                if page.favorite {
                    &g::STAR_FILLED
                } else {
                    &g::STAR
                },
                p,
                |s, _| s.favorite(),
            ))
            .child(super::icons::control(
                "page-menu",
                "Actions de la page",
                &g::MORE,
                p,
                |s, _| s.panel = Panel::Page,
            ));
    }
    bar
}

pub(super) fn view(s: &State, p: Palette, width: f32) -> El {
    let Some(page) = s.current() else {
        let (title, hint) = match s.phase {
            Phase::Loading => (
                "Votre carnet s’ouvre…",
                "Vos pages arrivent dans un instant.",
            ),
            Phase::Failed(_) => (
                "Le carnet n’a pas pu être ouvert.",
                "Le détail de l’erreur est affiché ci-dessous.",
            ),
            Phase::Ready => (
                "Une page blanche, mille possibilités.",
                "Créez une page pour commencer votre carnet.",
            ),
        };
        return col().grow(1.0).min_h(0.0).child(header(s, p)).child(
            col()
                .grow(1.0)
                .center()
                .gap(15.0)
                .child(label(title, 23.0, p.ink))
                .child(label(hint, 13.0, p.muted)),
        );
    };
    let words = page.plain_text.split_whitespace().count();
    let tasks = s
        .blocks
        .iter()
        .filter(|b| b.data.kind == folio::document::Kind::Todo)
        .count();
    let done = s
        .blocks
        .iter()
        .filter(|b| b.data.kind == folio::document::Kind::Todo && b.data.checked)
        .count();
    let mut content = col().w_full().gap(12.0);
    if !page.wide {
        content = content.max_w(770.0);
    }
    content = content
        .child(
            row()
                .gap(8.0)
                .center_y()
                .child(
                    row()
                        .key("page-icon")
                        .role("button")
                        .label("Changer l’icône de la page")
                        .focusable(true)
                        .size(58.0, 58.0)
                        .center()
                        .radius(6.0)
                        .hover(p.tint)
                        .on_press(|s: &mut State| s.panel = Panel::Icons)
                        .child(super::icons::page(&page.emoji, 40.0, p.accent)),
                )
                .child(el().grow(1.0))
                .child(super::icons::action(
                    "cover-cycle",
                    if page.cover == 0 {
                        "Ajouter une couverture"
                    } else {
                        "Couverture"
                    },
                    &g::COVER,
                    p,
                    |s, _| s.appearance(None, true, false),
                )),
        )
        .child(
            col()
                .key("page-heading")
                .role("heading")
                .level(1)
                .label(page.display_title())
                .w_full()
                .child(
                    text_input(&s.title, TextStyle::new(36.0, 650.0))
                        .key("page-title")
                        .label("Titre de la page")
                        .w_full()
                        .single_line()
                        .color(p.ink)
                        .caret(p.accent, p.tint)
                        .placeholder("Sans titre", p.faint)
                        .on_edit(|s: &mut State| &mut s.title)
                        .on_input_change(|s: &mut State, _| s.edit())
                        .on_submit(|s: &mut State, tx| s.focus_active(tx)),
                ),
        )
        .child(
            row()
                .gap(12.0)
                .center_y()
                .child(label(
                    if page.archived {
                        "Page archivée".into()
                    } else if page.dashboard.is_some() {
                        format!(
                            "{} fiches · {} bases",
                            s.tracking_items().len(),
                            s.tracking_sources().len()
                        )
                    } else if page.collection.is_some() {
                        format!(
                            "{} fiches",
                            s.pages
                                .iter()
                                .filter(|item| item.parent_id.as_deref() == Some(&page.id)
                                    && !item.archived)
                                .count()
                        )
                    } else {
                        format!("{words} mots")
                    },
                    11.0,
                    p.faint,
                ))
                .child(label("·", 11.0, p.faint))
                .child(label(
                    if page.dashboard.is_some() {
                        "Suivi dynamique".into()
                    } else if page.collection.is_some() {
                        "Tableau, Kanban, Calendrier et Dashboard".into()
                    } else {
                        format!("{} blocs", s.blocks.len())
                    },
                    11.0,
                    p.faint,
                ))
                .child(if tasks > 0 {
                    label(
                        format!("·   {done}/{tasks} tâches terminées"),
                        11.0,
                        p.accent,
                    )
                } else {
                    el()
                }),
        )
        .child(el().h(1.0).fill(p.line))
        .child(
            if page.parent_id.as_ref().is_some_and(|id| {
                s.pages
                    .iter()
                    .any(|p| p.id == *id && p.collection.is_some())
            }) {
                collections::properties(s, page, p)
            } else {
                el()
            },
        )
        .child(if s.demo && page.id == "demo-icon-catalog" {
            super::icons::catalog(p, width)
        } else if s.demo && page.id == "demo-control-catalog" {
            super::icons::controls_catalog(p, width)
        } else if page.dashboard.is_some() {
            super::dashboard::global(s, p)
        } else if page.collection.is_some() {
            collections::view(s, p, if page.wide { width } else { width.min(770.0) })
        } else {
            col()
                .w_full()
                .gap(10.0)
                .on_drop_files(|s: &mut State, files, _| s.import_files(files))
                .child(super::selection::blocks(s, p))
                .child(blocks::toolbar(s, p, width))
                .child(blocks::view(s, p))
        });
    let children: Vec<_> = s
        .pages
        .iter()
        .filter(|child| child.parent_id.as_deref() == Some(&page.id) && !child.archived)
        .collect();
    if !children.is_empty() && page.collection.is_none() {
        let mut links = col()
            .gap(4.0)
            .py(12.0)
            .child(label("DANS CETTE PAGE", 9.0, p.faint));
        for child in children {
            let id = child.id.clone();
            links = links.child(
                super::icons::page_action(
                    format!("child-link-{id}"),
                    child.display_title(),
                    &child.emoji,
                    p,
                    move |s, tx| {
                        s.select(&id);
                        tx.scroll_to_y("document-scroll", 0.0);
                    },
                )
                .w_full()
                .justify_start()
                .border_sides([0.0, 0.0, 1.0, 0.0], p.line),
            );
        }
        content = content.child(links);
    }
    content = content.child(
        row()
            .gap(12.0)
            .py(20.0)
            .child(super::icons::action(
                "new-child",
                if page.collection.is_some() {
                    "Nouvelle fiche"
                } else {
                    "Ajouter une sous-page"
                },
                &g::PLUS,
                p,
                |s, tx| {
                    if s.current().is_some_and(|p| p.collection.is_some()) {
                        s.create_record(None, tx);
                    } else {
                        s.create(true);
                        tx.focus("page-title", false);
                    }
                },
            ))
            .child(super::icons::action(
                "page-templates",
                "Choisir un modèle",
                &g::TEMPLATES,
                p,
                |s, _| s.panel = Panel::Templates,
            )),
    );
    let backlinks = s.backlinks(&page.id);
    if !backlinks.is_empty() {
        let mut links = col()
            .key("backlinks")
            .w_full()
            .gap(3.0)
            .py(12.0)
            .border_sides([1.0, 0.0, 0.0, 0.0], p.line)
            .child(label(
                format!("{} LIENS RETOUR", backlinks.len()),
                10.0,
                p.faint,
            ));
        for source in backlinks {
            let id = source.id.clone();
            links = links.child(
                super::icons::page_action(
                    format!("backlink-{id}"),
                    source.display_title(),
                    &source.emoji,
                    p,
                    move |s, tx| {
                        s.select(&id);
                        tx.scroll_to_y("document-scroll", 0.0);
                    },
                )
                .w_full()
                .justify_start(),
            );
        }
        content = content.child(links);
    }
    let mut scroll = col()
        .key("document-scroll")
        .grow(1.0)
        .min_h(0.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        });
    if page.cover > 0 {
        scroll = scroll.child(cover(page.cover, p));
    }
    scroll = scroll.child(
        row()
            .w_full()
            .px(35.0)
            .py(25.0)
            .justify_center()
            .child(content),
    );
    if let Some(feed) = s.block_drag.scroll_feed() {
        scroll = scroll
            .on_layout_tx(|s: &mut State, rect, tx| {
                s.block_drag.set_viewport(rect);
                tx.live_only();
            })
            .on_scroll_offset(move |position| feed.set(position));
    }
    col()
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .child(header(s, p))
        .child(super::entity_scroll(scroll, s, "document-scroll"))
}
