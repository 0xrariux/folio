use crate::{
    block_drag::BlockDrag,
    editor::{EditorBlock, input_key},
    state::{Panel, State},
};
use folio::document::{Kind, NoteBlock};
use rune::{Env, Key, Runtime};

fn app(count: usize, size: (f32, f32)) -> Runtime<State> {
    let mut s = State::demo();
    s.create(false);
    s.title = rune::TextEdit::new("Une liste à organiser");
    s.edit();
    s.blocks = (0..count)
        .map(|i| {
            let mut block = NoteBlock::text(
                if i == 1 { Kind::Todo } else { Kind::Numbered },
                &format!("Étape {i} 🦀"),
            );
            block.id = format!("item-{i}");
            block.checked = i == 1;
            if i == 1 {
                block.html = "<p><strong>été 🦀</strong> à conserver</p>".into();
            }
            EditorBlock::from(block)
        })
        .collect();
    s.active = Some("item-0".into());
    s.commit_blocks(true);
    s.undo.clear();
    let mut rt = Runtime::new(s, crate::ui::view, size, Env::new(false, 1.0));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.0);
    rt
}
fn ids(rt: &Runtime<State>) -> Vec<String> {
    rt.state.blocks.iter().map(|b| b.data.id.clone()).collect()
}
fn reveal(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..40 {
        rt.frame(1.0 / 60.0);
    }
}
fn point(rt: &Runtime<State>, key: &str, fraction: f32) -> (f32, f32) {
    let ([x, y, w, h], _) = rt.presented(key).unwrap_or_else(|| panic!("Missing {key}"));
    (x + w * 0.5, y + h * fraction)
}
fn press(rt: &mut Runtime<State>, id: &str) -> (f32, f32) {
    let key = format!("handle-{id}");
    reveal(rt, &key);
    let (x, y) = point(rt, &key, 0.5);
    rt.pointer_down(x, y);
    rt.frame(1.0 / 60.0);
    (x, y)
}
fn drop_at(rt: &mut Runtime<State>, source: &str, target: &str, after: bool) {
    press(rt, source);
    let (x, y) = point(
        rt,
        &format!("block-{target}"),
        if after { 0.75 } else { 0.25 },
    );
    rt.pointer_move(x, y);
    rt.frame(1.0 / 60.0);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    reveal(rt, key);
    let (x, y) = point(rt, key, 0.5);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
}

#[test]
fn mouse_drop_reorders_once_preserves_content_and_supports_undo_redo() {
    let mut rt = app(4, (1200.0, 820.0));
    rt.update(|s, _| {
        s.blocks[1].data.indent = 1;
        s.commit_blocks(true);
        s.undo.clear();
    });
    rt.frame(0.0);
    let original = rt.state.current().unwrap().blocks.clone();
    press(&mut rt, "item-1");
    let (x, y) = point(&rt, "block-item-3", 0.75);
    rt.pointer_move(x, y);
    rt.frame(1.0 / 60.0);
    assert_eq!(
        rt.state.current().unwrap().blocks,
        original,
        "hover does not write data"
    );
    assert_eq!(rt.state.undo.len(), 0);
    assert!(rt.find("block-drag-preview").is_some());
    assert_eq!(rt.state.active.as_deref(), Some("item-1"));
    assert!(rt.find("block-drop-indicator").is_some());
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    assert_eq!(ids(&rt), ["item-0", "item-2", "item-3", "item-1"]);
    let mut moved = original[1].clone();
    moved.indent = 0; // Dropping beside a root makes it a root, preserving its content.
    assert_eq!(rt.state.current().unwrap().blocks[3], moved);
    assert_eq!(rt.state.undo.len(), 1);
    assert!(rt.find("block-drag-preview").is_none());
    assert!(matches!(rt.state.panel, Panel::Closed));
    rt.key(Key::Command('z').into());
    rt.frame(0.0);
    assert_eq!(rt.state.current().unwrap().blocks, original);
    tap(&mut rt, "redo");
    assert_eq!(ids(&rt), ["item-0", "item-2", "item-3", "item-1"]);
}

#[test]
fn before_after_and_adjacent_no_op_slots_work_in_both_directions() {
    let mut rt = app(4, (1200.0, 820.0));
    drop_at(&mut rt, "item-3", "item-0", false);
    assert_eq!(ids(&rt), ["item-3", "item-0", "item-1", "item-2"]);
    drop_at(&mut rt, "item-0", "item-1", true);
    assert_eq!(ids(&rt), ["item-3", "item-1", "item-0", "item-2"]);
    let steps = rt.state.undo.len();
    drop_at(&mut rt, "item-0", "item-2", false);
    assert_eq!(rt.state.undo.len(), steps);
    drop_at(&mut rt, "item-1", "item-3", true);
    assert_eq!(rt.state.undo.len(), steps);
    press(&mut rt, "item-3");
    let (x, y) = point(&rt, "block-end-zone", 0.5);
    rt.pointer_move(x, y);
    rt.frame(0.0);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert_eq!(ids(&rt), ["item-1", "item-0", "item-2", "item-3"]);
}

#[test]
fn escape_focus_loss_and_outside_drop_do_not_edit_the_document() {
    for cancel in 0..3 {
        let mut rt = app(4, (1200.0, 820.0));
        let before = ids(&rt);
        press(&mut rt, "item-0");
        let (x, y) = point(&rt, "block-item-3", 0.75);
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        match cancel {
            0 => rt.key(Key::Escape.into()),
            1 => {
                rt.window_focus(true);
                rt.window_focus(false);
            }
            _ => rt.pointer_move(20.0, 200.0),
        }
        rt.frame(1.0 / 60.0);
        rt.pointer_up(
            if cancel == 2 { 20.0 } else { x },
            if cancel == 2 { 200.0 } else { y },
        );
        rt.frame(0.0);
        assert_eq!(ids(&rt), before);
        assert!(rt.state.undo.is_empty());
        assert!(rt.find("block-drag-preview").is_none());
        assert!(rt.find("block-drop-indicator").is_none());
    }
}

#[test]
fn clicks_tiny_moves_and_keyboard_activation_keep_the_block_menu() {
    let mut rt = app(4, (1200.0, 820.0));
    let (x, y) = press(&mut rt, "item-0");
    rt.pointer_move(x + 2.0, y + 2.0);
    rt.pointer_up(x + 2.0, y + 2.0);
    rt.frame(0.0);
    assert!(matches!(&rt.state.panel, Panel::Block(id) if id == "item-0"));
    rt.key(Key::Escape.into());
    rt.frame(0.0);
    let (x, y) = press(&mut rt, "item-0");
    rt.pointer_move(x + 25.0, y + 20.0);
    rt.frame(0.0);
    rt.pointer_move(x, y);
    rt.frame(0.0);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert!(
        matches!(rt.state.panel, Panel::Closed),
        "a drag returning to its handle is not a click"
    );
    assert!(rt.state.undo.is_empty());
    drop_at(&mut rt, "item-0", "item-3", true);
    assert!(
        rt.elements()
            .iter()
            .any(|e| e.key.as_deref() == Some("handle-item-0") && e.focused),
        "focus: {:?}",
        rt.elements()
            .iter()
            .filter(|e| e.focused)
            .map(|e| (&e.key, &e.role))
            .collect::<Vec<_>>()
    );
    rt.key(Key::Text(" ".into()).into());
    rt.frame(0.0);
    assert!(matches!(&rt.state.panel, Panel::Block(id) if id == "item-0"));
}

#[test]
fn transparent_drop_zones_leave_text_editing_and_checkboxes_accessible() {
    let mut rt = app(4, (1200.0, 820.0));
    tap(&mut rt, &input_key("item-0"));
    rt.key(Key::Command('a').into());
    rt.key(Key::Text("Une nouvelle idée".into()).into());
    rt.frame(0.0);
    assert_eq!(rt.state.blocks[0].edit.text, "Une nouvelle idée");
    tap(&mut rt, "check-item-1");
    assert!(!rt.state.blocks[1].data.checked);
}

#[test]
fn a_fast_gesture_works_without_an_intermediate_frame_and_navigation_cancels_it() {
    let mut rt = app(4, (1200.0, 820.0));
    let (x, y) = point(&rt, "handle-item-0", 0.5);
    let (dx, dy) = point(&rt, "block-item-3", 0.75);
    rt.pointer_down(x, y);
    rt.pointer_up(dx, dy);
    rt.frame(0.0);
    assert_eq!(ids(&rt), ["item-1", "item-2", "item-3", "item-0"]);
    press(&mut rt, "item-1");
    rt.pointer_move(dx, dy);
    rt.frame(0.0);
    rt.update(|s, _| s.select("demo-home"));
    rt.frame(0.0);
    let before = rt.state.current().unwrap().clone();
    rt.pointer_up(dx, dy);
    rt.frame(0.0);
    assert_eq!(rt.state.current().unwrap(), &before);
    assert!(matches!(rt.state.block_drag, BlockDrag::Idle));
}

#[test]
fn mixed_block_heights_use_the_visible_upper_and_lower_halves() {
    let mut rt = app(4, (1200.0, 820.0));
    rt.update(|s, _| {
        s.blocks[0].data.kind = Kind::Bullet;
        s.blocks[1].data.kind = Kind::Code;
        s.blocks[1].edit = rune::TextEdit::new("ligne 1\nligne 2\nligne 3\nligne 4");
        s.blocks[2].data.kind = Kind::Divider;
        s.blocks[2].edit = rune::TextEdit::new("");
        s.commit_blocks(true);
        s.undo.clear();
    });
    rt.frame(0.0);
    let original = rt.state.current().unwrap().blocks.clone();
    drop_at(&mut rt, "item-3", "item-1", false);
    assert_eq!(ids(&rt), ["item-0", "item-3", "item-1", "item-2"]);
    drop_at(&mut rt, "item-0", "item-1", true);
    assert_eq!(ids(&rt), ["item-3", "item-1", "item-0", "item-2"]);
    assert_eq!(rt.state.current().unwrap().blocks[1], original[1]);
}

#[test]
fn holding_near_the_viewport_edge_scrolls_and_releasing_stops_it() {
    let mut rt = app(40, (920.0, 620.0));
    press(&mut rt, "item-0");
    rt.frame(1.0 / 60.0);
    let viewport = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("document-scroll"))
        .unwrap()
        .rect;
    let (x, y) = (viewport[0] + 100.0, viewport[1] + viewport[3] - 10.0);
    let before = point(&rt, "block-item-10", 0.5).1;
    rt.pointer_move(x, y);
    for _ in 0..45 {
        rt.frame(1.0 / 60.0);
    }
    let after = point(&rt, "block-item-10", 0.5).1;
    assert!(
        after < before - 100.0,
        "held edge scrolls: {before} -> {after}"
    );
    assert!(rt.state.undo.is_empty());
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let settled = point(&rt, "block-item-10", 0.5).1;
    for _ in 0..30 {
        rt.frame(1.0 / 60.0);
    }
    assert_eq!(point(&rt, "block-item-10", 0.5).1, settled);
}

#[test]
fn dropped_order_is_saved_in_turso_and_restored_after_closing() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("drag.db");
    let mut rt = app(4, (1200.0, 820.0));
    rt.state.demo = false;
    rt.state.worker = Some(crate::worker::Worker::start(
        path.clone(),
        std::sync::Arc::new(|| {}),
    ));
    drop_at(&mut rt, "item-0", "item-3", true);
    let saved = rt.state.current().unwrap().clone();
    assert!(!rt.state.pending.is_empty());
    drop(rt);
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let store = folio::storage::Store::open(&path).await.unwrap();
            assert_eq!(
                store
                    .load()
                    .await
                    .unwrap()
                    .iter()
                    .find(|p| p.id == saved.id),
                Some(&saved)
            );
        });
}

#[test]
fn mouse_drag_moves_open_and_closed_subtrees_and_refuses_a_drop_into_itself() {
    for collapsed in [false, true] {
        let mut rt = app(4, (1200.0, 1000.0));
        rt.update(|s, _| {
            s.blocks[0].data.kind = Kind::ToggleHeading2;
            s.blocks[0].data.collapsed = collapsed;
            s.blocks[1].data.indent = 1;
            s.blocks[2].data.indent = 2;
            s.commit_blocks(true);
            s.undo.clear();
        });
        rt.frame(0.0);
        let before = rt.state.current().unwrap().blocks.clone();
        if !collapsed {
            drop_at(&mut rt, "item-0", "item-2", true);
            assert_eq!(rt.state.current().unwrap().blocks, before);
            assert!(rt.state.undo.is_empty());
        }
        drop_at(&mut rt, "item-0", "item-3", true);
        assert_eq!(ids(&rt), ["item-3", "item-0", "item-1", "item-2"]);
        assert_eq!(&rt.state.current().unwrap().blocks[1..], &before[..3]);
        assert_eq!(rt.state.undo.len(), 1);
        rt.key(Key::Command('z').into());
        rt.frame(0.0);
        assert_eq!(rt.state.current().unwrap().blocks, before);
    }
}
