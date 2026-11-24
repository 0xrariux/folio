use super::{Palette, label};
use rune::{El, Icon, TextStyle, col, row};

pub(super) fn surface(p: Palette) -> El {
    col()
        .w_full()
        .min_w(0.)
        .padding(22.)
        .gap(16.)
        .radius(12.)
        .border(1., p.line)
        .fill(p.bg)
}

pub(super) fn badge(title: impl Into<String>, accent: bool, p: Palette) -> El {
    label(title, 10., if accent { p.accent } else { p.muted })
        .font(TextStyle::new(10., 550.))
        .px(9.)
        .py(5.)
        .radius(12.)
        .self_start()
        .fill(if accent { p.tint } else { p.sidebar })
}

pub(super) fn heading(title: &str, description: &str, glyph: &'static Icon, p: Palette) -> El {
    row()
        .w_full()
        .min_w(0.)
        .gap(12.)
        .center_y()
        .child(
            col()
                .size(36., 36.)
                .radius(10.)
                .fill(p.tint)
                .center()
                .child(rune::icon(glyph, 18.).color(p.accent).a11y_hidden()),
        )
        .child(
            col()
                .grow(1.)
                .min_w(0.)
                .gap(4.)
                .child(
                    label(title, 16., p.ink)
                        .font(TextStyle::new(16., 600.))
                        .role("heading")
                        .level(2),
                )
                .child(label(description, 12., p.muted).w_full().wrap()),
        )
}

pub(super) fn header(
    title: &str,
    description: &str,
    glyph: &'static Icon,
    controls: El,
    p: Palette,
) -> El {
    row()
        .key(format!("{}-header", title.to_lowercase()))
        .w_full()
        .min_w(0.)
        .padding(28.)
        .gap(16.)
        .center_y()
        .border_sides([0., 0., 1., 0.], p.line)
        .child(
            col()
                .size(44., 44.)
                .radius(13.)
                .fill(p.tint)
                .center()
                .child(rune::icon(glyph, 23.).color(p.accent).a11y_hidden()),
        )
        .child(
            col()
                .grow(1.)
                .min_w(0.)
                .gap(6.)
                .child(
                    label(title, 30., p.ink)
                        .font(TextStyle::new(30., 650.))
                        .role("heading")
                        .level(1),
                )
                .child(label(description, 12., p.muted).w_full().wrap()),
        )
        .child(controls)
}

pub(super) fn page(key: &str, header: El, content: El, p: Palette) -> El {
    col()
        .key(key)
        .w_full()
        .min_w(0.)
        .grow(1.)
        .min_h(0.)
        .child(header)
        .child(
            col()
                .key(format!("{key}-scroll"))
                .w_full()
                .grow(1.)
                .min_w(0.)
                .min_h(0.)
                .fill(p.sidebar.alpha(0.35))
                .scroll(Default::default())
                .child(content.w_full().min_w(0.).padding(28.)),
        )
}
