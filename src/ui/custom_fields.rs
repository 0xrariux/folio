use super::{
    Palette, action, label,
    panels::{item, surface},
};
use crate::{
    custom_fields::{CellForm, ColumnForm},
    state::{Panel, State},
};
use folio::{
    collection::BuiltinColumn,
    fields::{CellValue, Column, ColumnKind},
};
use rune::{El, Key, TextStyle, col, el, row, text_input};

pub(super) fn cell(s: &State, record: &folio::model::Page, column: &Column, p: Palette) -> El {
    let key = format!("cell-{}-{}", record.id, column.id);
    cell_with_key(s, record, column, p, key)
}
pub(super) fn cell_with_key(
    s: &State,
    record: &folio::model::Page,
    column: &Column,
    p: Palette,
    key: String,
) -> El {
    let id = record.id.clone();
    let column_id = column.id.clone();
    if column.kind == ColumnKind::Checkbox {
        let checked = matches!(
            record.properties.custom.get(&column.id),
            Some(CellValue::Checkbox(true))
        );
        return row()
            .key(key)
            .role("checkbox")
            .label(&column.name)
            .checked(checked)
            .focusable(true)
            .size(19.0, 19.0)
            .radius(3.0)
            .border(1.0, if checked { p.accent } else { p.faint })
            .fill(if checked { p.accent } else { p.bg })
            .center()
            .on_press(move |s: &mut State| s.check_cell(&id, &column_id))
            .child(if checked {
                rune::icon(&folio::icons::controls::CHECK, 13.0)
                    .color(p.bg)
                    .a11y_hidden()
            } else {
                el()
            });
    }
    let title = folio::calculations::display(record, column, &s.pages);
    if matches!(column.kind, ColumnKind::Rollup | ColumnKind::Formula) {
        return row()
            .key(key)
            .role("cell")
            .label(format!("{} : {title}", column.name))
            .w_full()
            .h(31.0)
            .px(6.0)
            .center_y()
            .help("Valeur calculée automatiquement")
            .child(label(&title, 12.0, p.ink).min_w(0.0).truncate());
    }
    let kind = column.kind;
    let anchor = key.clone();
    row()
        .key(key)
        .role("button")
        .label(format!("Modifier {} : {}", column.name, title))
        .focusable(true)
        .h(31.0)
        .w_full()
        .px(6.0)
        .center_y()
        .radius(4.0)
        .hover(p.tint)
        .on_press_tx(move |s: &mut State, tx| {
            s.open_cell(&id, &column_id, &anchor);
            tx.focus(
                match kind {
                    ColumnKind::Relation => "palette-search",
                    ColumnKind::Date => "date-value",
                    _ => "cell-value",
                },
                false,
            );
        })
        .child(label(title, 12.0, p.ink).grow(1.0).min_w(0.0).truncate())
}

pub(super) fn record_fields(s: &State, page: &folio::model::Page, p: Palette) -> El {
    let columns = page
        .parent_id
        .as_ref()
        .and_then(|id| s.pages.iter().find(|p| p.id == *id))
        .and_then(|p| p.collection.as_ref())
        .map(|c| c.columns.iter().filter(|c| c.visible).collect::<Vec<_>>())
        .unwrap_or_default();
    let mut fields = col().w_full().gap(7.0);
    for column in columns {
        let value = cell(s, page, column, p);
        fields = fields.child(super::collections::property_row(
            &column.id,
            &column.name,
            if column.kind == ColumnKind::Checkbox {
                value.ml(6.0)
            } else {
                value
            },
            p,
        ));
    }
    fields
}

pub(super) fn columns(s: &State, form: &ColumnForm, p: Palette) -> El {
    let Some(settings) = s
        .pages
        .iter()
        .find(|page| page.id == form.collection_id)
        .and_then(|page| page.collection.as_ref())
    else {
        return el();
    };
    let mut list = col().gap(2.0);
    for builtin in BuiltinColumn::ALL {
        let visible = !settings.hidden.contains(&builtin);
        let key = format!("{builtin:?}");
        let up = key.clone();
        let down = key.clone();
        list = list.child(
            row()
                .w_full()
                .gap(3.0)
                .child(
                    super::icons::toggle(
                        format!("builtin-{builtin:?}"),
                        builtin.name(),
                        visible,
                        p,
                        move |s, _| s.toggle_builtin(builtin),
                    )
                    .grow(1.0)
                    .min_w(0.0),
                )
                .child(super::icons::control(
                    format!("column-up-{key}"),
                    "Déplacer à gauche",
                    &folio::icons::controls::ARROW_UP,
                    p,
                    move |s, _| s.move_column(&up, false),
                ))
                .child(super::icons::control(
                    format!("column-down-{key}"),
                    "Déplacer à droite",
                    &folio::icons::controls::ARROW_DOWN,
                    p,
                    move |s, _| s.move_column(&down, true),
                )),
        );
    }
    for column in &settings.columns {
        let id = column.id.clone();
        let edit_id = id.clone();
        let up_id = id.clone();
        let down_id = id.clone();
        list = list.child(
            row()
                .gap(3.0)
                .center_y()
                .child(
                    super::button(
                        format!("toggle-column-{id}"),
                        &format!("Afficher la colonne {}", column.name),
                        p,
                        move |s, _| s.toggle_column(&id),
                    )
                    .role("checkbox")
                    .checked(column.visible)
                    .size(30.0, 30.0)
                    .px(6.0)
                    .help(format!("Afficher ou masquer {}", column.name))
                    .child(super::icons::checkbox_mark(column.visible, p)),
                )
                .child(
                    label(&column.name, 12.0, p.ink)
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                )
                .child(super::icons::control(
                    format!("column-up-{}", column.id),
                    "Déplacer la colonne vers la gauche",
                    &folio::icons::controls::ARROW_UP,
                    p,
                    move |s, _| s.move_column(&up_id, false),
                ))
                .child(super::icons::control(
                    format!("column-down-{}", column.id),
                    "Déplacer la colonne vers la droite",
                    &folio::icons::controls::ARROW_DOWN,
                    p,
                    move |s, _| s.move_column(&down_id, true),
                ))
                .child(action(
                    format!("rename-column-{}", column.id),
                    "Renommer",
                    p,
                    move |s, tx| {
                        s.edit_column(&edit_id);
                        tx.focus("column-name", false);
                    },
                )),
        );
    }
    let mut menu = col()
        .key("columns-form")
        .w_full()
        .padding(12.0)
        .gap(8.0)
        .child(
            row()
                .center_y()
                .child(label("COLONNES DU TABLEAU", 10.0, p.faint))
                .child(el().grow(1.0))
                .child(super::icons::control(
                    "close-columns",
                    "Fermer les colonnes",
                    &folio::icons::controls::CLOSE,
                    p,
                    |s, _| s.panel = Panel::Closed,
                )),
        )
        .child(
            col()
                .h(((settings.columns.len() + 4) as f32 * 34.0).min(180.0))
                .min_h(0.0)
                .scroll(rune::ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
        .child(el().h(1.0).fill(p.line))
        .child(label(
            if form.id.is_some() {
                "RENOMMER UNE COLONNE"
            } else {
                "NOUVELLE COLONNE"
            },
            10.0,
            p.faint,
        ))
        .child(
            text_input(&form.name, TextStyle::new(13.0, 400.0))
                .key("column-name")
                .label("Nom de la colonne")
                .single_line()
                .w_full()
                .h(32.0)
                .px(8.0)
                .border(1.0, p.line)
                .radius(5.0)
                .color(p.ink)
                .caret(p.accent, p.tint)
                .placeholder("Ex. Charge (h), Référent…", p.faint)
                .on_edit(|s: &mut State| s.column_name_edit())
                .on_submit(|s: &mut State, _| s.save_column())
                .on_input_key(|s: &mut State, event, _| {
                    if event.key == Key::Escape {
                        s.panel = Panel::Closed;
                        true
                    } else {
                        false
                    }
                }),
        );
    if form.id.is_none() {
        for kinds in ColumnKind::ALL.chunks(2) {
            let mut buttons = row().gap(5.0);
            for kind in kinds {
                let kind = *kind;
                buttons = buttons.child(
                    super::icons::action(
                        format!("column-kind-{kind:?}"),
                        kind.name(),
                        match kind {
                            ColumnKind::Text => &folio::icons::controls::TEXT,
                            ColumnKind::Number => &folio::icons::controls::NUMBER,
                            ColumnKind::Date => &folio::icons::CALENDAR,
                            ColumnKind::Checkbox => &folio::icons::TASKS,
                            ColumnKind::Relation => &folio::icons::LINK,
                            ColumnKind::Rollup | ColumnKind::Formula => {
                                &folio::icons::controls::NUMBER
                            }
                        },
                        p,
                        move |s, tx| {
                            s.column_kind(kind);
                            tx.scroll_into_view("column-config");
                        },
                    )
                    .grow(1.0)
                    .fill(if form.kind == kind { p.tint } else { p.bg }),
                );
            }
            menu = menu.child(buttons);
        }
    } else {
        menu = menu.child(label(
            format!("Type conservé : {}", form.kind.name()),
            11.0,
            p.faint,
        ));
    }
    menu = menu.child(super::field_config::config(s, form, p));
    if let Some(error) = &form.error {
        menu = menu.child(label(error, 11.0, p.ink).w_full().wrap());
    }
    let content = menu.child(
        super::icons::action(
            "save-column",
            if form.id.is_some() {
                "Enregistrer la propriété"
            } else {
                "Ajouter la colonne"
            },
            if form.id.is_some() {
                &folio::icons::controls::CHECK
            } else {
                &folio::icons::controls::PLUS
            },
            p,
            |s, _| s.save_column(),
        )
        .fill(p.tint),
    );
    el().key("column-overlay")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .center()
        .fill(rune::Color::BLACK.alpha(0.25))
        .modal(true, |s: &mut State| s.panel = Panel::Closed)
        .child(
            col()
                .w(460.0)
                .h(500.0)
                .radius(9.0)
                .fill(p.bg)
                .border(1.0, p.line)
                .child(
                    col()
                        .key("column-form-scroll")
                        .w_full()
                        .h_full()
                        .min_h(0.0)
                        .scroll(rune::ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(content),
                ),
        )
}

pub(super) fn cell_form(s: &State, form: &CellForm, p: Palette) -> El {
    let column = s.field_column(&form.record_id, &form.column_id);
    let title = column.map_or("Valeur", |c| c.name.as_str());
    let hint = match column.map(|c| c.kind) {
        Some(ColumnKind::Number) => "Un nombre, ou vide pour effacer",
        Some(ColumnKind::Date) => "AAAA-MM-JJ, ou vide pour effacer",
        _ => "Une valeur, ou vide pour effacer",
    };
    let mut menu = surface("cell-form", &form.anchor, 320.0, p)
        .padding(12.0)
        .gap(10.0)
        .child(label(title, 13.0, p.ink))
        .child(
            text_input(&form.value, TextStyle::new(13.0, 400.0))
                .key("cell-value")
                .label(title)
                .single_line()
                .w_full()
                .h(34.0)
                .px(8.0)
                .radius(5.0)
                .border(1.0, p.line)
                .color(p.ink)
                .caret(p.accent, p.tint)
                .placeholder(hint, p.faint)
                .on_edit(|s: &mut State| s.cell_edit())
                .on_submit(|s: &mut State, _| s.save_cell())
                .on_input_key(|s: &mut State, event, _| {
                    if event.key == Key::Escape {
                        s.panel = Panel::Closed;
                        true
                    } else {
                        false
                    }
                }),
        );
    if let Some(error) = &form.error {
        menu = menu.child(label(error, 11.0, p.ink).w_full().wrap());
    }
    menu.child(
        row()
            .gap(6.0)
            .child(el().grow(1.0))
            .child(action("cancel-cell", "Annuler", p, |s, _| {
                s.panel = Panel::Closed
            }))
            .child(action("save-cell", "Enregistrer", p, |s, _| s.save_cell()).fill(p.tint)),
    )
}

pub(super) fn sources(s: &State, p: Palette) -> El {
    let Some(config) = s.current().and_then(|page| page.dashboard.as_ref()) else {
        return el();
    };
    let mut rows = col()
        .gap(2.0)
        .child(item("sources-all", "Toutes les bases", p, |s, _| {
            s.dashboard_scope(None)
        }));
    for source in s
        .pages
        .iter()
        .filter(|p| p.collection.is_some() && !p.archived)
    {
        let id = source.id.clone();
        let checked = config.scope.contains(&id);
        rows = rows.child(super::icons::toggle(
            format!("source-{id}"),
            source.display_title(),
            checked,
            p,
            move |s, _| s.dashboard_scope(Some(&id)),
        ));
    }
    surface("sources-menu", "dashboard-sources", 310.0, p)
        .padding(10.0)
        .gap(7.0)
        .child(
            row()
                .center_y()
                .child(label("BASES À SUIVRE", 10.0, p.faint))
                .child(el().grow(1.0))
                .child(super::icons::control(
                    "close-sources",
                    "Fermer les bases suivies",
                    &folio::icons::controls::CLOSE,
                    p,
                    |s, _| s.panel = Panel::Closed,
                )),
        )
        .child(
            col()
                .h(
                    (s.pages.iter().filter(|p| p.collection.is_some()).count() as f32 * 36.0
                        + 36.0)
                        .clamp(70.0, 220.0),
                )
                .min_h(0.0)
                .scroll(rune::ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        )
}
