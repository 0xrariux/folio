use super::{Palette, button, label};
use crate::state::State;
use rune::{Color, El, col, el, row};

pub(super) fn page(value: &str, size: f32, color: Color) -> El {
    if let Some(choice) = folio::icons::choice(value) {
        rune::icon(choice.glyph, size).color(color).a11y_hidden()
    } else if value.starts_with("folio:") {
        rune::icon(&folio::icons::PAGE, size)
            .color(color)
            .a11y_hidden()
    } else {
        label(value, size, color).a11y_hidden()
    }
}

pub(super) fn color(value: folio::icons::IconColor, p: Palette, fallback: Color) -> Color {
    value
        .rgb(p.dark)
        .map_or(fallback, |[r, g, b]| super::rgb(r, g, b))
}

pub(super) fn page_icon(target: &folio::model::Page, size: f32, fallback: Color, p: Palette) -> El {
    page(&target.emoji, size, color(target.icon_color, p, fallback))
}

pub(super) fn page_action(
    key: impl Into<String>,
    title: &str,
    target: &folio::model::Page,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    button(key, title, p, f)
        .gap(8.0)
        .child(page_icon(target, 16.0, p.accent, p))
        .child(label(title, 12.0, p.muted).min_w(0.0).truncate())
}

pub(super) fn action(
    key: impl Into<String>,
    title: &str,
    glyph: &'static rune::Icon,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    button(key, title, p, f)
        .gap(7.0)
        .child(rune::icon(glyph, 15.0).color(p.muted).a11y_hidden())
        .child(label(title, 12.0, p.muted).min_w(0.0).truncate())
}

pub(super) fn control(
    key: impl Into<String>,
    title: &str,
    glyph: &'static rune::Icon,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    button(key, title, p, f)
        .size(30.0, 30.0)
        .px(6.0)
        .help(title)
        .child(rune::icon(glyph, 16.0).color(p.muted).a11y_hidden())
}

pub(super) fn option(
    key: impl Into<String>,
    title: &str,
    checked: bool,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    let mark = if checked {
        rune::icon(&folio::icons::controls::CHECK, 15.0)
            .color(p.accent)
            .a11y_hidden()
    } else {
        el().size(15.0, 15.0).a11y_hidden()
    };
    button(key, title, p, f)
        .label(if checked {
            format!("{title} (selected)")
        } else {
            title.to_string()
        })
        .w_full()
        .justify_start()
        .h(34.0)
        .gap(8.0)
        .fill(if checked { p.tint } else { p.bg })
        .child(mark)
        .child(label(title, 12.0, p.muted).min_w(0.0).truncate())
}

pub(super) fn checkbox_mark(checked: bool, p: Palette) -> El {
    el().size(16.0, 16.0)
        .radius(3.0)
        .border(1.0, if checked { p.accent } else { p.faint })
        .fill(if checked { p.accent } else { p.bg })
        .center()
        .a11y_hidden()
        .child(if checked {
            rune::icon(&folio::icons::controls::CHECK, 12.0)
                .color(p.bg)
                .a11y_hidden()
        } else {
            el()
        })
}

pub(super) fn toggle(
    key: impl Into<String>,
    title: &str,
    checked: bool,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    button(key, title, p, f)
        .role("checkbox")
        .checked(checked)
        .w_full()
        .justify_start()
        .h(34.0)
        .gap(8.0)
        .child(checkbox_mark(checked, p))
        .child(label(title, 12.0, p.muted).min_w(0.0).truncate())
}

/// Read-only design preview, mounted only by the demo scene.
pub(super) fn catalog(p: Palette, width: f32) -> El {
    let icons: Vec<_> = std::iter::once(("folio", "Folio", &folio::icons::FOLIO))
        .chain(
            folio::icons::CHOICES
                .iter()
                .map(|c| (c.key, c.name, c.glyph)),
        )
        .collect();
    gallery(&icons, p, width)
}

pub(super) fn controls_catalog(p: Palette, width: f32) -> El {
    let icons: Vec<_> = folio::icons::controls::ALL
        .iter()
        .map(|c| (c.key, c.name, c.glyph))
        .collect();
    gallery(&icons, p, width)
}

fn gallery(icons: &[(&str, &str, &'static rune::Icon)], p: Palette, width: f32) -> El {
    let mut gallery =
        col()
            .w_full()
            .gap(12.0)
            .child(label("Consistent icons at every size.", 13.0, p.muted));
    for group in icons.chunks(if width < 680.0 {
        3
    } else if width > 1000.0 {
        5
    } else {
        4
    }) {
        let mut cards = row().w_full().gap(12.0);
        for (key, name, glyph) in group {
            cards = cards.child(
                col()
                    .key(format!("specimen-{key}"))
                    .grow(1.0)
                    .min_w(0.0)
                    .h(104.0)
                    .padding(14.0)
                    .gap(12.0)
                    .radius(8.0)
                    .fill(p.sidebar)
                    .border(1.0, p.line)
                    .child(
                        row()
                            .gap(12.0)
                            .center_y()
                            .child(rune::icon(glyph, 36.0).color(p.accent).a11y_hidden())
                            .child(el().grow(1.0))
                            .child(rune::icon(glyph, 24.0).color(p.muted).a11y_hidden())
                            .child(rune::icon(glyph, 16.0).color(p.muted).a11y_hidden()),
                    )
                    .child(label(*name, 11.0, p.ink).w_full().truncate()),
            );
        }
        gallery = gallery.child(cards);
    }
    gallery
}
