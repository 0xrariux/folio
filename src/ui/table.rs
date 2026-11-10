use super::{
    Palette, action,
    collections::{details_button, priority_button, status_button, title_button},
    icons, label,
};
use crate::state::State;
use folio::{collection::BuiltinColumn, fields::Column};
use rune::{El, TextStyle, col, el, row};

pub(super) fn column_heading(title: &str, p: Palette) -> El {
    label(title, 11.0, p.muted)
        .font(TextStyle::new(11.0, 500.0))
        .role("columnheader")
        .min_w(0.0)
        .truncate()
}

pub(super) fn header_row(key: impl Into<String>, height: f32, p: Palette) -> El {
    row()
        .key(key)
        .role("row")
        .w_full()
        .sticky_top(0.0)
        .h(height)
        .px(10.0)
        .gap(8.0)
        .center_y()
        .fill(p.sidebar)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.line)
        .border_layout()
}

/// Keep the outer stroke outside the scrolling content, with exact content
/// height and separators only between rows, so the final row has one border.
pub(super) fn frame(
    key: impl Into<String>,
    scroll_key: impl Into<String>,
    title: &str,
    content: El,
    content_height: f32,
    max_height: f32,
    p: Palette,
) -> El {
    col()
        .key(key)
        .w_full()
        .h((content_height + 2.0).min(max_height))
        .min_h(0.0)
        .radius(6.0)
        .clip_rounded(6.0)
        .fill(p.bg)
        .border(1.0, p.line)
        .border_layout()
        .child(
            col()
                .key(scroll_key)
                .role("table")
                .label(title)
                .w_full()
                .h_full()
                .min_h(0.0)
                .scroll_xy()
                // Rune stretches this wrapper to the viewport; the table inside
                // can keep its pixel minimum without overriding that width floor.
                .child(col().w_full().child(content)),
        )
}

enum Field<'a> {
    Builtin(BuiltinColumn),
    Custom(&'a Column),
}
impl Field<'_> {
    fn id(&self) -> String {
        match self {
            Self::Builtin(c) => format!("{c:?}"),
            Self::Custom(c) => c.id.clone(),
        }
    }
    fn name(&self) -> &str {
        match self {
            Self::Builtin(c) => c.name(),
            Self::Custom(c) => &c.name,
        }
    }
    fn width(&self) -> f32 {
        match self {
            Self::Builtin(c) => c.width(),
            Self::Custom(_) => 140.0,
        }
    }
}
fn header(id: &str, title: &str, width: f32, default: f32, p: Palette) -> El {
    let key = id.to_string();
    row()
        .w(width)
        .gap(3.0)
        .center_y()
        .child(column_heading(title, p).grow(1.0))
        .child(
            el().key(format!("resize-column-{id}"))
                .w(6.0)
                .h(24.0)
                .radius(2.0)
                .hover(p.line)
                .help("Glisser pour ajuster la largeur")
                .child(el().w(1.0).h_full().fill(p.line))
                .on_drag(move |s: &mut State, drag, _| s.resize_column(&key, default, drag)),
        )
}
pub(super) fn view(s: &State, p: Palette) -> El {
    let items = s.collection_items();
    let settings = s.current().and_then(|p| p.collection.as_ref()).unwrap();
    let mut fields: Vec<_> = BuiltinColumn::ALL
        .into_iter()
        .filter(|c| !settings.hidden.contains(c))
        .map(Field::Builtin)
        .chain(
            settings
                .columns
                .iter()
                .filter(|c| c.visible)
                .map(Field::Custom),
        )
        .collect();
    fields.sort_by_key(|f| {
        settings
            .layout
            .order
            .iter()
            .position(|id| id == &f.id())
            .unwrap_or(usize::MAX)
    });
    let name_width = settings.width("name", 190.0);
    let min_width = 48.0
        + name_width
        + fields
            .iter()
            .map(|c| settings.width(&c.id(), c.width()) + 8.0)
            .sum::<f32>();
    let selected = !items.is_empty()
        && items
            .iter()
            .all(|p| s.workspace.selected_records.contains(&p.id));
    let ids: Vec<_> = items.iter().map(|p| p.id.clone()).collect();
    let mut head = header_row("collection-table-header", 40.0, p)
        .child(
            super::button(
                "select-all-records",
                "Sélectionner toutes les fiches visibles",
                p,
                move |s, _| {
                    if selected {
                        for id in &ids {
                            s.workspace.selected_records.remove(id);
                        }
                    } else {
                        s.workspace.selected_records.extend(ids.iter().cloned());
                    }
                },
            )
            .role("checkbox")
            .checked(selected)
            .size(20.0, 25.0)
            .px(0.0)
            .child(icons::checkbox_mark(selected, p)),
        )
        .child(
            el().w(name_width)
                .child(header("name", "Aa  Nom", name_width, 190.0, p)),
        );
    for field in &fields {
        head = head.child(header(
            &field.id(),
            field.name(),
            settings.width(&field.id(), field.width()),
            field.width(),
            p,
        ));
    }
    let mut rows = col()
        .key("collection-table-content")
        .w_full()
        .min_w(min_width)
        .child(head);
    for (index, page) in items.iter().enumerate() {
        if index > 0 {
            rows = rows.child(
                el().key(format!("table-separator-{}", page.id))
                    .w_full()
                    .h(1.0)
                    .fill(p.line),
            );
        }
        let id = page.id.clone();
        let selected = s.workspace.selected_records.contains(&id);
        let mut cells = row()
            .key(format!("table-row-{id}"))
            .role("row")
            .h(44.0)
            .px(10.0)
            .gap(8.0)
            .center_y()
            .fill(if selected { p.tint } else { p.bg })
            .child(
                super::button(
                    format!("select-record-{id}"),
                    "Sélectionner cette fiche",
                    p,
                    move |s, _| {
                        if !s.workspace.selected_records.insert(id.clone()) {
                            s.workspace.selected_records.remove(&id);
                        }
                    },
                )
                .size(20.0, 25.0)
                .px(0.0)
                .role("checkbox")
                .checked(selected)
                .child(icons::checkbox_mark(selected, p)),
            )
            .child(
                el().w(name_width)
                    .child(title_button(page, p).w_full().min_w(0.0)),
            );
        for field in &fields {
            let cell = match field {
                Field::Builtin(BuiltinColumn::Status) => status_button(page, p),
                Field::Builtin(BuiltinColumn::Priority) => priority_button(page, p),
                Field::Builtin(BuiltinColumn::Due) => {
                    let id = page.id.clone();
                    let anchor = format!("due-{id}");
                    let key = anchor.clone();
                    action(
                        key,
                        page.properties.due.as_deref().unwrap_or("—"),
                        p,
                        move |s, tx| {
                            s.open_date(&id, None, &anchor);
                            tx.focus("date-value", false);
                        },
                    )
                    .px(3.0)
                }
                Field::Builtin(BuiltinColumn::Tags) => details_button(
                    page,
                    if page.properties.tags.is_empty() {
                        "—"
                    } else {
                        "Étiquettes…"
                    },
                    p,
                ),
                Field::Custom(c) => super::custom_fields::cell(s, page, c, p),
            };
            cells = cells.child(
                el().w(settings.width(&field.id(), field.width()))
                    .child(cell),
            );
        }
        rows = rows.child(cells);
    }
    if items.is_empty() {
        rows = rows.child(label("Aucune fiche pour ce filtre.", 12.0, p.faint).padding(18.0));
    }
    let height = 40.0
        + if items.is_empty() {
            54.0
        } else {
            items.len() as f32 * 44.0 + items.len().saturating_sub(1) as f32
        };
    frame(
        "collection-table-frame",
        "collection-table",
        "Fiches de la base",
        rows,
        height,
        380.0,
        p,
    )
}
