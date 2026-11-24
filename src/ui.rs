mod blocks;
mod calendar;
mod collection_panels;
mod collections;
mod custom_fields;
mod dashboard;
mod embeds;
mod field_config;
mod icons;
mod library;
mod management;
mod overview;
mod page;
mod panels;
mod saved_views;
mod selection;
mod settings;
mod sidebar;
mod table;
#[cfg(test)]
mod theme_tests;
mod workspace;

use crate::state::{Panel, Phase, State};
use folio::icons::controls as g;
use folio::workspace::{DarkPalette, LightPalette};
use rune::{Color, Cx, El, TextStyle, col, el, row, text};

#[derive(Clone, Copy)]
pub(super) struct Palette {
    pub dark: bool,
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
fn palette(dark: bool, dark_palette: DarkPalette) -> Palette {
    if dark && dark_palette == DarkPalette::Aurum {
        Palette {
            dark,
            bg: rgb(27, 20, 18),
            sidebar: rgb(20, 15, 13),
            ink: rgb(244, 229, 205),
            muted: rgb(190, 166, 148),
            faint: rgb(157, 132, 115),
            line: rgb(68, 46, 37),
            accent: rgb(232, 158, 91),
            tint: rgb(67, 36, 26),
        }
    } else if dark && dark_palette == DarkPalette::Nord {
        Palette {
            dark,
            bg: rgb(46, 52, 64),
            sidebar: rgb(40, 46, 57),
            ink: rgb(236, 239, 244),
            muted: rgb(216, 222, 233),
            faint: rgb(156, 169, 191),
            line: rgb(67, 76, 94),
            accent: rgb(136, 192, 208),
            tint: rgb(59, 66, 82),
        }
    } else if dark && dark_palette == DarkPalette::Mocha {
        Palette {
            dark,
            bg: rgb(30, 30, 46),
            sidebar: rgb(24, 24, 37),
            ink: rgb(205, 214, 244),
            muted: rgb(186, 194, 222),
            faint: rgb(147, 153, 178),
            line: rgb(69, 71, 90),
            accent: rgb(203, 166, 247),
            tint: rgb(49, 50, 68),
        }
    } else if dark && dark_palette == DarkPalette::RosePine {
        Palette {
            dark,
            bg: rgb(25, 23, 36),
            sidebar: rgb(22, 20, 31),
            ink: rgb(224, 222, 244),
            muted: rgb(181, 176, 204),
            faint: rgb(144, 140, 170),
            line: rgb(64, 61, 82),
            accent: rgb(235, 188, 186),
            tint: rgb(38, 35, 58),
        }
    } else if dark && dark_palette == DarkPalette::Gruvbox {
        Palette {
            dark,
            bg: rgb(40, 40, 40),
            sidebar: rgb(27, 27, 27),
            ink: rgb(221, 199, 161),
            muted: rgb(189, 174, 147),
            faint: rgb(168, 153, 132),
            line: rgb(80, 73, 69),
            accent: rgb(216, 166, 87),
            tint: rgb(51, 62, 52),
        }
    } else if dark {
        Palette {
            dark,
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
            dark,
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
fn light_palette(choice: LightPalette) -> Palette {
    if choice == LightPalette::Original {
        return palette(false, DarkPalette::Aurum);
    }
    Palette {
        dark: false,
        bg: rgb(255, 251, 238),
        sidebar: rgb(248, 240, 211),
        ink: rgb(64, 53, 38),
        muted: rgb(117, 96, 68),
        faint: rgb(141, 121, 83),
        line: rgb(231, 217, 177),
        accent: rgb(133, 90, 31),
        tint: rgb(243, 226, 169),
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
    let p = if state.dark {
        palette(true, state.workspace.preferences.dark_palette)
    } else {
        light_palette(state.workspace.preferences.light_palette)
    };
    if state.settings.locked && !matches!(state.panel, crate::state::Panel::Settings) {
        return settings::locked(state, p);
    }
    let page_width = cx
        .layout_size("document-scroll")
        .map_or(882.0, |size| (size[0] - 70.0).max(0.0));
    let status = match &state.phase {
        Phase::Loading => "Opening notebook…",
        Phase::Failed(_) => "Cannot open notebook",
        Phase::Ready if state.demo => "Demo · temporary changes",
        Phase::Ready if state.pending.values().any(|p| p.error.is_some()) => "Save failed · retry",
        Phase::Ready if !state.pending.is_empty() => "Saving…",
        Phase::Ready => "All changes saved",
    };
    let error = match &state.phase {
        Phase::Failed(message) => Some(message.as_str()),
        _ => state.pending.values().find_map(|p| p.error.as_deref()),
    };
    let mut main = col()
        .key("main")
        .role("main")
        .label("Notebook content")
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .child(if matches!(state.panel, crate::state::Panel::Settings) {
            settings::view(
                state,
                p,
                cx.layout_size("main").map_or(952., |size| size[0]),
            )
        } else if matches!(
            state.panel,
            Panel::Library | Panel::ModuleReview | Panel::ModuleConfirm
        ) {
            library::view(state, p)
        } else if state.overview.open {
            overview::view(state, p)
        } else {
            page::view(state, p, page_width)
        });
    if let Some(error) = error {
        main = main.child(
            label(format!("Error: {error}"), 12.0, p.ink)
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
                "Close notification",
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
                if matches!(state.panel, Panel::Settings) {
                    "Workspace preferences"
                } else if matches!(
                    state.panel,
                    Panel::Library | Panel::ModuleReview | Panel::ModuleConfirm
                ) {
                    "Your module library"
                } else if state.overview.open {
                    "Your notebook at a glance"
                } else if state
                    .current()
                    .is_some_and(|p| p.collection.is_some() || p.dashboard.is_some())
                {
                    "Records and tracking"
                } else {
                    "/  Insert a block"
                },
                10.0,
                p.faint,
            ))
            .child(icons::action(
                "theme",
                if state.dark {
                    "Light mode"
                } else {
                    "Dark mode"
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
                .child(
                    if matches!(
                        state.block_drag.target(),
                        Some(crate::block_drag::DropPosition::Inside(_))
                    ) {
                        label("Nest inside", 10.0, p.accent)
                    } else if count > 1 {
                        label(format!("{count} blocks"), 10.0, p.faint)
                    } else {
                        el()
                    },
                ),
        );
    }
    if let Some(drag) = &state.workspace.entity_drag
        && drag.active
    {
        let title = if drag.kind == crate::entity_drag::Kind::Shortcut {
            drag.id
                .strip_prefix("shortcut:")
                .and_then(folio::workspace::SidebarShortcut::from_key)
                .map_or("Move", |shortcut| shortcut.name())
        } else {
            state
                .pages
                .iter()
                .find(|p| p.id == drag.id)
                .map_or("Move", |p| p.display_title())
        };
        root = root.child(
            label(title, 12.0, p.ink)
                .key("entity-drag-preview")
                .cursor(rune::Cursor::Grabbing)
                .a11y_hidden()
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
