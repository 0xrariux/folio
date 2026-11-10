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
            active.map_or("Texte", |b| b.data.kind.name()),
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
        ("bold", "Gras", Mark::Bold),
        ("italic", "Italique", Mark::Italic),
        ("underline", "Souligné", Mark::Underline),
        ("inline-code", "Code", Mark::Code),
    ] {
        let selected = active.is_some_and(|b| b.edit.has_mark(&mark));
        bar = bar.child(
            Button::new(if width < 610.0 {
                match key {
                    "bold" => "G",
                    "italic" => "I",
                    "underline" => "S",
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
                "Désindenter · Maj+Tab",
                &g::CHEVRON_LEFT,
                p,
                move |s, tx| {
                    s.indent_block(&outdent, true, tx);
                },
            ))
            .child(super::icons::control(
                "list-indent",
                "Indenter · Tab",
                &g::CHEVRON_RIGHT,
                p,
                move |s, tx| {
                    s.indent_block(&id, false, tx);
                },
            ));
    }
    bar.child(super::icons::control(
        "undo",
        "Annuler",
        &g::UNDO,
        p,
        |s, tx| s.undo_document(false, tx),
    ))
    .child(super::icons::control(
        "redo",
        "Rétablir",
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
        .color(if kind == Kind::Todo && b.data.checked {
            p.faint
        } else {
            p.ink
        })
        .caret(p.accent, p.tint)
        .placeholder(
            if kind == Kind::Code {
                "Écrivez du code…"
            } else {
                "Écrivez, ou tapez / pour insérer un bloc…"
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
    let prefix = match kind {
        kind if kind.is_toggle() => {
            let id = b.data.id.clone();
            super::icons::control(
                format!("toggle-{id}"),
                if b.data.collapsed {
                    "Déplier le tiroir"
                } else {
                    "Replier le tiroir"
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
        }
        Kind::Todo => {
            let id = b.data.id.clone();
            row()
                .key(format!("check-{id}"))
                .role("checkbox")
                .label(format!("Terminer : {}", b.edit.text))
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
                    if p.archived { " (archivée)" } else { "" }
                )
            })
            .unwrap_or_else(|| "Page introuvable".into());
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
            .child(super::icons::page(
                target.map_or("📄", |p| p.emoji.as_str()),
                16.0,
                p.accent,
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
        if kind.is_toggle() {
            let children = &s.blocks[index + 1..s.block_end(index)];
            let tasks: Vec<_> = children
                .iter()
                .filter(|b| b.data.kind == Kind::Todo)
                .collect();
            let hint = if !tasks.is_empty() {
                format!(
                    "{}/{} tâches",
                    tasks.iter().filter(|b| b.data.checked).count(),
                    tasks.len()
                )
            } else if !children.is_empty() {
                format!("{} éléments", children.len())
            } else {
                String::new()
            };
            if !hint.is_empty() {
                content = content.child(label(hint, 10.0, p.faint));
            }
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
    row()
        .key(format!("block-{}", b.data.id))
        .w_full()
        .gap(3.0)
        .py(4.0)
        .radius(4.0)
        .fill(if dragging || s.selection.ids.contains(&b.data.id) {
            p.tint
        } else {
            rune::Color::TRANSPARENT
        })
        .center_y()
        .child(
            super::icons::control(
                format!("add-after-{}", b.data.id),
                "Ajouter un bloc après",
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
                "Déplacer le bloc ou ouvrir son menu",
                &g::GRIP,
                p,
                move |s, tx| {
                    s.block_handle_click(&menu_id, tx);
                },
            )
            .w(20.0)
            .px(0.0)
            .label("Déplacer le bloc ou ouvrir son menu")
            .help("Glisser pour déplacer · Cliquer pour le menu · Échap pour annuler")
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
        .child(content)
        // No handlers: these transparent halves leave editing and selection to the inputs.
        .child(
            col()
                .absolute(0.0, 0.0)
                .w_full()
                .h_full()
                .child(before_zone)
                .child(after_zone),
        )
}

pub(super) fn view(s: &State, p: Palette) -> El {
    let mut blocks = col().key("block-editor").w_full().instant_changes();
    let numbers = folio::outline::numbers(s.blocks.iter().map(|b| &b.data));
    let ends = folio::outline::subtree_ends(s.blocks.iter().map(|b| &b.data));
    let visible = s.visible_blocks();
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
        blocks = blocks.child(block(s, b, index, numbers[index], after_marker, p));
        if b.data.kind.is_toggle()
            && !b.data.collapsed
            && ends[index] == index + 1
            && b.data.indent < folio::outline::MAX_INDENT
        {
            let id = b.data.id.clone();
            blocks = blocks.child(row().pl(52.0 + (b.data.indent as f32 + 1.0) * 18.0).child(
                super::icons::action(
                    format!("toggle-add-{id}"),
                    "Ajouter du contenu au tiroir",
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
                super::icons::action("new-block", "Ajouter un bloc", &g::PLUS, p, |s, tx| {
                    s.add_block(None, Kind::Text, true, tx)
                })
                .w_full()
                .justify_start(),
            )
            .child(el().key("drop-end").absolute(0.0, 0.0).w_full().h_full()),
    )
}
