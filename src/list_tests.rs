use crate::{
    editor::{EditorBlock, input_key},
    state::{Panel, State},
};
use folio::document::{Kind, NoteBlock};
use rune::{ClipboardContent, Env, Key, KeyEvent, Mods, Runtime, TextEdit};

fn block(id: &str, kind: Kind, indent: u8, text: &str) -> NoteBlock {
    let mut b = NoteBlock::text(kind, text);
    b.id = id.into();
    b.indent = indent;
    b
}
fn app(blocks: Vec<NoteBlock>) -> Runtime<State> {
    app_at(blocks, (1200.0, 1000.0))
}
fn app_at(blocks: Vec<NoteBlock>, size: (f32, f32)) -> Runtime<State> {
    let mut s = State::demo();
    s.create(false);
    s.blocks = blocks.into_iter().map(EditorBlock::from).collect();
    s.active = Some(s.blocks[0].data.id.clone());
    s.commit_blocks(true);
    s.undo.clear();
    let mut rt = Runtime::new(s, crate::ui::view, size, Env::new(false, 1.0));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.0);
    rt
}
fn nested() -> Vec<NoteBlock> {
    let mut blocks = vec![
        block("parent", Kind::ToggleHeading2, 0, "Un projet 🦀"),
        block("first", Kind::Numbered, 1, "Préparer"),
        block("nested", Kind::Bullet, 2, "Un détail"),
        block("second", Kind::Numbered, 1, "Partager"),
        block("inner", Kind::Toggle, 1, "Les tâches"),
        block("task-a", Kind::Todo, 2, "Valider"),
        block("task-b", Kind::Todo, 2, "Publier"),
        block("outside", Kind::Text, 0, "Hors du tiroir"),
        block("other-task", Kind::Todo, 0, "Une autre liste"),
    ];
    blocks[1].html = "<p><strong>Préparer 🦀</strong></p>".into();
    blocks[4].collapsed = true;
    blocks
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..35 {
        rt.frame(1.0 / 60.0);
    }
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("Missing {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
}
fn key(rt: &mut Runtime<State>, key: Key, shift: bool, cmd: bool) {
    rt.key(KeyEvent {
        key,
        mods: Mods {
            shift,
            cmd,
            ..Default::default()
        },
    });
    rt.frame(0.0);
}
fn ids(rt: &Runtime<State>) -> Vec<&str> {
    rt.state.blocks.iter().map(|b| b.data.id.as_str()).collect()
}

#[test]
fn toggle_commands_create_children_and_keyboard_exits_an_empty_nested_list() {
    let mut rt = app(vec![block("root", Kind::Text, 0, "")]);
    tap(&mut rt, &input_key("root"));
    key(&mut rt, Key::Text("/tiroir".into()), false, false);
    tap(&mut rt, "slash-Toggle");
    key(&mut rt, Key::Text("Mes idées".into()), false, false);
    key(&mut rt, Key::Enter, false, false);
    assert_eq!(rt.state.blocks[0].data.kind, Kind::Toggle);
    assert_eq!(rt.state.blocks[1].data.indent, 1);
    key(&mut rt, Key::Text("[] ".into()), false, false);
    assert_eq!(rt.state.blocks[1].data.kind, Kind::Todo);
    key(&mut rt, Key::Text("Une tâche".into()), false, false);
    key(&mut rt, Key::Enter, false, false);
    assert_eq!(rt.state.blocks[2].data.kind, Kind::Todo);
    assert_eq!(rt.state.blocks[2].data.indent, 1);
    key(&mut rt, Key::Enter, false, false);
    assert_eq!(rt.state.blocks[2].data.indent, 0);
    key(&mut rt, Key::Enter, false, false);
    assert_eq!(rt.state.blocks[2].data.kind, Kind::Text);
    assert!(!rt.state.blocks[2].data.checked);
}

#[test]
fn nested_toggles_hide_inputs_keep_their_state_and_support_undo() {
    let mut rt = app(nested());
    assert!(rt.find(&input_key("first")).is_some());
    assert!(rt.find(&input_key("task-a")).is_none());
    tap(&mut rt, "toggle-parent");
    assert!(rt.find(&input_key("first")).is_none());
    assert!(rt.find("toggle-inner").is_none());
    assert!(rt.find(&input_key("outside")).is_some());
    let toggle = rt
        .elements()
        .into_iter()
        .find(|e| e.key.as_deref() == Some("toggle-parent"))
        .unwrap();
    assert_eq!(toggle.role, "button");
    assert_eq!(rt.state.current().unwrap().blocks.len(), 9);
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.0);
    assert!(rt.find("toggle-inner").is_some());
    assert!(rt.state.blocks[4].data.collapsed);
    tap(&mut rt, "toggle-inner");
    tap(&mut rt, &input_key("task-a"));
    rt.update(|s, tx| s.fold_all(true, tx));
    rt.frame(0.0);
    assert_eq!(rt.state.active.as_deref(), Some("parent"));
    assert!(rt.find(&input_key("task-a")).is_none());
    rt.update(|s, tx| s.fold_all(false, tx));
    rt.frame(0.0);
    assert!(rt.find(&input_key("task-a")).is_some());
}

#[test]
fn groups_duplicate_delete_and_move_without_losing_hidden_children_or_marks() {
    let mut rt = app(nested());
    tap(&mut rt, "toggle-parent");
    let before = rt.state.current().unwrap().blocks.clone();
    rt.update(|s, tx| s.duplicate_block("parent", tx));
    rt.frame(0.0);
    assert_eq!(rt.state.blocks.len(), 16);
    let copy = rt.state.blocks[7].data.id.clone();
    for i in 0..7 {
        assert_ne!(rt.state.blocks[i].data.id, rt.state.blocks[i + 7].data.id);
        assert_eq!(
            rt.state.blocks[i].data.html,
            rt.state.blocks[i + 7].data.html
        );
        assert_eq!(
            rt.state.blocks[i].data.collapsed,
            rt.state.blocks[i + 7].data.collapsed
        );
    }
    rt.update(|s, tx| s.remove_block(&copy, tx));
    rt.frame(0.0);
    assert_eq!(rt.state.current().unwrap().blocks, before);
    rt.update(|s, _| s.move_block("parent", true));
    rt.frame(0.0);
    assert_eq!(ids(&rt)[0], "outside");
    assert_eq!(
        &ids(&rt)[1..8],
        [
            "parent", "first", "nested", "second", "inner", "task-a", "task-b"
        ]
    );
    rt.update(|s, tx| s.remove_block("parent", tx));
    rt.frame(0.0);
    assert_eq!(ids(&rt), ["outside", "other-task"]);
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.0);
    assert!(rt.state.blocks[1].data.collapsed);
    assert!(rt.state.blocks[2].data.html.contains("<strong>"));
}

#[test]
fn indentation_moves_the_whole_subtree_and_outdent_does_not_adopt_following_siblings() {
    let mut rt = app(vec![
        block("a", Kind::Bullet, 0, "A"),
        block("b", Kind::Toggle, 0, "B"),
        block("child", Kind::Text, 1, "C"),
        block("d", Kind::Bullet, 0, "D"),
    ]);
    rt.update(|s, tx| {
        assert!(s.indent_block("b", false, tx));
    });
    rt.frame(0.0);
    assert_eq!(
        rt.state
            .blocks
            .iter()
            .map(|b| b.data.indent)
            .collect::<Vec<_>>(),
        [0, 1, 2, 0]
    );
    rt.update(|s, tx| {
        assert!(s.indent_block("child", true, tx));
    });
    rt.frame(0.0);
    assert_eq!(
        rt.state
            .blocks
            .iter()
            .map(|b| b.data.indent)
            .collect::<Vec<_>>(),
        [0, 1, 1, 0]
    );
    rt.update(|s, tx| {
        assert!(s.indent_block("b", true, tx));
    });
    rt.frame(0.0);
    assert_eq!(ids(&rt), ["a", "child", "b", "d"]);
    assert_eq!(rt.state.blocks[1].data.indent, 1);
    assert_eq!(rt.state.blocks[2].data.indent, 0);
    rt.update(|s, tx| {
        assert!(!s.indent_block("a", false, tx));
        s.blocks[2].data.indent = 6;
        assert!(!s.indent_block("b", false, tx));
    });
}

#[test]
fn numbered_lists_continue_across_children_restart_and_reset_at_new_parents() {
    let mut rt = app(nested());
    let n = || folio::outline::numbers(nested().iter());
    assert_eq!(n()[1], 1);
    assert_eq!(n()[3], 2);
    rt.update(|s, _| s.restart_numbering("second", true));
    assert_eq!(
        folio::outline::numbers(rt.state.blocks.iter().map(|b| &b.data))[3],
        1
    );
    rt.update(|s, _| s.restart_numbering("second", false));
    assert_eq!(
        folio::outline::numbers(rt.state.blocks.iter().map(|b| &b.data))[3],
        2
    );
    let mut blocks = vec![
        block("a", Kind::Numbered, 0, "A"),
        block("b", Kind::Numbered, 1, "B"),
        block("c", Kind::Numbered, 0, "C"),
        block("d", Kind::Numbered, 1, "D"),
    ];
    blocks[0].number_start = Some(5);
    assert_eq!(folio::outline::numbers(&blocks), [5, 1, 6, 1]);
}

#[test]
fn paste_creates_real_list_items_preserves_rich_selection_and_undo_is_atomic() {
    let mut rt = app(vec![block("root", Kind::Text, 0, "avant après")]);
    tap(&mut rt, &input_key("root"));
    rt.update(|s, _| s.blocks[0].edit.set_caret(6));
    rt.frame(0.0);
    rt.paste_content(
        ClipboardContent::new("été\ndemain")
            .with_html("<p><strong>été 🦀</strong></p><p><em>demain</em></p>"),
    );
    rt.frame(0.0);
    assert_eq!(rt.state.blocks.len(), 2);
    assert_eq!(rt.state.blocks[0].edit.text, "avant été 🦀");
    assert_eq!(rt.state.blocks[1].edit.text, "demainaprès");
    assert!(
        rt.state.blocks[0]
            .data
            .html
            .contains("<strong>été 🦀</strong>")
    );
    assert!(rt.state.blocks[1].data.html.contains("<em>demain</em>"));
    assert_eq!(
        rt.state.active.as_deref(),
        Some(rt.state.blocks[1].data.id.as_str())
    );
    assert_eq!(rt.state.undo.len(), 1);
    key(&mut rt, Key::Command('z'), false, true);
    assert_eq!(rt.state.blocks.len(), 1);
    assert_eq!(rt.state.blocks[0].edit.text, "avant après");
    rt.update(|s, _| {
        s.blocks[0].edit = TextEdit::new("").rich();
        s.commit_blocks(true);
    });
    rt.frame(0.0);
    tap(&mut rt, &input_key("root"));
    rt.paste("3. Première\n    - Sous-liste\n4. Deuxième\n- [x] Validée\n");
    rt.frame(0.0);
    assert_eq!(rt.state.blocks.len(), 4);
    assert_eq!(rt.state.blocks[0].data.kind, Kind::Numbered);
    assert_eq!(rt.state.blocks[0].data.number_start, Some(3));
    assert_eq!(rt.state.blocks[1].data.indent, 1);
    assert_eq!(rt.state.blocks[3].data.kind, Kind::Todo);
    assert!(rt.state.blocks[3].data.checked);
    assert!(
        rt.state.blocks[2].data.number_start.is_none(),
        "a pasted sequence continues automatically"
    );
    rt.paste_content(ClipboardContent::new(" important").with_html("<strong> important</strong>"));
    rt.frame(0.0);
    assert_eq!(rt.state.blocks.len(), 4);
    assert!(
        rt.state.blocks[3]
            .data
            .html
            .contains("<strong>important</strong>"),
        "{}",
        rt.state.blocks[3].data.html
    );
    assert_eq!(rt.state.blocks[3].edit.text, "Validée important");
}

#[test]
fn soft_breaks_code_and_inline_shortcuts_keep_expected_editing_behavior() {
    let mut rt = app(vec![block("root", Kind::Text, 0, "")]);
    tap(&mut rt, &input_key("root"));
    key(&mut rt, Key::Text("- ".into()), false, false);
    assert_eq!(rt.state.blocks[0].data.kind, Kind::Bullet);
    key(&mut rt, Key::Text("Première".into()), false, false);
    key(&mut rt, Key::Enter, true, false);
    key(&mut rt, Key::Text("seconde".into()), false, false);
    assert_eq!(rt.state.blocks.len(), 1);
    assert_eq!(rt.state.blocks[0].edit.text, "Première\nseconde");
    rt.update(|s, tx| s.set_list_kind("root", Kind::Code, tx));
    rt.frame(0.0);
    rt.paste("\n- du code\n1. encore");
    rt.frame(0.0);
    assert_eq!(rt.state.blocks.len(), 1);
    assert!(
        rt.state.blocks[0]
            .edit
            .text
            .contains("- du code\n1. encore")
    );
    rt.update(|s, tx| {
        s.set_list_kind("root", Kind::Text, tx);
        s.blocks[0].edit = TextEdit::new("").rich();
        s.commit_blocks(true);
        s.focus_active(tx);
    });
    rt.frame(0.0);
    key(&mut rt, Key::Text("--- ".into()), false, false);
    assert_eq!(rt.state.blocks[0].data.kind, Kind::Divider);
    assert_eq!(rt.state.blocks[1].data.kind, Kind::Text);
    key(
        &mut rt,
        Key::Text("Après le séparateur".into()),
        false,
        false,
    );
    assert_eq!(rt.state.blocks[1].edit.text, "Après le séparateur");
}

#[test]
fn task_bulk_actions_are_scoped_to_the_list_and_undo_together() {
    let mut rt = app(nested());
    rt.update(|s, _| s.check_list("task-a", true));
    rt.frame(0.0);
    assert!(rt.state.blocks[5].data.checked && rt.state.blocks[6].data.checked);
    assert!(!rt.state.blocks[8].data.checked);
    assert_eq!(rt.state.undo.len(), 1);
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.0);
    assert!(!rt.state.blocks[5].data.checked && !rt.state.blocks[6].data.checked);
    assert!(matches!(rt.state.panel, Panel::Closed));
    assert!(rt.state.dark, "dark is the initial theme");
}

#[test]
fn group_menu_actions_remain_reachable_in_a_small_window() {
    let mut state = State::demo();
    state.demo_lists();
    let mut rt = Runtime::new(state, crate::ui::view, (920.0, 620.0), Env::new(false, 1.0));
    rt.on_key = Some(crate::on_key);
    rt.frame(0.0);
    tap(&mut rt, "handle-demo-list-7");
    tap(&mut rt, "list-check-all");
    assert!(rt.state.blocks[8..11].iter().all(|b| b.data.checked));
    assert!(matches!(rt.state.panel, Panel::Closed));
    tap(&mut rt, "handle-demo-list-7");
    tap(&mut rt, "block-delete");
    assert!(!rt.state.blocks.iter().any(|b| b.data.id == "demo-list-8"));
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.0);
    assert!(rt.state.blocks.iter().any(|b| b.data.id == "demo-list-8"));
}

#[test]
fn backspace_outdents_before_converting_or_merging_and_task_shortcut_works() {
    let mut rt = app(vec![
        block("parent", Kind::Toggle, 0, "Parent"),
        block("child", Kind::Todo, 1, "Une tâche 🦀"),
        block("sibling", Kind::Bullet, 1, "À garder"),
    ]);
    tap(&mut rt, &input_key("child"));
    rt.update(|s, _| s.editor_mut("child").set_caret(0));
    rt.frame(0.0);
    key(&mut rt, Key::Backspace, false, false);
    assert_eq!(ids(&rt), ["parent", "sibling", "child"]);
    assert_eq!(rt.state.blocks[2].data.indent, 0);
    key(&mut rt, Key::Enter, false, true);
    assert!(rt.state.blocks[2].data.checked);
    key(&mut rt, Key::Backspace, false, false);
    assert_eq!(rt.state.blocks[2].data.kind, Kind::Text);
    assert_eq!(rt.state.blocks[2].edit.text, "Une tâche 🦀");
    key(&mut rt, Key::Backspace, false, false);
    assert_eq!(
        rt.state.blocks.len(),
        3,
        "a merge cannot swallow the preceding subtree"
    );
}

#[test]
fn boundary_arrow_keys_move_between_visible_blocks_without_opening_closed_toggles() {
    let mut blocks = nested();
    blocks[0].collapsed = true;
    let mut rt = app(blocks);
    tap(&mut rt, &input_key("parent"));
    rt.update(|s, _| {
        let b = s.editor_mut("parent");
        b.set_caret(b.text.len());
    });
    rt.frame(0.0);
    let before = rt.state.current().unwrap().blocks.clone();
    key(&mut rt, Key::Down, false, false);
    assert_eq!(rt.state.active.as_deref(), Some("outside"));
    key(&mut rt, Key::Up, false, false);
    assert_eq!(rt.state.active.as_deref(), Some("parent"));
    assert_eq!(rt.state.current().unwrap().blocks, before);
}

#[test]
fn opening_an_existing_note_compacts_empty_paragraphs_with_undo_and_keeps_structured_content() {
    let mut rt = app(vec![block("start", Kind::Text, 0, "Start")]);
    let mut original = vec![block("start", Kind::Text, 0, "Start")];
    for i in 0..15 {
        original.push(block(&format!("gap-{i}"), Kind::Text, 0, ""));
    }
    original.extend([
        block("heading", Kind::Heading2, 0, ""),
        block("task", Kind::Todo, 0, ""),
        block("code", Kind::Code, 0, "\n\n\n"),
        block("parent", Kind::Text, 0, ""),
        block("child", Kind::Bullet, 1, "Keep this child"),
        block("end", Kind::Text, 0, "End"),
    ]);
    rt.update(|s, _| {
        let id = s.selected.clone().unwrap();
        let page = s.pages.iter_mut().find(|p| p.id == id).unwrap();
        page.blocks = original.clone();
        page.sync_document();
        s.select("demo-home");
        s.select(&id);
    });
    rt.frame(0.0);
    assert_eq!(
        ids(&rt),
        [
            "start", "gap-0", "heading", "task", "code", "parent", "child", "end"
        ]
    );
    assert_eq!(rt.state.blocks[4].edit.text, "\n\n\n");
    assert_eq!(rt.state.undo.len(), 1);
    rt.update(|s, tx| s.undo_document(false, tx));
    assert_eq!(rt.state.current().unwrap().blocks, original);
    rt.update(|s, tx| s.undo_document(true, tx));
    rt.frame(0.0);
    tap(&mut rt, &input_key("gap-0"));
    for _ in 0..15 {
        key(&mut rt, Key::Enter, false, false);
    }
    assert_eq!(rt.state.blocks.len(), 8);
    assert_eq!(rt.state.active.as_deref(), Some("gap-0"));
}

#[test]
fn paste_compacts_blank_runs_keeps_the_active_editor_and_undo_restores_the_original() {
    let mut rt = app(vec![block("root", Kind::Text, 0, "")]);
    tap(&mut rt, &input_key("root"));
    rt.paste("First\n\n\n\nSecond\n\n\n\n");
    rt.frame(0.0);
    assert_eq!(
        rt.state
            .blocks
            .iter()
            .map(|b| b.edit.text.as_str())
            .collect::<Vec<_>>(),
        ["First", "", "Second", ""]
    );
    assert_eq!(
        rt.state.active.as_ref(),
        rt.state.blocks.last().map(|b| &b.data.id)
    );
    key(&mut rt, Key::Text("Continue here".into()), false, false);
    assert_eq!(rt.state.blocks.last().unwrap().edit.text, "Continue here");
    rt.update(|s, tx| {
        s.undo_document(false, tx);
        s.undo_document(false, tx);
    });
    assert_eq!(ids(&rt), ["root"]);
    assert!(rt.state.blocks[0].edit.text.is_empty());
}

#[test]
fn nested_checkboxes_roll_up_through_plain_groups_and_toggle_all_descendants_atomically() {
    let mut rt = app(vec![
        block("parent", Kind::Todo, 0, "Prepare release"),
        block("group", Kind::Text, 1, "Checks"),
        block("first", Kind::Todo, 2, "Review"),
        block("step", Kind::Todo, 2, "Publish"),
        block("last", Kind::Todo, 3, "Verify build"),
        block("outside", Kind::Todo, 0, "Unrelated"),
    ]);
    tap(&mut rt, "check-first");
    assert!(!rt.state.blocks[0].data.checked);
    tap(&mut rt, "check-last");
    assert!(rt.state.blocks[..5].iter().all(|b| b.data.checked));
    assert!(!rt.state.blocks[5].data.checked);
    assert_eq!(
        folio::outline::task_totals(rt.state.blocks.iter().map(|b| &b.data)),
        (2, 3)
    );
    assert_eq!(
        folio::outline::task_progress(rt.state.blocks.iter().map(|b| &b.data))[0],
        (2, 2)
    );
    tap(&mut rt, "toggle-parent");
    assert!(rt.find("check-last").is_none());
    let undo = rt.state.undo.len();
    tap(&mut rt, "check-parent");
    assert!(rt.state.blocks[..5].iter().all(|b| !b.data.checked));
    assert_eq!(rt.state.undo.len(), undo + 1);
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.0);
    assert!(rt.state.blocks[0].data.collapsed);
    assert!(rt.state.blocks[..5].iter().all(|b| b.data.checked));
    tap(&mut rt, "toggle-parent");
    tap(&mut rt, "check-first");
    assert!(!rt.state.blocks[0].data.checked && !rt.state.blocks[1].data.checked);
    assert!(rt.state.blocks[3].data.checked);
    rt.update(|s, tx| s.remove_block("first", tx));
    assert!(
        rt.state.blocks[0].data.checked,
        "removing the incomplete child recalculates the parent"
    );
    rt.update(|s, tx| s.add_child_kind("group", Kind::Todo, tx));
    assert!(
        !rt.state.blocks[0].data.checked,
        "adding a subtask reopens the parent"
    );
}

#[test]
fn a_plain_point_can_add_subtasks_from_its_menu_and_fold_them_at_the_minimum_size() {
    let mut rt = app_at(
        vec![block("parent", Kind::Text, 0, "Appointment")],
        (920.0, 620.0),
    );
    tap(&mut rt, "handle-parent");
    tap(&mut rt, "block-subtask");
    let child = rt.state.blocks[1].data.id.clone();
    assert_eq!(rt.state.blocks[1].data.kind, Kind::Todo);
    assert_eq!(rt.state.blocks[1].data.indent, 1);
    key(&mut rt, Key::Text("Book the visit".into()), false, false);
    assert!(rt.find("check-parent").is_some());
    tap(&mut rt, &format!("check-{child}"));
    assert!(rt.state.blocks[0].data.checked);
    tap(&mut rt, "toggle-parent");
    assert!(rt.find(&input_key(&child)).is_none());
    tap(&mut rt, "toggle-parent");
    assert!(rt.find(&input_key(&child)).is_some());
    tap(&mut rt, "handle-parent");
    tap(&mut rt, "block-task");
    assert_eq!(rt.state.blocks[0].data.kind, Kind::Todo);
    assert_eq!(rt.state.blocks[0].edit.text, "Appointment");
    assert!(rt.state.blocks[0].data.checked);
}
