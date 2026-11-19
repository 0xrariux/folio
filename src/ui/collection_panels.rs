use super::{
    Palette, action, label,
    panels::{item, surface},
};
use crate::{
    collections::{PropertyField, PropertyForm},
    state::{Panel, State},
};
use folio::collection::{Priority, Sort, Status};
use rune::{El, Key, TextStyle, col, el, row, text_input};

pub(super) fn filter(s: &State, p: Palette) -> El {
    use folio::dashboard::Deadline;
    let filter = s.tracking_filter();
    let mut entries = col()
        .gap(2.0)
        .child(super::panels::icon_item(
            "filter-all",
            "Clear all filters",
            &folio::icons::controls::CLOSE,
            p,
            |s, _| s.clear_tracking(),
        ))
        .child(label("STATUS", 9.0, p.faint).padding(7.0))
        .child(super::icons::option(
            "filter-status-all",
            "All statuses",
            filter.status.is_none(),
            p,
            |s, _| s.tracking_status(None),
        ));
    for status in Status::ALL {
        entries = entries.child(super::icons::option(
            format!("filter-{status:?}"),
            status.name(),
            filter.status == Some(status),
            p,
            move |s, _| s.tracking_status(Some(status)),
        ));
    }
    entries = entries
        .child(label("PRIORITY", 9.0, p.faint).padding(7.0))
        .child(super::icons::option(
            "filter-priority-all",
            "All priorities",
            filter.priority.is_none(),
            p,
            |s, _| s.tracking_priority(None),
        ));
    for priority in Priority::ALL {
        entries = entries.child(super::icons::option(
            format!("filter-priority-{priority:?}"),
            priority.name(),
            filter.priority == Some(priority),
            p,
            move |s, _| s.tracking_priority(Some(priority)),
        ));
    }
    entries = entries.child(label("DUE DATE", 9.0, p.faint).padding(7.0));
    for deadline in Deadline::ALL {
        entries = entries.child(super::icons::option(
            format!("filter-deadline-{deadline:?}"),
            deadline.name(),
            filter.deadline == deadline,
            p,
            move |s, _| s.tracking_deadline(deadline),
        ));
    }
    surface("filter-menu", "collection-filter", 260.0, p).child(
        col()
            .h(370.0)
            .min_h(0.0)
            .scroll(rune::ScrollSpec {
                stick_bottom: false,
            })
            .child(entries),
    )
}

pub(super) fn sort(p: Palette) -> El {
    let mut menu = surface("sort-menu", "collection-sort", 240.0, p)
        .child(label("RECORD ORDER", 9.0, p.faint).padding(9.0));
    for sort in Sort::ALL {
        menu = menu.child(item(
            format!("sort-{sort:?}"),
            sort.name(),
            p,
            move |s, _| s.collection_sort(sort),
        ));
    }
    menu
}
pub(super) fn property(id: &str, field: PropertyField, anchor: &str, p: Palette) -> El {
    let mut menu = surface("property-menu", anchor, 200.0, p);
    match field {
        PropertyField::Status => {
            for status in Status::ALL {
                let id = id.to_string();
                menu = menu.child(item(
                    format!("set-status-{status:?}"),
                    status.name(),
                    p,
                    move |s, _| s.set_status(&id, status),
                ));
            }
        }
        PropertyField::Priority => {
            for priority in Priority::ALL {
                let id = id.to_string();
                menu = menu.child(item(
                    format!("set-priority-{priority:?}"),
                    priority.name(),
                    p,
                    move |s, _| s.set_priority(&id, priority),
                ));
            }
        }
    }
    menu
}
pub(super) fn properties(form: &PropertyForm, p: Palette) -> El {
    let input = |due: bool| {
        text_input(
            if due { &form.due } else { &form.tags },
            TextStyle::new(13.0, 400.0),
        )
        .key(if due { "property-due" } else { "property-tags" })
        .label(if due {
            "Due date"
        } else {
            "Comma-separated tags"
        })
        .single_line()
        .w_full()
        .h(33.0)
        .px(8.0)
        .radius(5.0)
        .border(1.0, p.line)
        .color(p.ink)
        .caret(p.accent, p.tint)
        .placeholder(
            if due {
                "YYYY-MM-DD, or leave blank"
            } else {
                "Project, personal…"
            },
            p.faint,
        )
        .on_edit(move |s: &mut State| s.property_edit(due))
        .on_input_key(|s: &mut State, event, _| {
            if event.key == Key::Escape {
                s.panel = Panel::Closed;
                true
            } else {
                false
            }
        })
        .on_submit(move |s: &mut State, tx| {
            if due {
                tx.focus("property-tags", false);
            } else {
                s.save_properties();
            }
        })
    };
    let mut form_ui = surface("properties-form", &form.anchor, 340.0, p)
        .padding(14.0)
        .gap(10.0)
        .child(label("RECORD PROPERTIES", 10.0, p.faint))
        .child(label("Due date", 12.0, p.ink))
        .child(input(true))
        .child(label("Tags", 12.0, p.ink))
        .child(input(false))
        .child(label("Separate tags with commas.", 10.0, p.faint));
    if let Some(error) = &form.error {
        form_ui = form_ui.child(label(error, 11.0, p.ink).w_full().wrap());
    }
    form_ui.child(
        row()
            .gap(7.0)
            .child(el().grow(1.0))
            .child(action("cancel-properties", "Cancel", p, |s, _| {
                s.panel = Panel::Closed
            }))
            .child(action("save-properties", "Save", p, |s, _| s.save_properties()).fill(p.tint)),
    )
}
pub(super) fn links(s: &State, p: Palette) -> El {
    let Panel::Link(form) = &s.panel else {
        return el();
    };
    let options = s.link_targets();
    let mut rows = col().gap(3.0);
    for page in &options {
        let id = page.id.clone();
        rows = rows.child(
            row()
                .key(format!("link-target-{id}"))
                .role("menuitem")
                .label(page.display_title())
                .focusable(true)
                .h(34.0)
                .px(7.0)
                .gap(8.0)
                .center_y()
                .radius(5.0)
                .hover(p.tint)
                .on_press_tx(move |s: &mut State, tx| s.link_page(&id, tx))
                .child(super::icons::page_icon(page, 16.0, p.muted, p))
                .child(
                    label(page.display_title(), 12.0, p.ink)
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                ),
        );
    }
    if options.is_empty() {
        rows = rows.child(label("No matching pages.", 12.0, p.faint).padding(10.0));
    }
    surface("link-picker", crate::editor::input_key(&form.id), 330.0, p)
        .padding(10.0)
        .gap(8.0)
        .child(label("LINK A NOTEBOOK PAGE", 10.0, p.faint))
        .child(
            text_input(&form.query, TextStyle::new(13.0, 400.0))
                .key("link-search")
                .label("Search for a page to link")
                .single_line()
                .h(32.0)
                .px(8.0)
                .radius(5.0)
                .border(1.0, p.line)
                .color(p.ink)
                .caret(p.accent, p.tint)
                .placeholder("Search pages…", p.faint)
                .on_edit(|s: &mut State| s.link_query())
                .on_input_key(|s: &mut State, event, tx| match event.key {
                    Key::Escape => {
                        s.panel = Panel::Closed;
                        true
                    }
                    Key::Enter => {
                        if let Some(id) = s.link_targets().first().map(|p| p.id.clone()) {
                            s.link_page(&id, tx);
                        }
                        true
                    }
                    _ => false,
                }),
        )
        .child(
            col()
                .h((options.len() as f32 * 37.0).clamp(45.0, 220.0))
                .min_h(0.0)
                .scroll(rune::ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        )
}
