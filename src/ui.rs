mod blocks;
mod calendar;
mod collection_panels;
mod collections;
mod custom_fields;
mod dashboard;
mod embeds;
mod field_config;
mod icons;
mod page;
mod panels;
mod saved_views;
mod selection;
mod sidebar;
mod table;
mod workspace;

use crate::state::{Phase, State};
use folio::icons::controls as g;
use rune::{Color, Cx, El, TextStyle, col, el, row, text};

#[derive(Clone, Copy)]
pub(super) struct Palette {
    pub bg: Color,
    pub sidebar: Color,
    pub ink: Color,
    pub muted: Color,
    pub faint: Color,
    pub line: Color,
    pub accent: Color,
    pub tint: Color,
}
fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: rune::color::srgb_decode(r as f32 / 255.0),
        g: rune::color::srgb_decode(g as f32 / 255.0),
        b: rune::color::srgb_decode(b as f32 / 255.0),
        a: 1.0,
    }
}
fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: rgb(29, 31, 30),
            sidebar: rgb(24, 26, 25),
            ink: rgb(232, 234, 228),
            muted: rgb(163, 170, 160),
            faint: rgb(130, 138, 126),
            line: rgb(53, 58, 53),
            accent: rgb(153, 189, 146),
            tint: rgb(42, 57, 44),
        }
    } else {
        Palette {
            bg: rgb(253, 252, 249),
            sidebar: rgb(244, 244, 238),
            ink: rgb(43, 49, 42),
            muted: rgb(105, 114, 101),
            faint: rgb(131, 139, 126),
            line: rgb(226, 229, 219),
            accent: rgb(65, 101, 62),
            tint: rgb(230, 237, 222),
        }
    }
}
pub(super) fn label(value: impl Into<String>, size: f32, color: Color) -> El {
    text(value).font(TextStyle::new(size, 400.0)).color(color)
}
pub(super) fn button(
    key: impl Into<String>,
    title: &str,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    row()
        .key(key)
        .role("button")
        .label(title)
        .focusable(true)
        .h(30.0)
        .px(9.0)
        .center()
        .radius(5.0)
        .hover(p.ink.alpha(0.065))
        .press(p.ink.alpha(0.1))
        .on_press_tx(f)
}
pub(super) fn action(
    key: impl Into<String>,
    title: &str,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    button(key, title, p, f).child(label(title, 12.0, p.muted))
}
pub fn view(state: &State, cx: &mut Cx<State>) -> El {
    let p = palette(state.dark);
    let page_width = cx
        .layout_size("document-scroll")
        .map_or(882.0, |size| (size[0] - 70.0).max(0.0));
    let status = match &state.phase {
        Phase::Loading => "Ouverture du carnet…",
        Phase::Failed(_) => "Impossible d’ouvrir le carnet",
        Phase::Ready if state.demo => "Démonstration · modifications temporaires",
        Phase::Ready if state.pending.values().any(|p| p.error.is_some()) => {
            "Échec de l’enregistrement · réessayer"
        }
        Phase::Ready if !state.pending.is_empty() => "Enregistrement…",
        Phase::Ready => "Tout est enregistré",
    };
    let error = match &state.phase {
        Phase::Failed(message) => Some(message.as_str()),
        _ => state.pending.values().find_map(|p| p.error.as_deref()),
    };
    let mut main = col()
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .child(page::view(state, p, page_width));
    if let Some(error) = error {
        main = main.child(
            label(format!("Erreur : {error}"), 12.0, p.ink)
                .w_full()
                .padding(12.0)
                .fill(p.tint),
        );
    }
    if let Some(message) = &state.export_message {
        let mut notice = row()
            .key("export-notice")
            .w_full()
            .padding(10.0)
            .gap(8.0)
            .center_y()
            .fill(p.tint)
            .child(label(message, 11.0, p.ink).grow(1.0).min_w(0.0).wrap());
        if !state.export_busy {
            notice = notice.child(icons::control(
                "dismiss-export",
                "Fermer la notification",
                &g::CLOSE,
                p,
                |s, _| s.export_message = None,
            ));
        }
        main = main.child(notice);
    }
    main = main.child(
        row()
            .h(39.0)
            .px(20.0)
            .gap(4.0)
            .center_y()
            .border_sides([1.0, 0.0, 0.0, 0.0], p.line)
            .child(
                rune::icon(
                    if matches!(state.phase, Phase::Failed(_))
                        || state.pending.values().any(|p| p.error.is_some())
                    {
                        &g::ALERT
                    } else if matches!(state.phase, Phase::Loading) || !state.pending.is_empty() {
                        &g::CLOCK
                    } else {
                        &g::CHECK
                    },
                    14.0,
                )
                .color(p.accent)
                .a11y_hidden(),
            )
            .child(action("save-status", status, p, |s, _| s.retry()))
            .child(el().grow(1.0))
            .child(label(
                if state
                    .current()
                    .is_some_and(|p| p.collection.is_some() || p.dashboard.is_some())
                {
                    "Fiches et suivi"
                } else {
                    "/  Insérer un bloc"
                },
                10.0,
                p.faint,
            ))
            .child(icons::action(
                "theme",
                if state.dark {
                    "Mode clair"
                } else {
                    "Mode sombre"
                },
                if state.dark { &g::SUN } else { &g::MOON },
                p,
                |s, _| s.switch_theme(),
            )),
    );
    let mut root = col()
        .key("folio")
        .w_full()
        .h_full()
        .fill(p.bg)
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .child(sidebar::view(state, p))
                .child(
                    el().key("sidebar-resize")
                        .w(4.0)
                        .h_full()
                        .hover(p.line)
                        .on_drag(|s: &mut State, d, _| {
                            if d.phase == rune::Phase::Down {
                                d.data = s.workspace.preferences.sidebar_width as f32;
                            }
                            s.workspace.preferences.sidebar_width = if d.cancelled() {
                                d.data as u16
                            } else {
                                (d.data + d.dx).clamp(180.0, 360.0) as u16
                            };
                            if d.phase == rune::Phase::Up {
                                s.save_preferences();
                            }
                        }),
                )
                .child(main),
        )
        .children(panels::view(state, p));
    if let Some([x, y]) = state.block_drag.point()
        && let Some(block) = state
            .blocks
            .iter()
            .find(|b| Some(b.data.id.as_str()) == state.block_drag.source())
    {
        let [width, height] = cx.layout_size("folio").unwrap_or([1200.0, 820.0]);
        let x = (x + 14.0).clamp(8.0, (width - 298.0).max(8.0));
        let y = (y + 12.0).clamp(8.0, (height - 52.0).max(8.0));
        let title: String = block
            .edit
            .text
            .replace('\n', " ")
            .chars()
            .take(80)
            .collect();
        let count = state
            .blocks
            .iter()
            .position(|b| b.data.id == block.data.id)
            .map_or(1, |i| state.block_end(i) - i);
        root = root.child(
            row()
                .key("block-drag-preview")
                .absolute(x, y)
                .w(290.0)
                .h(44.0)
                .px(12.0)
                .gap(9.0)
                .center_y()
                .radius(6.0)
                .fill(p.bg)
                .border(1.0, p.accent)
                .shadow(5.0, 16.0, rune::Color::BLACK.alpha(0.14))
                .cursor(rune::Cursor::Grabbing)
                .child(
                    rune::icon(g::block(block.data.kind), 17.0)
                        .color(p.accent)
                        .a11y_hidden(),
                )
                .child(
                    label(
                        if title.is_empty() {
                            block.data.kind.name()
                        } else {
                            &title
                        },
                        12.0,
                        p.ink,
                    )
                    .grow(1.0)
                    .min_w(0.0)
                    .truncate(),
                )
                .child(if count > 1 {
                    label(format!("{count} blocs"), 10.0, p.faint)
                } else {
                    el()
                }),
        );
    }
    if let Some(drag) = &state.workspace.entity_drag
        && drag.active
    {
        let title = state
            .pages
            .iter()
            .find(|p| p.id == drag.id)
            .map_or("Déplacer", |p| p.display_title());
        root = root.child(
            label(title, 12.0, p.ink)
                .max_w(250.0)
                .truncate()
                .padding(12.0)
                .radius(6.0)
                .fill(p.bg)
                .border(1.0, p.accent)
                .absolute(drag.point[0] + 14.0, drag.point[1] + 12.0),
        );
    }
    if state.workspace.busy {
        root = root.child(workspace::busy(p));
    }
    root
}

pub(super) fn entity_scroll(view: El, s: &State, key: &str) -> El {
    if let Some(session) = &s.workspace.entity_drag
        && session.scroll_key == key
        && !session.released
    {
        let feed = session.scroll.clone();
        let key = key.to_string();
        view.on_layout_tx(move |s: &mut State, rect, tx| {
            if let Some(session) = &mut s.workspace.entity_drag
                && session.scroll_key == key
            {
                session.viewport = Some(rect);
            }
            tx.live_only();
        })
        .on_scroll_offset(move |position| feed.set(position))
    } else {
        view
    }
}

pub(super) fn entity_target(s: &State, key: &str) -> bool {
    s.workspace
        .entity_drag
        .as_ref()
        .is_some_and(|d| d.active && d.target.as_deref() == Some(key))
}
