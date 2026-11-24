use crate::{editor::EditorBlock, state::State};
use folio::document::{Kind, NoteBlock};
use rune::{Env, Key, Mods, Runtime};

fn task(id: &str, checked: bool, indent: u8) -> NoteBlock {
    let mut b = NoteBlock::text(Kind::Todo, id);
    b.id = id.into();
    b.checked = checked;
    b.indent = indent;
    b
}

fn paragraph(id: &str) -> NoteBlock {
    let mut b = NoteBlock::text(Kind::Text, id);
    b.id = id.into();
    b
}

fn app(blocks: Vec<NoteBlock>) -> Runtime<State> {
    let mut s = State::demo();
    s.create(false);
    s.blocks = blocks.into_iter().map(EditorBlock::from).collect();
    s.active = s.blocks.first().map(|b| b.data.id.clone());
    s.commit_blocks(true);
    s.undo.clear();
    let mut rt = Runtime::new(s, crate::ui::view, (1200., 1000.), Env::new(false, 1.));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.);
    rt
}

fn settle(rt: &mut Runtime<State>) {
    for _ in 0..40 {
        rt.frame(1. / 60.);
    }
}

fn tap(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    settle(rt);
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("missing {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.);
}

fn y(rt: &Runtime<State>, key: &str) -> f32 {
    rt.presented(key)
        .unwrap_or_else(|| panic!("missing {key}"))
        .0[1]
}

fn assert_order(rt: &Runtime<State>, keys: &[&str]) {
    for pair in keys.windows(2) {
        assert!(y(rt, pair[0]) < y(rt, pair[1]), "{pair:?}");
    }
}

#[test]
fn completed_entities_render_below_the_note_and_return_to_their_original_place() {
    let mut rt = app(vec![
        task("done-a", true, 0),
        paragraph("context"),
        task("open", false, 0),
        task("done-b", true, 0),
        paragraph("footer"),
    ]);
    let original = rt.state.current().unwrap().blocks.clone();
    assert_order(
        &rt,
        &[
            "block-context",
            "block-open",
            "block-footer",
            "block-end-zone",
            "completed-tasks-heading",
            "block-done-a",
            "block-done-b",
        ],
    );
    assert_eq!(rt.state.current().unwrap().blocks, original);
    tap(&mut rt, "check-done-a");
    assert_order(
        &rt,
        &[
            "block-done-a",
            "block-context",
            "block-open",
            "block-done-b",
        ],
    );
    tap(&mut rt, "check-done-b");
    assert!(rt.find("completed-tasks-heading").is_none());
    assert_order(&rt, &["block-open", "block-done-b", "block-footer"]);
    assert_eq!(rt.state.blocks.len(), original.len());
}

#[test]
fn nested_tasks_stay_with_their_parent_until_the_whole_entity_is_complete() {
    let mut parent = task("parent", false, 0);
    parent.html = "<p><strong>Project 🦀</strong></p>".into();
    let mut rt = app(vec![
        parent,
        task("first", true, 1),
        task("last", false, 1),
        task("other", false, 0),
    ]);
    assert!(rt.find("completed-tasks-heading").is_none());
    tap(&mut rt, "check-last");
    assert_order(
        &rt,
        &[
            "block-other",
            "completed-tasks-heading",
            "block-parent",
            "block-first",
            "block-last",
        ],
    );
    assert!(rt.state.blocks[0].data.checked);
    assert!(rt.state.blocks[0].data.html.contains("<strong>"));
    tap(&mut rt, "check-first");
    assert!(rt.find("completed-tasks-heading").is_none());
    assert_order(
        &rt,
        &["block-parent", "block-first", "block-last", "block-other"],
    );
    assert!(!rt.state.blocks[0].data.checked);
    assert!(rt.state.blocks[2].data.checked);
}

#[test]
fn folded_groups_keep_their_children_and_completion_survives_undo_redo_and_navigation() {
    let mut parent = task("parent", false, 0);
    parent.collapsed = true;
    let mut rt = app(vec![parent, task("child", false, 1), paragraph("note")]);
    let original = rt.state.current().unwrap().blocks.clone();
    let page = rt.state.selected.clone().unwrap();
    tap(&mut rt, "check-parent");
    assert_order(
        &rt,
        &["block-note", "completed-tasks-heading", "block-parent"],
    );
    assert!(rt.find("block-child").is_none());
    assert_eq!(rt.state.undo.len(), 1);
    let completed = rt.state.current().unwrap().blocks.clone();
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.);
    assert_eq!(rt.state.current().unwrap().blocks, original);
    assert!(rt.find("completed-tasks-heading").is_none());
    rt.update(|s, tx| s.undo_document(true, tx));
    rt.frame(0.);
    assert_eq!(rt.state.current().unwrap().blocks, completed);
    rt.update(|s, _| {
        s.select("demo-home");
        s.select(&page);
    });
    rt.frame(0.);
    assert_eq!(rt.state.current().unwrap().blocks, completed);
    assert!(rt.find("completed-tasks-heading").is_some());
    tap(&mut rt, "toggle-parent");
    tap(&mut rt, "check-child");
    assert!(rt.find("completed-tasks-heading").is_none());
    assert!(!rt.state.blocks[0].data.checked);
    assert!(!rt.state.blocks[1].data.checked);
}

#[test]
fn keyboard_navigation_and_shift_selection_follow_the_displayed_order() {
    let mut rt = app(vec![
        task("done-a", true, 0),
        task("a", false, 0),
        task("done-b", true, 0),
        task("b", false, 0),
    ]);
    rt.update(|s, tx| {
        s.editor_mut("b").set_caret(1);
        s.active = Some("b".into());
        tx.focus("block-input-b", false);
    });
    rt.frame(0.);
    rt.key(Key::Down.into());
    rt.frame(0.);
    assert_eq!(rt.state.active.as_deref(), Some("done-a"));
    rt.update(|s, _| {
        s.editor_mut("done-a").set_caret(0);
    });
    rt.frame(0.);
    rt.key(Key::Up.into());
    rt.frame(0.);
    assert_eq!(rt.state.active.as_deref(), Some("b"));
    rt.update(|s, _| {
        s.select_block(
            "a",
            Mods {
                cmd: true,
                ..Default::default()
            },
        );
        s.select_block(
            "b",
            Mods {
                shift: true,
                ..Default::default()
            },
        );
    });
    assert_eq!(
        rt.state
            .selection
            .ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
}

#[test]
fn backspace_merges_adjacent_active_paragraphs_without_swallowing_completed_tasks() {
    let mut rt = app(vec![
        paragraph("first"),
        task("done", true, 0),
        paragraph("last"),
    ]);
    rt.update(|s, tx| {
        s.editor_mut("last").set_caret(0);
        assert!(s.merge_previous("last", tx));
    });
    rt.frame(0.);
    assert_eq!(rt.state.blocks.len(), 2);
    assert_eq!(rt.state.blocks[0].edit.text, "firstlast");
    assert_eq!(rt.state.blocks[1].data.id, "done");
    assert!(rt.state.blocks[1].data.checked);
    assert!(rt.find("completed-tasks-heading").is_some());
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.);
    assert_eq!(rt.state.blocks.len(), 3);
    assert_eq!(rt.state.blocks[0].edit.text, "first");
}

#[test]
fn adding_a_subtask_reopens_a_completed_group_and_new_tasks_stay_above_completed() {
    let mut rt = app(vec![task("parent", true, 0), task("child", true, 1)]);
    tap(&mut rt, "new-task");
    let new = rt.state.active.clone().unwrap();
    assert_order(
        &rt,
        &[
            &format!("block-{new}"),
            "completed-tasks-heading",
            "block-parent",
        ],
    );
    assert!(!rt.state.blocks.last().unwrap().data.checked);
    rt.update(|s, tx| s.add_child("parent", tx));
    rt.frame(0.);
    assert!(rt.find("completed-tasks-heading").is_none());
    assert!(!rt.state.blocks[0].data.checked);
    assert_eq!(rt.state.blocks[2].data.indent, 1);
    assert!(!rt.state.blocks[2].data.checked);
}

#[test]
fn moving_and_indenting_active_tasks_skip_completed_neighbours() {
    let mut rt = app(vec![
        task("a", false, 0),
        task("done", true, 0),
        task("b", false, 0),
    ]);
    rt.update(|s, _| s.move_block("b", false));
    rt.frame(0.);
    assert_order(
        &rt,
        &[
            "block-b",
            "block-a",
            "completed-tasks-heading",
            "block-done",
        ],
    );
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.);
    rt.update(|s, tx| {
        assert!(s.indent_block("b", false, tx));
    });
    rt.frame(0.);
    assert_eq!(
        rt.state
            .blocks
            .iter()
            .map(|b| (b.data.id.as_str(), b.data.indent))
            .collect::<Vec<_>>(),
        [("a", 0), ("b", 1), ("done", 0)]
    );
    assert!(rt.state.blocks[2].data.checked);
    assert_order(
        &rt,
        &[
            "block-a",
            "block-b",
            "completed-tasks-heading",
            "block-done",
        ],
    );
}

#[test]
fn dragging_to_the_active_list_end_keeps_completed_entities_separate() {
    let mut rt = app(vec![
        task("a", false, 0),
        task("b", false, 0),
        task("done", true, 0),
    ]);
    let (x, y) = rt.find("handle-a").unwrap();
    let (dx, dy) = rt.find("drop-end").unwrap();
    rt.pointer_down(x, y);
    rt.frame(0.);
    rt.pointer_move(dx, dy);
    rt.frame(0.);
    rt.pointer_up(dx, dy);
    rt.frame(0.);
    assert_eq!(
        rt.state
            .blocks
            .iter()
            .map(|b| b.data.id.as_str())
            .collect::<Vec<_>>(),
        ["b", "a", "done"]
    );
    assert_order(
        &rt,
        &[
            "block-b",
            "block-a",
            "completed-tasks-heading",
            "block-done",
        ],
    );
    assert!(rt.state.blocks[2].data.checked);
}
