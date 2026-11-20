use crate::{
    editor::{EditorBlock, input_key},
    state::{Panel, State},
};
use folio::document::{Kind, NoteBlock};
use rune::{Env, Key, Runtime, TextEdit};

fn app() -> Runtime<State> {
    let mut s = State::demo();
    s.create(false);
    let mut rt = Runtime::new(s, crate::ui::view, (1200.0, 820.0), Env::new(false, 1.0));
    rt.frame(0.0);
    rt
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    let (x, y) = rt
        .find(key)
        .unwrap_or_else(|| panic!("Missing control: {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
}
fn type_text(rt: &mut Runtime<State>, value: &str) {
    rt.key(Key::Text(value.into()).into());
    rt.frame(0.0);
}

#[test]
fn slash_menu_turns_a_query_into_a_task_and_enter_continues_it() {
    let mut rt = app();
    let id = rt.state.blocks[0].data.id.clone();
    tap(&mut rt, &input_key(&id));
    type_text(&mut rt, "/todo");
    assert!(rt.find("slash-menu").is_some());
    assert_eq!(rt.state.slash_options(), vec![Kind::Todo]);
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    assert_eq!(rt.state.blocks[0].data.kind, Kind::Todo);
    assert_eq!(rt.state.blocks[0].edit.text, "");
    type_text(&mut rt, "Publier la première version");
    tap(&mut rt, &format!("check-{id}"));
    assert!(rt.state.current().unwrap().blocks[0].checked);
    tap(&mut rt, &input_key(&id));
    rt.key(Key::Command('a').into());
    rt.key(Key::Right.into());
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    assert_eq!(rt.state.blocks.len(), 2);
    assert_eq!(rt.state.blocks[1].data.kind, Kind::Todo);
    assert!(!rt.state.blocks[1].data.checked);
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    assert_eq!(rt.state.blocks.len(), 2);
    assert_eq!(rt.state.blocks[1].data.kind, Kind::Text);
}

#[test]
fn block_menu_reorders_duplicates_and_undo_restores_the_document() {
    let mut rt = app();
    let id = rt.state.blocks[0].data.id.clone();
    tap(&mut rt, &input_key(&id));
    type_text(&mut rt, "Une idée à garder");
    tap(&mut rt, &format!("handle-{id}"));
    tap(&mut rt, "block-copy");
    assert_eq!(rt.state.blocks.len(), 2);
    let copy = rt.state.blocks[1].data.id.clone();
    assert_ne!(id, copy);
    assert_eq!(rt.state.blocks[1].edit.text, "Une idée à garder");
    tap(&mut rt, &format!("handle-{id}"));
    tap(&mut rt, "block-down");
    assert_eq!(rt.state.blocks[1].data.id, id);
    tap(&mut rt, "undo");
    assert_eq!(rt.state.blocks[0].data.id, id);
    tap(&mut rt, "redo");
    assert_eq!(rt.state.blocks[1].data.id, id);
    tap(&mut rt, &format!("handle-{id}"));
    tap(&mut rt, "block-delete");
    assert_eq!(rt.state.blocks.len(), 1);
    assert_eq!(rt.state.blocks[0].data.id, copy);
    tap(&mut rt, "undo");
    assert_eq!(rt.state.blocks.len(), 2);
}

#[test]
fn splitting_and_merging_preserve_unicode_and_inline_marks() {
    let mut rt = app();
    let id = rt.state.blocks[0].data.id.clone();
    rt.update(|s, tx| {
        s.blocks[0].edit = TextEdit::from_html("<p><strong>été 🦀</strong> demain</p>");
        s.blocks[0].edit.set_caret("été ".len());
        s.block_changed(&id);
        s.focus_active(tx);
    });
    rt.frame(0.0);
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    assert_eq!(rt.state.blocks[0].edit.text, "été ");
    assert_eq!(rt.state.blocks[1].edit.text, "🦀 demain");
    assert!(rt.state.blocks[0].data.html.contains("<strong>"));
    assert!(rt.state.blocks[1].data.html.contains("<strong>"));
    rt.key(Key::Backspace.into());
    rt.frame(0.0);
    assert_eq!(rt.state.blocks.len(), 1);
    assert_eq!(rt.state.blocks[0].edit.text, "été 🦀 demain");
    assert!(
        rt.state.blocks[0]
            .data
            .html
            .contains("<strong>été 🦀</strong>")
    );
}

#[test]
fn templates_customization_and_page_duplication_work_through_menus() {
    let mut rt = app();
    let before = rt.state.pages.len();
    tap(&mut rt, "sidebar-more");
    tap(&mut rt, "templates");
    tap(&mut rt, "template-1");
    assert_eq!(rt.state.pages.len(), before + 1);
    assert_eq!(rt.state.current().unwrap().title, "New project");
    assert!(rt.state.blocks.iter().any(|b| b.data.kind == Kind::Todo));
    let original = rt.state.current().unwrap().clone();
    tap(&mut rt, "page-icon");
    tap(&mut rt, "icon-2");
    assert_eq!(rt.state.current().unwrap().emoji, "💡");
    tap(&mut rt, "cover-cycle");
    assert_ne!(rt.state.current().unwrap().cover, original.cover);
    tap(&mut rt, "page-menu");
    tap(&mut rt, "duplicate-page");
    assert!(rt.state.current().unwrap().title.ends_with("(copy)"));
    assert_ne!(rt.state.selected.as_deref(), Some(original.id.as_str()));
    assert!(
        rt.state
            .blocks
            .iter()
            .all(|b| original.blocks.iter().all(|old| old.id != b.data.id))
    );
}

#[test]
fn moving_pages_prevents_cycles_and_history_tracks_navigation() {
    let mut state = State::demo();
    let parent = state.pages[1].id.clone();
    let child = state.pages[2].id.clone();
    state.select(&parent);
    assert!(!state.can_move_to(&child));
    state.move_page(Some(child.clone()));
    assert_eq!(state.current().unwrap().parent_id, None);
    state.select(&child);
    state.move_page(None);
    assert_eq!(state.current().unwrap().parent_id, None);
    state.move_page(Some(parent.clone()));
    assert_eq!(
        state.current().unwrap().parent_id.as_deref(),
        Some(parent.as_str())
    );
    state.travel(false);
    assert_eq!(state.selected.as_deref(), Some(parent.as_str()));
    state.travel(true);
    assert_eq!(state.selected.as_deref(), Some(child.as_str()));
    state.collapsed.insert(parent.clone());
    assert!(!state.sidebar_pages().iter().any(|(p, _)| p.id == child));
    state.search.text = "next".into();
    assert!(state.sidebar_pages().iter().any(|(p, _)| p.id == child));
}

#[test]
fn changing_kind_keeps_content_and_dividers_do_not_hide_existing_text() {
    let mut rt = app();
    let id = rt.state.blocks[0].data.id.clone();
    tap(&mut rt, &input_key(&id));
    type_text(&mut rt, "Une idée importante");
    tap(&mut rt, "block-type");
    tap(&mut rt, "slash-Heading1");
    assert_eq!(rt.state.blocks[0].data.kind, Kind::Heading1);
    assert_eq!(rt.state.blocks[0].edit.text, "Une idée importante");
    tap(&mut rt, "block-type");
    rt.update(|_, tx| tx.scroll_into_view("slash-Divider"));
    for _ in 0..35 {
        rt.frame(1.0 / 60.0);
    }
    tap(&mut rt, "slash-Divider");
    assert_eq!(rt.state.blocks.len(), 2);
    assert_eq!(rt.state.blocks[0].edit.text, "Une idée importante");
    assert_eq!(rt.state.blocks[1].data.kind, Kind::Divider);
    assert!(matches!(rt.state.panel, Panel::Closed));
}

#[test]
fn code_enter_stays_in_the_same_block_and_removing_the_last_block_leaves_an_editor() {
    let mut rt = app();
    rt.update(|s, tx| {
        s.blocks = vec![EditorBlock::from(NoteBlock::text(Kind::Code, "let n = 1;"))];
        s.active = Some(s.blocks[0].data.id.clone());
        s.commit_blocks(true);
        s.focus_active(tx);
    });
    rt.frame(0.0);
    rt.key(Key::Enter.into());
    type_text(&mut rt, "n + 1");
    assert_eq!(rt.state.blocks.len(), 1);
    assert!(rt.state.blocks[0].edit.text.contains("\nn + 1"));
    let id = rt.state.blocks[0].data.id.clone();
    tap(&mut rt, &format!("handle-{id}"));
    tap(&mut rt, "block-delete");
    assert_eq!(rt.state.blocks.len(), 1);
    assert_eq!(rt.state.blocks[0].data.kind, Kind::Text);
    assert!(rt.state.blocks[0].edit.text.is_empty());
}

#[test]
fn page_icon_colors_and_new_symbols_work_at_the_minimum_window_size() {
    use folio::icons::{CHOICES, IconColor};
    let mut s = State::demo();
    s.create(false);
    let mut rt = Runtime::new(s, crate::ui::view, (920., 620.), Env::new(false, 1.));
    rt.frame(0.);
    tap(&mut rt, "page-icon");
    for color in IconColor::ALL {
        assert!(rt.find(&format!("icon-color-{}", color.key())).is_some());
    }
    tap(&mut rt, "icon-color-blue");
    assert_eq!(rt.state.current().unwrap().icon_color, IconColor::Blue);
    assert_eq!(rt.state.current().unwrap().emoji, "📄");
    assert!(matches!(rt.state.panel, Panel::Icons));
    let last = format!("icon-{}", CHOICES.len() - 1);
    rt.update(|_, tx| tx.scroll_into_view(&last));
    for _ in 0..40 {
        rt.frame(1. / 60.);
    }
    tap(&mut rt, &last);
    assert_eq!(
        rt.state.current().unwrap().emoji,
        CHOICES.last().unwrap().value
    );
    assert_eq!(rt.state.current().unwrap().icon_color, IconColor::Blue);
    assert!(matches!(rt.state.panel, Panel::Closed));
    tap(&mut rt, "page-icon");
    tap(&mut rt, "icon-color-default");
    assert_eq!(rt.state.current().unwrap().icon_color, IconColor::Default);
    assert_eq!(
        rt.state.current().unwrap().emoji,
        CHOICES.last().unwrap().value
    );
    rt.update(|s, _| s.panel = Panel::Closed);
    rt.frame(0.);
    rt.update(|s, _| {
        s.set_icon_color(IconColor::Rose);
        s.duplicate_page();
    });
    assert_eq!(rt.state.current().unwrap().icon_color, IconColor::Rose);
}

#[test]
fn favorites_stay_on_the_page_without_changing_sidebar_title_space() {
    for dark in [true, false] {
        let mut s = State::demo();
        s.dark = dark;
        s.select("demo-home");
        let mut rt = Runtime::new(s, crate::ui::view, (920., 620.), Env::new(dark, 1.));
        rt.frame(0.);
        let title_rect = |rt: &Runtime<State>| {
            rt.elements()
                .into_iter()
                .find(|e| e.key.as_deref() == Some("page-label-demo-home"))
                .unwrap()
                .rect
        };
        let before = title_rect(&rt);
        assert!(rt.state.current().unwrap().favorite);
        tap(&mut rt, "favorite-page");
        assert!(!rt.state.current().unwrap().favorite);
        assert_eq!(title_rect(&rt), before);
        tap(&mut rt, "favorite-page");
        assert!(rt.state.current().unwrap().favorite);
        assert_eq!(title_rect(&rt), before);
        assert!(rt.find("favorites").is_some());
    }
}

#[test]
fn sidebar_page_hover_covers_the_rounded_row_at_both_edges() {
    for dark in [true, false] {
        let mut s = State::demo();
        s.dark = dark;
        let mut rt = Runtime::new(s, crate::ui::view, (920., 620.), Env::new(dark, 1.));
        rt.frame(0.);
        for id in ["demo-home", "demo-ideas"] {
            let key = format!("page-row-{id}");
            rt.update(|_, tx| tx.scroll_into_view(&key));
            for _ in 0..35 {
                rt.frame(1. / 60.);
            }
            let row = rt
                .elements()
                .into_iter()
                .find(|e| e.key.as_deref() == Some(&key))
                .unwrap();
            let [x, y, w, h] = row.rect;
            for px in [x + 1., x + w / 2., x + w - 1.] {
                rt.pointer_move(px, y + h / 2.);
                for _ in 0..12 {
                    rt.frame(1. / 60.);
                }
                let row = rt
                    .elements()
                    .into_iter()
                    .find(|e| e.key.as_deref() == Some(&key))
                    .unwrap();
                assert!(
                    row.hover > 0.99,
                    "{id}, x={px}, dark={dark}, hover={}",
                    row.hover
                );
                assert_eq!(row.radius, 6.);
            }
        }
    }
}
