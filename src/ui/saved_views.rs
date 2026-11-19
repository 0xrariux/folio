use super::{Palette, action, label, panels::surface};
use crate::{
    saved_views::ViewForm,
    state::{Panel, State},
};
use rune::{El, Key, ScrollSpec, TextStyle, col, el, row, text_input};

pub(super) fn form(s: &State, form: &ViewForm, p: Palette) -> El {
    let Some(settings) = s.current().and_then(|page| page.collection.as_ref()) else {
        return el();
    };
    let mut views = col().gap(3.0);
    for view in &settings.views {
        let id = view.id.clone();
        views = views.child(super::icons::option(
            format!("saved-view-{id}"),
            &view.name,
            settings.active_view.as_ref() == Some(&id),
            p,
            move |s, _| s.select_view(&id),
        ));
    }
    if settings.views.is_empty() {
        views = views.child(
            label("Save your first view below.", 11.0, p.faint)
                .w_full()
                .wrap()
                .padding(6.0),
        );
    }
    let mut menu = col()
        .gap(9.0)
        .child(
            row()
                .center_y()
                .child(label("VIEWS IN THIS DATABASE", 10.0, p.faint))
                .child(el().grow(1.0))
                .child(super::icons::control(
                    "close-views",
                    "Close views",
                    &folio::icons::controls::CLOSE,
                    p,
                    |s, _| s.panel = Panel::Closed,
                )),
        )
        .child(
            col()
                .h((settings.views.len() as f32 * 37.0).clamp(40.0, 180.0))
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(views),
        );
    if let Some(active) = settings.active() {
        let id = active.id.clone();
        let mut actions = row()
            .gap(5.0)
            .child(action("rename-view", "Rename", p, |s, tx| {
                s.open_views(true);
                tx.focus("view-name", false);
            }));
        if settings.views.len() > 1 {
            actions = actions.child(action("remove-view", "Remove this view", p, move |s, _| {
                s.remove_view(&id)
            }));
        }
        menu = menu.child(actions);
    }
    menu = menu.child(el().h(1.0).fill(p.line))
        .child(label(if form.rename { "RENAME VIEW" } else { "SAVE CURRENT SETTINGS" }, 10.0, p.faint))
        .child(text_input(&form.name, TextStyle::new(13.0, 400.0))
            .key("view-name").label("View name").single_line().w_full()
            .h(34.0).px(8.0).radius(5.0).border(1.0, p.line).color(p.ink).caret(p.accent, p.tint)
            .placeholder("E.g. This week, Priorities…", p.faint)
            .on_edit(|s: &mut State| s.view_name_edit())
            .on_submit(|s: &mut State, _| s.save_view())
            .on_input_key(|s: &mut State, event, _| {
                if event.key == Key::Escape { s.panel = Panel::Closed; true } else { false }
            }))
        .child(label("The layout, filters, and sort order are saved. Changes to the active view are saved automatically.", 11.0, p.faint).w_full().wrap());
    if let Some(error) = &form.error {
        menu = menu.child(label(error, 11.0, p.ink).w_full().wrap());
    }
    menu = menu.child(
        row()
            .gap(6.0)
            .child(action("new-view", "New view", p, |s, tx| {
                s.open_views(false);
                tx.focus("view-name", false);
            }))
            .child(el().grow(1.0))
            .child(
                action(
                    "save-view",
                    if form.rename { "Rename" } else { "Create view" },
                    p,
                    |s, _| s.save_view(),
                )
                .fill(p.tint),
            ),
    );
    surface("views-form", "collection-views", 340.0, p)
        .padding(12.0)
        .min_h(0.0)
        .child(
            menu.key("views-scroll")
                .shrink()
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                }),
        )
}
