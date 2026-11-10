use super::{Palette, action, label, rgb};
use crate::{
    collections::PropertyField,
    state::{Panel, State},
};
use folio::icons::controls as g;
use folio::{
    collection::{Priority, Status, View},
    model::Page,
};
use rune::{El, ScrollSpec, TextStyle, col, el, row, text_input};

pub(super) fn status_color(status: Status, p: Palette) -> rune::Color {
    match status {
        Status::Todo => p.muted,
        Status::InProgress => {
            if p.bg.luminance() < 0.1 {
                rgb(140, 185, 211)
            } else {
                rgb(54, 105, 137)
            }
        }
        Status::Done => p.accent,
    }
}
fn priority_color(priority: Priority, p: Palette) -> rune::Color {
    match priority {
        Priority::Low => p.faint,
        Priority::Medium => p.muted,
        Priority::High => {
            if p.bg.luminance() < 0.1 {
                rgb(232, 176, 121)
            } else {
                rgb(164, 87, 40)
            }
        }
    }
}
pub(super) fn status_button(page: &Page, p: Palette) -> El {
    let id = page.id.clone();
    let anchor = format!("status-{id}");
    let key = anchor.clone();
    super::button(key, page.properties.status.name(), p, move |s, _| {
        s.panel = Panel::Property {
            id: id.clone(),
            field: PropertyField::Status,
            anchor: anchor.clone(),
        }
    })
    .gap(6.0)
    .child(
        rune::icon(&g::STATUS_DOT, 14.0)
            .color(status_color(page.properties.status, p))
            .a11y_hidden(),
    )
    .child(label(page.properties.status.name(), 12.0, p.muted))
    .label(format!("Statut : {}", page.properties.status.name()))
    .fill(status_color(page.properties.status, p).alpha(0.12))
    .h(27.0)
}
pub(super) fn priority_button(page: &Page, p: Palette) -> El {
    let id = page.id.clone();
    let anchor = format!("priority-{id}");
    let key = anchor.clone();
    super::icons::action(
        key,
        page.properties.priority.name(),
        &g::FLAG,
        p,
        move |s, _| {
            s.panel = Panel::Property {
                id: id.clone(),
                field: PropertyField::Priority,
                anchor: anchor.clone(),
            }
        },
    )
    .label(format!("Priorité : {}", page.properties.priority.name()))
    .fill(priority_color(page.properties.priority, p).alpha(0.1))
    .h(27.0)
}
pub(super) fn details_button(page: &Page, title: &str, p: Palette) -> El {
    let id = page.id.clone();
    let anchor = format!("details-{id}");
    let key = anchor.clone();
    let edit = move |s: &mut State, tx: &mut rune::Tx| {
        s.open_properties(&id, &anchor);
        tx.focus("property-due", false);
    };
    if title == "•••" {
        super::icons::control(key, "Éditer échéance et étiquettes", &g::MORE, p, edit)
    } else {
        super::button(key, title, p, edit)
            .justify_start()
            .px(6.0)
            .min_w(0.0)
            .child(label(title, 12.0, p.muted).min_w(0.0).truncate())
    }
}
pub(super) fn title_button(page: &Page, p: Palette) -> El {
    let id = page.id.clone();
    row()
        .key(format!("record-{id}"))
        .role("button")
        .label(page.display_title())
        .focusable(true)
        .grow(1.0)
        .min_w(130.0)
        .h(33.0)
        .gap(7.0)
        .center_y()
        .radius(4.0)
        .hover(p.tint)
        .on_press_tx(move |s: &mut State, tx| {
            if s.entity_click(&id) {
                return;
            }
            s.select(&id);
            tx.scroll_to_y("document-scroll", 0.0);
        })
        .child(super::icons::page(&page.emoji, 16.0, p.muted))
        .child(
            label(page.display_title(), 13.0, p.ink)
                .grow(1.0)
                .min_w(0.0)
                .truncate(),
        )
}
fn board(s: &State, p: Palette, width: f32) -> El {
    let items = s.collection_items();
    let mut columns = row().w(width.max(630.0)).h_full().gap(12.0);
    for status in Status::ALL {
        let selected: Vec<_> = items
            .iter()
            .filter(|page| page.properties.status == status)
            .collect();
        let mut cards = col().gap(8.0).w_full();
        for page in selected.iter().copied() {
            let forward_id = page.id.clone();
            let back_id = page.id.clone();
            let drag_id = page.id.clone();
            let mut card = col()
                .key(format!("card-{}", page.id))
                .w_full()
                .min_w(0.0)
                .padding(10.0)
                .gap(8.0)
                .radius(6.0)
                .fill(p.bg)
                .border(1.0, p.line)
                .child(
                    title_button(page, p)
                        .min_w(0.0)
                        .on_drag(move |s: &mut State, d, tx| {
                            s.drag_entity(&drag_id, crate::entity_drag::Kind::Card, d, tx)
                        }),
                )
                .child(
                    row()
                        .gap(5.0)
                        .center_y()
                        .child(
                            rune::icon(&g::FLAG, 12.0)
                                .color(priority_color(page.properties.priority, p))
                                .a11y_hidden(),
                        )
                        .child(label(
                            page.properties.priority.name(),
                            10.0,
                            priority_color(page.properties.priority, p),
                        )),
                );
            if let Some(due) = &page.properties.due {
                card = card.child(
                    row()
                        .gap(5.0)
                        .center_y()
                        .child(rune::icon(&g::CLOCK, 12.0).color(p.faint).a11y_hidden())
                        .child(label(due, 10.0, p.faint)),
                );
            }
            if !page.properties.tags.is_empty() {
                card = card.child(
                    label(page.properties.tags.join(" · "), 10.0, p.accent)
                        .w_full()
                        .truncate(),
                );
            }
            card = card.child(
                row()
                    .gap(2.0)
                    .center_y()
                    .child(details_button(page, "•••", p).label("Éditer échéance et étiquettes"))
                    .child(el().grow(1.0))
                    .child(
                        super::icons::control(
                            format!("board-back-{}", page.id),
                            "Reculer le statut",
                            &g::CHEVRON_LEFT,
                            p,
                            move |s, _| s.set_status(&back_id, status.next(false)),
                        )
                        .label("Reculer le statut"),
                    )
                    .child(
                        super::icons::control(
                            format!("board-forward-{}", page.id),
                            "Avancer le statut",
                            &g::CHEVRON_RIGHT,
                            p,
                            move |s, _| s.set_status(&forward_id, status.next(true)),
                        )
                        .label("Avancer le statut"),
                    ),
            );
            cards = cards.child(card);
        }
        if selected.is_empty() {
            cards = cards.child(label("Rien ici pour le moment.", 11.0, p.faint).padding(8.0));
        }
        columns = columns.child(
            col()
                .key(format!("column-{status:?}"))
                .border(
                    1.0,
                    if super::entity_target(s, &format!("board-drop-{status:?}")) {
                        p.accent
                    } else {
                        p.line
                    },
                )
                .border_layout()
                .grow(1.0)
                .min_w(200.0)
                .min_h(0.0)
                .h_full()
                .padding(10.0)
                .gap(9.0)
                .radius(7.0)
                .fill(p.sidebar)
                .child(
                    row()
                        .gap(6.0)
                        .center_y()
                        .child(
                            rune::icon(&g::STATUS_DOT, 12.0)
                                .color(status_color(status, p))
                                .a11y_hidden(),
                        )
                        .child(label(status.name(), 12.0, p.ink))
                        .child(label(selected.len().to_string(), 10.0, p.faint)),
                )
                .child(super::entity_scroll(
                    col()
                        .key(format!("board-scroll-{status:?}"))
                        .grow(1.0)
                        .min_h(0.0)
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(cards),
                    s,
                    &format!("board-scroll-{status:?}"),
                ))
                .child(super::icons::action(
                    format!("add-{status:?}"),
                    "Nouvelle fiche",
                    &g::PLUS,
                    p,
                    move |s, tx| s.create_record(Some(status), tx),
                ))
                .child(
                    el().key(format!("board-drop-{status:?}"))
                        .absolute(0.0, 0.0)
                        .w_full()
                        .h_full(),
                ),
        );
    }
    col()
        .key("collection-board")
        .w_full()
        .h(370.0)
        .min_h(0.0)
        .scroll_x()
        .child(columns)
}
pub(super) fn search(s: &State, p: Palette) -> El {
    row()
        .grow(1.0)
        .min_w(110.0)
        .h(31.0)
        .px(8.0)
        .gap(6.0)
        .center_y()
        .radius(5.0)
        .border(1.0, p.line)
        .child(rune::icon(&g::SEARCH, 16.0).color(p.faint).a11y_hidden())
        .child(
            text_input(&s.collection_search, TextStyle::new(12.0, 400.0))
                .key("collection-search")
                .label("Rechercher une fiche")
                .single_line()
                .grow(1.0)
                .min_w(0.0)
                .color(p.ink)
                .caret(p.accent, p.tint)
                .placeholder("Rechercher une fiche…", p.faint)
                .on_edit(|s: &mut State| &mut s.collection_search),
        )
}
pub(super) fn view(s: &State, p: Palette, width: f32) -> El {
    let Some(settings) = s.current().and_then(|page| page.collection.as_ref()) else {
        return el();
    };
    let count = s.collection_items().len();
    let mut tabs = row()
        .key("collection-tabs")
        .w_full()
        .h(36.0)
        .gap(5.0)
        .center_y();
    for (key, title, view, glyph) in [
        ("view-table", "Tableau", View::Table, &g::TABLE),
        ("view-board", "Kanban", View::Board, &g::BOARD),
        (
            "view-calendar",
            "Calendrier",
            View::Calendar,
            &folio::icons::CALENDAR,
        ),
        (
            "view-dashboard",
            "Dashboard",
            View::Dashboard,
            &folio::icons::DASHBOARD,
        ),
    ] {
        tabs = tabs.child(
            super::icons::action(key, title, glyph, p, move |s, _| s.collection_view(view)).fill(
                if settings.view == view {
                    p.tint
                } else {
                    rune::Color::TRANSPARENT
                },
            ),
        );
    }
    tabs = tabs.child(el().grow(1.0)).child(label(
        format!("{} fiche{}", count, if count == 1 { "" } else { "s" }),
        10.0,
        p.faint,
    ));
    let filter = s.tracking_filter();
    let mut body = col()
        .w_full()
        .gap(12.0)
        .instant_changes()
        .child(
            row()
                .center_y()
                .gap(6.0)
                .child(
                    super::icons::action(
                        "collection-views",
                        settings
                            .active()
                            .map_or("Vues enregistrées", |v| v.name.as_str()),
                        &g::TEMPLATES,
                        p,
                        |s, tx| {
                            s.open_views(false);
                            tx.focus("view-name", false);
                        },
                    )
                    .fill(p.tint)
                    .child(
                        rune::icon(&g::CHEVRON_DOWN, 12.0)
                            .color(p.muted)
                            .a11y_hidden(),
                    ),
                )
                .child(label("Filtres et tri propres à chaque vue", 10.0, p.faint)),
        )
        .child(
            col()
                .key("collection-view-header")
                .w_full()
                .gap(7.0)
                .child(tabs)
                .child(
                    el().key("collection-tabs-divider")
                        .w_full()
                        .h(1.0)
                        .fill(p.line),
                ),
        )
        .child(
            row()
                .key("collection-tools")
                .gap(7.0)
                .center_y()
                .child(search(s, p))
                .child(
                    super::icons::action(
                        "collection-filter",
                        if filter.active() {
                            "Filtres actifs"
                        } else {
                            "Filtrer"
                        },
                        &g::FILTER,
                        p,
                        |s, _| s.panel = Panel::CollectionFilter,
                    )
                    .fill(if filter.active() { p.tint } else { p.bg }),
                )
                .child(super::icons::action(
                    "collection-sort",
                    "Trier",
                    &g::SORT,
                    p,
                    |s, _| s.panel = Panel::CollectionSort,
                ))
                .child(super::icons::action(
                    "collection-columns",
                    "Colonnes",
                    &g::COLUMNS,
                    p,
                    |s, tx| {
                        s.open_columns();
                        tx.focus("column-name", false);
                    },
                ))
                .child(
                    super::icons::action("new-record", "Nouveau", &g::PLUS, p, |s, tx| {
                        s.create_record(None, tx)
                    })
                    .fill(p.tint),
                ),
        );
    if filter.active() || !s.collection_search.text.is_empty() {
        body = body.child(action(
            "reset-tracking",
            "Effacer les filtres et la recherche",
            p,
            |s, _| s.clear_tracking(),
        ));
    }
    body = body.child(super::selection::records(s, p));
    body.child(match settings.view {
        View::Table => super::table::view(s, p, width),
        View::Board => board(s, p, width),
        View::Calendar => super::calendar::view(s, p),
        View::Dashboard => super::dashboard::view(s, p, width),
    })
}

/// The same label width and value inset apply to built-in and custom properties.
pub(super) fn property_row(key: &str, title: &str, value: El, p: Palette) -> El {
    row()
        .key(format!("record-property-{key}"))
        .w_full()
        .gap(12.0)
        .center_y()
        .child(label(title, 11.0, p.faint).w(150.0).truncate().help(title))
        .child(
            el().key(format!("property-value-{key}"))
                .grow(1.0)
                .min_w(0.0)
                .child(value),
        )
}

pub(super) fn properties(s: &State, page: &Page, p: Palette) -> El {
    col()
        .key("record-properties")
        .w_full()
        .gap(7.0)
        .py(7.0)
        .child(property_row(
            "status",
            "Statut",
            status_button(page, p).px(6.0).justify_start(),
            p,
        ))
        .child(property_row(
            "priority",
            "Priorité",
            priority_button(page, p).px(6.0).justify_start(),
            p,
        ))
        .child(property_row(
            "due",
            "Échéance",
            details_button(
                page,
                page.properties
                    .due
                    .as_deref()
                    .unwrap_or("Ajouter une échéance ou des étiquettes"),
                p,
            ),
            p,
        ))
        .child(if page.properties.tags.is_empty() {
            el()
        } else {
            property_row(
                "tags",
                "Étiquettes",
                label(page.properties.tags.join(" · "), 11.0, p.accent)
                    .px(6.0)
                    .w_full()
                    .wrap(),
                p,
            )
        })
        .child(super::custom_fields::record_fields(s, page, p))
}
