use super::{Palette, label};
use crate::{
    block_drag::DropPosition,
    editor::{EditorBlock, input_key},
    state::{Panel, State},
};
use folio::document::Kind;
use folio::icons::controls as g;
use rune::{Cursor, El, Key, Mark, TextStyle, col, el, row, text_input};
use rune_kit::{Button, ButtonVariant, ControlSize, Kit, PaletteSeed};

pub(super) fn toolbar(s: &State, p: Palette, width: f32) -> El {
    let kit = Kit::from_palette(PaletteSeed {
        bg: p.bg,
        fg: p.ink,
        accent: p.accent,
    });
    let active = s
        .blocks
        .iter()
        .find(|b| Some(&b.data.id) == s.active.as_ref());
    let mut bar = row().key("formatting").h(34.0).gap(3.0).center_y();
    bar = bar
        .child(super::icons::action(
            "block-type",
            active.map_or("Text", |b| b.data.kind.name()),
            active.map_or(&g::TEXT, |b| g::block(b.data.kind)),
            p,
            |s, _| {
                if let Some(id) = s.active.clone() {
                    s.panel = Panel::Slash {
                        id,
                        selected: 0,
                        typed: false,
                    };
                }
            },
        ))
        .child(el().w(1.0).h(16.0).fill(p.line));
    for (key, title, mark) in [
        ("bold", "Bold", Mark::Bold),
        ("italic", "Italic", Mark::Italic),
        ("underline", "Underline", Mark::Underline),
        ("inline-code", "Code", Mark::Code),
    ] {
        let selected = active.is_some_and(|b| b.edit.has_mark(&mark));
        bar = bar.child(
            Button::new(if width < 610.0 {
                match key {
                    "bold" => "B",
                    "italic" => "I",
                    "underline" => "U",
                    _ => "</>",
                }
            } else {
                title
            })
            .control_size(ControlSize::Small)
            .variant(if selected {
                ButtonVariant::Contrast
            } else {
                ButtonVariant::Ghost
            })
            .on_press_tx(move |s: &mut State, tx| {
                s.mark(mark.clone());
                s.focus_active(tx);
            })
            .build(&kit)
            .key(key)
            .label(title)
            .help(title),
        );
    }
    bar = bar.child(el().grow(1.0));
    if let Some(active) = active {
        let id = active.data.id.clone();
        let outdent = id.clone();
        bar = bar
            .child(super::icons::control(
                "list-outdent",
                "Outdent · Shift+Tab",
                &g::CHEVRON_LEFT,
                p,
                move |s, tx| {
                    s.indent_block(&outdent, true, tx);
                },
            ))
            .child(super::icons::control(
                "list-indent",
                "Indent · Tab",
                &g::CHEVRON_RIGHT,
                p,
                move |s, tx| {
                    s.indent_block(&id, false, tx);
                },
            ));
    }
    bar.child(super::icons::control(
        "undo",
        "Undo",
        &g::UNDO,
        p,
        |s, tx| s.undo_document(false, tx),
    ))
    .child(super::icons::control(
        "redo",
        "Redo",
        &g::REDO,
        p,
        |s, tx| s.undo_document(true, tx),
    ))
}

fn block(
    s: &State,
    b: &EditorBlock,
    index: usize,
    number: u32,
    progress: (usize, usize),
    after_marker: Option<&str>,
    p: Palette,
) -> El {
    let id = b.data.id.clone();
    let edit_id = id.clone();
    let change_id = id.clone();
    let key_id = id.clone();
    let focus_id = id.clone();
    let paste_id = id.clone();
    let menu_id = id.clone();
    let drag_id = id.clone();
    let plus_id = id.clone();
    let kind = b.data.kind;
    let has_children = s.block_end(index) > index + 1;
    let font = match kind {
        Kind::Heading1 | Kind::ToggleHeading1 => TextStyle::new(29.0, 650.0),
        Kind::Heading2 | Kind::ToggleHeading2 => TextStyle::new(23.0, 600.0),
        Kind::Heading3 | Kind::ToggleHeading3 => TextStyle::new(18.0, 600.0),
        Kind::Code => TextStyle {
            line_height: 23.0,
            ..TextStyle::new(13.0, 400.0).mono()
        },
        _ => TextStyle {
            line_height: 25.0,
            ..TextStyle::new(14.0, 400.0)
        },
    };
    let mut input = text_input(&b.edit, font)
        .key(input_key(&id))
        .label(format!("{} {}", kind.name(), index + 1))
        .grow(1.0)
        .min_w(0.0)
        .min_h(25.0)
        .color(
            if (kind == Kind::Todo || progress.1 > 0) && b.data.checked {
                p.faint
            } else {
                p.ink
            },
        )
        .caret(p.accent, p.tint)
        .placeholder(
            if kind == Kind::Code {
                "Write some code…"
            } else {
                "Write, or type / to insert a block…"
            },
            p.faint,
        )
        .on_link(|s: &mut State, url, tx| s.follow_reference(url, tx))
        .on_edit(move |s: &mut State| s.editor_mut(&edit_id))
        .on_input_change(move |s: &mut State, tx| {
            s.block_changed(&change_id);
            if s.active.as_deref() != Some(&change_id) {
                s.focus_active(tx);
            }
        })
        .on_input_paste(move |s: &mut State, content, tx| s.paste_blocks(&paste_id, content, tx))
        .on_input_key(move |s: &mut State, event, tx| s.block_key(&key_id, event, tx))
        .on_press(move |s: &mut State| {
            if s.active.as_deref() != Some(&focus_id) {
                s.panel = Panel::Closed;
            }
            s.active = Some(focus_id.clone());
        })
        .on_submit(move |s: &mut State, _| {
            s.editor_mut(&id).insert("\n");
            s.block_changed(&id);
        });
    if matches!(
        kind,
        Kind::Heading1
            | Kind::Heading2
            | Kind::Heading3
            | Kind::ToggleHeading1
            | Kind::ToggleHeading2
            | Kind::ToggleHeading3
    ) {
        input = input.py(6.0);
    }
    let disclosure = if kind.is_toggle() || has_children {
        let id = b.data.id.clone();
        super::icons::control(
            format!("toggle-{id}"),
            if b.data.collapsed {
                "Expand section"
            } else {
                "Collapse section"
            },
            if b.data.collapsed {
                &g::CHEVRON_RIGHT
            } else {
                &g::CHEVRON_DOWN
            },
            p,
            move |s, tx| s.toggle_block(&id, tx),
        )
        .w(22.0)
        .px(0.0)
        .expanded(!b.data.collapsed)
    } else {
        el().w(22.0)
    };
    let prefix = match kind {
        _ if kind == Kind::Todo || progress.1 > 0 => {
            let id = b.data.id.clone();
            row()
                .key(format!("check-{id}"))
                .role("checkbox")
                .label(format!("Complete: {}", b.edit.text))
                .checked(b.data.checked)
                .focusable(true)
                .size(17.0, 17.0)
                .radius(3.0)
                .border(1.0, if b.data.checked { p.accent } else { p.faint })
                .fill(if b.data.checked { p.accent } else { p.bg })
                .on_press(move |s: &mut State| s.check_task(&id))
                .center()
                .child(if b.data.checked {
                    rune::icon(&g::CHECK, 13.0).color(p.bg).a11y_hidden()
                } else {
                    el()
                })
        }
        Kind::Bullet => label(
            match b.data.indent % 3 {
                0 => "•",
                1 => "◦",
                _ => "▪",
            },
            16.0,
            p.muted,
        )
        .w(17.0),
        Kind::Numbered => label(format!("{number}."), 13.0, p.muted)
            .key(format!("list-number-{}", b.data.id))
            .w(18.0 + number.to_string().len().saturating_sub(1) as f32 * 8.0),
        Kind::Callout => el().w(25.0).child(
            rune::icon(&folio::icons::IDEA, 19.0)
                .color(p.accent)
                .a11y_hidden(),
        ),
        _ => el(),
    };
    let content = if kind == Kind::DatabaseView {
        super::embeds::database(s, &b.data, p)
    } else if matches!(kind, Kind::Image | Kind::Attachment) {
        super::embeds::media(s, &b.data, p)
    } else if kind == Kind::TableOfContents {
        super::embeds::toc(s, p)
    } else if kind == Kind::PageLink {
        let target = b
            .data
            .target_id
            .as_ref()
            .and_then(|id| s.pages.iter().find(|p| p.id == *id));
        let title = target
            .map(|p| {
                format!(
                    "{}{}",
                    p.display_title(),
                    if p.archived { " (archived)" } else { "" }
                )
            })
            .unwrap_or_else(|| "Page not found".into());
        let target_id = target.map(|p| p.id.clone());
        let block_id = b.data.id.clone();
        row()
            .key(format!("page-link-{block_id}"))
            .role("button")
            .label(&title)
            .focusable(true)
            .grow(1.0)
            .min_w(0.0)
            .h(35.0)
            .px(8.0)
            .gap(8.0)
            .center_y()
            .radius(5.0)
            .hover(p.tint)
            .on_press_tx(move |s: &mut State, tx| {
                if let Some(id) = &target_id {
                    s.select(id);
                    tx.scroll_to_y("document-scroll", 0.0);
                }
            })
            .child(
                rune::icon(&g::ARROW_UP_RIGHT, 14.0)
                    .color(p.accent)
                    .a11y_hidden(),
            )
            .child(target.map_or_else(
                || super::icons::page("📄", 16.0, p.accent),
                |target| super::icons::page_icon(target, 16.0, p.accent, p),
            ))
            .child(label(title, 14.0, p.ink).grow(1.0).min_w(0.0).truncate())
    } else if kind == Kind::Divider {
        row()
            .grow(1.0)
            .h(27.0)
            .center_y()
            .child(el().w_full().h(1.0).fill(p.line))
    } else {
        let mut content = row()
            .grow(1.0)
            .min_w(0.0)
            .gap(8.0)
            .center_y()
            .child(prefix)
            .child(input);
        if progress.1 > 0 {
            content = content.child(
                label(format!("{}/{}", progress.0, progress.1), 10.0, p.faint).label(format!(
                    "{} of {} subtasks completed",
                    progress.0, progress.1
                )),
            );
        } else if kind.is_toggle() && has_children {
            content = content.child(label(
                format!("{} items", s.block_end(index) - index - 1),
                10.0,
                p.faint,
            ));
        }
        if kind == Kind::Callout {
            content = content.padding(14.0).radius(6.0).fill(p.tint);
        }
        if kind == Kind::Code {
            content = content.padding(14.0).radius(5.0).fill(p.sidebar);
        }
        if kind == Kind::Quote {
            content = content
                .px(14.0)
                .border_sides([0.0, 0.0, 0.0, 3.0], p.accent);
        }
        content
    };
    let dragging = s.block_drag.source() == Some(b.data.id.as_str());
    let before = s.block_drag.target() == Some(&DropPosition::Before(b.data.id.clone()));
    let after = after_marker == Some(b.data.id.as_str());
    let inside = s.block_drag.target() == Some(&DropPosition::Inside(b.data.id.clone()));
    let marker = || {
        el().key("block-drop-indicator")
            .w_full()
            .h(2.0)
            .fill(p.accent)
    };
    let mut before_zone = col()
        .key(format!("drop-before-{}", b.data.id))
        .w_full()
        .grow(1.0)
        .min_h(0.0);
    let mut after_zone = col()
        .key(format!("drop-after-{}", b.data.id))
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .justify_end();
    if before {
        before_zone = before_zone.child(marker());
    }
    if after {
        after_zone = after_zone.child(marker());
    }
    let can_nest = s.can_nest_into(index);
    let inside_zone = col()
        .key(format!("drop-inside-{}", b.data.id))
        .w_full()
        .grow(0.3)
        .min_h(0.0);
    if inside {
        after_zone = after_zone.child(
            row()
                .w_full()
                .pl(74.0 + (b.data.indent as f32 + 1.0) * 18.0)
                .child(marker()),
        );
    }
    let drop_zones = col()
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .child(before_zone.grow(if can_nest { 0.35 } else { 1.0 }));
    let drop_zones = if can_nest {
        drop_zones.child(inside_zone)
    } else {
        drop_zones
    };
    let drop_zones = drop_zones.child(after_zone.grow(if can_nest { 0.35 } else { 1.0 }));
    let level = match kind {
        Kind::Heading1 | Kind::ToggleHeading1 => Some(2),
        Kind::Heading2 | Kind::ToggleHeading2 => Some(3),
        Kind::Heading3 | Kind::ToggleHeading3 => Some(4),
        _ => None,
    };
    let mut block = row().key(format!("block-{}", b.data.id));
    if let Some(level) = level {
        block = block
            .role("heading")
            .level(level)
            .label(if b.edit.text.trim().is_empty() {
                kind.name().to_string()
            } else {
                b.edit.text.clone()
            });
    }
    block
        .w_full()
        .gap(3.0)
        .py(4.0)
        .radius(4.0)
        .fill(
            if dragging || inside || s.selection.ids.contains(&b.data.id) {
                p.tint
            } else {
                rune::Color::TRANSPARENT
            },
        )
        .center_y()
        .child(
            super::icons::control(
                format!("add-after-{}", b.data.id),
                "Add a block after",
                &g::PLUS,
                p,
                move |s, tx| s.add_block(Some(&plus_id), Kind::Text, true, tx),
            )
            .w(20.0)
            .px(0.0),
        )
        .child(
            super::icons::control(
                format!("handle-{}", b.data.id),
                "Move block or open its menu",
                &g::GRIP,
                p,
                move |s, tx| {
                    s.block_handle_click(&menu_id, tx);
                },
            )
            .w(20.0)
            .px(0.0)
            .label("Move block or open its menu")
            .help("Drag between items to move · Drop on an item to nest · Click for menu")
            .cursor(if dragging {
                Cursor::Grabbing
            } else {
                Cursor::Grab
            })
            .on_drag(move |s: &mut State, d, tx| s.drag_block(&drag_id, d, tx))
            .on_key(|s: &mut State, event, tx| {
                let redo = match event.key {
                    Key::Command('z') => event.mods.shift,
                    Key::Command('y') => true,
                    _ => return false,
                };
                s.block_drag = crate::block_drag::BlockDrag::Idle;
                s.undo_document(redo, tx);
                true
            }),
        )
        .child(el().w(b.data.indent.min(6) as f32 * 18.0))
        .child(disclosure)
        .child(content)
        // Transparent zones leave editing and selection to the inputs.
        .child(drop_zones)
}

pub(super) fn view(s: &State, p: Palette) -> El {
    let mut blocks = col().key("block-editor").w_full().instant_changes();
    let numbers = folio::outline::numbers(s.blocks.iter().map(|b| &b.data));
    let ends = folio::outline::subtree_ends(s.blocks.iter().map(|b| &b.data));
    let visible = s.visible_blocks();
    let progress = folio::outline::task_progress(s.blocks.iter().map(|b| &b.data));
    let after_marker = if let Some(DropPosition::After(id)) = s.block_drag.target() {
        s.blocks
            .iter()
            .position(|b| b.data.id == *id)
            .and_then(|i| (i..ends[i]).rev().find(|&j| visible[j]))
            .map(|i| s.blocks[i].data.id.as_str())
    } else {
        None
    };
    for (index, b) in s.blocks.iter().enumerate() {
        if !visible[index] {
            continue;
        }
        blocks = blocks.child(block(
            s,
            b,
            index,
            numbers[index],
            progress[index],
            after_marker,
            p,
        ));
        if b.data.kind.is_toggle()
            && !b.data.collapsed
            && ends[index] == index + 1
            && b.data.indent < folio::outline::MAX_INDENT
        {
            let id = b.data.id.clone();
            blocks = blocks.child(row().pl(74.0 + (b.data.indent as f32 + 1.0) * 18.0).child(
                super::icons::action(
                    format!("toggle-add-{id}"),
                    "Add content to this section",
                    &g::PLUS,
                    p,
                    move |s, tx| s.add_child(&id, tx),
                ),
            ));
        }
    }
    blocks.child(
        col()
            .key("block-end-zone")
            .w_full()
            .h(38.0)
            .child(
                super::icons::action("new-block", "Add a block", &g::PLUS, p, |s, tx| {
                    s.add_block(None, Kind::Text, true, tx)
                })
                .w_full()
                .justify_start(),
            )
            .child(el().key("drop-end").absolute(0.0, 0.0).w_full().h_full()),
    )
}
