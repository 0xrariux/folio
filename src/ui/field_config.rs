use super::{Palette, action, icons, label};
use crate::{
    custom_fields::ColumnForm,
    state::{Panel, State},
};
use folio::fields::{Aggregate, ColumnKind};
use rune::{El, ScrollSpec, TextStyle, col, row, text_input};

pub(super) fn config(s: &State, form: &ColumnForm, p: Palette) -> El {
    let mut body = col().key("column-config").w_full().gap(5.0);
    if form.kind == ColumnKind::Relation {
        if form.id.is_some() {
            let name = s
                .pages
                .iter()
                .find(|p| Some(&p.id) == form.config.target_base.as_ref())
                .map_or("Missing database", |p| p.display_title());
            return body
                .child(label(format!("Linked database: {name}"), 11.0, p.muted))
                .child(
                    label(
                        if form.config.reverse_of.is_some() {
                            "Reverse relation: links are calculated from the original property."
                        } else {
                            "The linked database cannot be changed after creation."
                        },
                        10.0,
                        p.faint,
                    )
                    .w_full()
                    .wrap(),
                );
        }
        body = body.child(icons::toggle(
            "relation-auto-inverse",
            "Also create the reverse relation",
            form.config.auto_inverse,
            p,
            |s, _| {
                if let Panel::Columns(f) = &mut s.panel {
                    f.config.auto_inverse = !f.config.auto_inverse;
                }
            },
        ));
        let mut options = col().gap(2.0);
        for base in s
            .pages
            .iter()
            .filter(|p| p.collection.is_some() && !p.archived)
        {
            let id = base.id.clone();
            options = options.child(icons::option(
                format!("relation-base-{id}"),
                base.display_title(),
                form.config.target_base.as_ref() == Some(&id),
                p,
                move |s, _| {
                    if let Panel::Columns(f) = &mut s.panel {
                        f.config.target_base = Some(id.clone());
                    }
                },
            ));
        }
        body = body.child(label("DATABASE TO LINK", 10.0, p.faint)).child(
            col()
                .h(90.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(options),
        );
    }
    if form.kind == ColumnKind::Rollup {
        let base = s
            .pages
            .iter()
            .find(|p| p.id == form.collection_id)
            .and_then(|p| p.collection.as_ref());
        let mut relations = row().gap(4.0);
        for field in base
            .into_iter()
            .flat_map(|c| &c.columns)
            .filter(|c| c.kind == ColumnKind::Relation)
        {
            let id = field.id.clone();
            relations = relations.child(
                action(
                    format!("rollup-relation-{id}"),
                    &field.name,
                    p,
                    move |s, tx| {
                        if let Panel::Columns(f) = &mut s.panel {
                            f.config.relation = Some(id.clone());
                        }
                        tx.scroll_into_view("column-config");
                    },
                )
                .fill(if form.config.relation.as_ref() == Some(&field.id) {
                    p.tint
                } else {
                    p.bg
                }),
            );
        }
        body = body
            .child(label("RELATION TO AGGREGATE", 10.0, p.faint))
            .child(relations);
        for pair in Aggregate::ALL.chunks(2) {
            let mut options = row().gap(4.0);
            for op in pair {
                let op = *op;
                options = options.child(
                    action(format!("aggregate-{op:?}"), op.name(), p, move |s, tx| {
                        if let Panel::Columns(f) = &mut s.panel {
                            f.config.aggregate = op;
                        }
                        tx.scroll_into_view("column-config");
                    })
                    .grow(1.0)
                    .fill(if op == form.config.aggregate {
                        p.tint
                    } else {
                        p.bg
                    }),
                );
            }
            body = body.child(options);
        }
        if form.config.aggregate == Aggregate::Sum {
            let target = base
                .and_then(|c| {
                    c.columns
                        .iter()
                        .find(|c| Some(&c.id) == form.config.relation.as_ref())
                })
                .and_then(|c| c.config.target_base.as_ref());
            let mut props = col().gap(2.0);
            for c in s
                .pages
                .iter()
                .find(|p| Some(&p.id) == target)
                .and_then(|p| p.collection.as_ref())
                .into_iter()
                .flat_map(|c| &c.columns)
                .filter(|c| {
                    matches!(
                        c.kind,
                        ColumnKind::Number | ColumnKind::Formula | ColumnKind::Rollup
                    )
                })
            {
                let id = c.id.clone();
                props = props.child(icons::option(
                    format!("rollup-property-{id}"),
                    &c.name,
                    form.config.property.as_ref() == Some(&id),
                    p,
                    move |s, tx| {
                        if let Panel::Columns(f) = &mut s.panel {
                            f.config.property = Some(id.clone());
                        }
                        tx.scroll_into_view("save-column");
                    },
                ));
            }
            body = body.child(
                col()
                    .h(75.0)
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .child(props),
            );
        }
    }
    if form.kind == ColumnKind::Formula {
        body = body.child(label("NUMERIC FORMULA", 10.0, p.faint))
            .child(text_input(&form.expression, TextStyle::new(12.0, 400.0)).key("column-expression").label("Formula").single_line().w_full().h(34.0).px(7.0).border(1.0, p.line).radius(4.0).color(p.ink).caret(p.accent, p.tint)
                .placeholder("prop(\"Effort\") * 2", p.faint).on_edit(|s: &mut State| { let Panel::Columns(f) = &mut s.panel else { unreachable!() }; &mut f.expression }))
            .child(label("+ − × /, parentheses, and prop(\"Name\"). status_done is 1 when completed, otherwise 0.", 10.0, p.faint).w_full().wrap());
    }
    body
}
