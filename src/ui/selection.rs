use super::{Palette, action, icons, label};
use crate::state::State;
use folio::{
    collection::{Priority, Status},
    document::Kind,
};
use rune::{El, col, el, row};
pub(super) fn blocks(s: &State, p: Palette) -> El {
    if s.selection.ids.is_empty() {
        return el();
    }
    let mut bar = col()
        .key("block-selection-toolbar")
        .w_full()
        .gap(4.0)
        .padding(7.0)
        .radius(6.0)
        .fill(p.tint)
        .child(
            row()
                .gap(4.0)
                .center_y()
                .child(label(
                    format!("{} blocks", s.selected_indices().len()),
                    11.0,
                    p.ink,
                ))
                .child(action("selection-copy", "Copy", p, |s, tx| {
                    let text = s.copy_selected();
                    tx.copy_rich(
                        rune::ClipboardContent::new(text)
                            .with_html(folio::outline::html(&s.selection.clipboard)),
                    );
                }))
                .child(action("selection-paste", "Paste", p, |s, tx| {
                    s.paste_selected(tx)
                }))
                .child(action("selection-duplicate", "Duplicate", p, |s, tx| {
                    s.duplicate_selected(tx)
                }))
                .child(action("selection-delete", "Delete", p, |s, tx| {
                    s.delete_selected(tx)
                }))
                .child(icons::control(
                    "selection-clear",
                    "Deselect",
                    &folio::icons::controls::CLOSE,
                    p,
                    |s, _| s.selection.clear(),
                )),
        );
    let mut types = row().gap(3.0).center_y();
    for kind in [
        Kind::Text,
        Kind::Bullet,
        Kind::Numbered,
        Kind::Todo,
        Kind::Toggle,
    ] {
        types = types.child(icons::control(
            format!("selection-kind-{kind:?}"),
            kind.name(),
            folio::icons::controls::block(kind),
            p,
            move |s, _| s.transform_selected(kind),
        ));
    }
    types = types
        .child(el().grow(1.0))
        .child(icons::control(
            "selection-up",
            "Move selection up",
            &folio::icons::controls::ARROW_UP,
            p,
            |s, _| s.move_selected(false),
        ))
        .child(icons::control(
            "selection-down",
            "Move selection down",
            &folio::icons::controls::ARROW_DOWN,
            p,
            |s, _| s.move_selected(true),
        ));
    bar = bar.child(types);
    bar
}
pub(super) fn records(s: &State, p: Palette) -> El {
    if s.workspace.selected_records.is_empty() {
        return el();
    }
    let mut status = row().gap(4.0).center_y().child(label(
        format!("{} records", s.workspace.selected_records.len()),
        11.0,
        p.ink,
    ));
    for value in Status::ALL {
        status = status.child(action(
            format!("bulk-status-{value:?}"),
            value.name(),
            p,
            move |s, _| {
                let ids: Vec<_> = s.workspace.selected_records.iter().cloned().collect();
                for id in ids {
                    s.set_status(&id, value);
                }
                s.workspace.selected_records.clear();
            },
        ));
    }
    status = status.child(icons::control(
        "records-clear",
        "Deselect",
        &folio::icons::controls::CLOSE,
        p,
        |s, _| s.workspace.selected_records.clear(),
    ));
    let mut priorities = row()
        .gap(4.0)
        .center_y()
        .child(label("Priority", 11.0, p.muted));
    for value in Priority::ALL {
        priorities = priorities.child(action(
            format!("bulk-priority-{value:?}"),
            value.name(),
            p,
            move |s, _| {
                let ids: Vec<_> = s.workspace.selected_records.iter().cloned().collect();
                for id in ids {
                    s.set_priority(&id, value);
                }
                s.workspace.selected_records.clear();
            },
        ));
    }
    priorities = priorities.child(action("bulk-due", "Due date…", p, |s, _| {
        if let Some(id) = s.workspace.selected_records.first().cloned() {
            s.open_date(&id, None, "bulk-due");
            if let crate::state::Panel::Date(f) = &mut s.panel {
                f.record = "__selection__".into();
            }
        }
    }));
    col()
        .w_full()
        .padding(5.0)
        .gap(3.0)
        .radius(5.0)
        .fill(p.tint)
        .child(status)
        .child(priorities)
}
