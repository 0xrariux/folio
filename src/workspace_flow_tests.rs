use crate::{
    command_palette::{Action, Mode},
    editor::input_key,
    state::{Panel, State},
    workspace_worker::{Request, Response},
};
use folio::{
    collection::{Status, View},
    document::Kind,
    fields::{CellValue, ColumnKind},
};
use rune::{Env, Key, Mods, PhysicalInput, Runtime};
fn app(scene: &str) -> Runtime<State> {
    let mut state = State::demo();
    match scene {
        "lists" => state.demo_lists(),
        "table" => {
            state.select("demo-tracker");
            state.collection_view(View::Table);
        }
        "board" => {
            state.select("demo-tracker");
            state.collection_view(View::Board);
        }
        _ => {}
    }
    let mut rt = Runtime::new(
        state,
        crate::ui::view,
        (1200.0, 1000.0),
        Env::new(false, 1.0),
    );
    rt.on_key = Some(crate::on_key);
    rt.frame(0.0);
    rt
}
fn reveal(rt: &mut Runtime<State>, key: &str) {
    rt.update(|_, tx| tx.scroll_into_view(key));
    for _ in 0..40 {
        rt.frame(1.0 / 60.0);
    }
}
fn tap(rt: &mut Runtime<State>, key: &str) {
    reveal(rt, key);
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("Missing {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
}
fn replace(rt: &mut Runtime<State>, key: &str, value: &str) {
    tap(rt, key);
    rt.key(Key::Command('a').into());
    rt.key(Key::Text(value.into()).into());
    rt.frame(0.0);
}
fn drag(rt: &mut Runtime<State>, from: &str, to: &str) {
    reveal(rt, from);
    let (x, y) = rt.find(from).unwrap_or_else(|| panic!("Missing {from}"));
    let (dx, mut dy) = rt.find(to).unwrap_or_else(|| panic!("Missing {to}"));
    if to.starts_with("calendar-day-") {
        let ([_, top, _, height], _) = rt.presented("document-scroll").unwrap();
        dy = dy.clamp(top + 8.0, top + height - 8.0);
    }
    rt.pointer_down(x, y);
    rt.pointer_move(dx, dy);
    rt.frame(0.0);
    rt.pointer_up(dx, dy);
    rt.frame(0.0);
}

#[test]
fn command_palette_searches_all_pages_and_keyboard_executes_actions() {
    let mut rt = app("lists");
    rt.state.filter = crate::state::Filter::Favorites;
    rt.key(Key::Command('k').into());
    rt.frame(0.0);
    assert!(rt.find("palette-search").is_some());
    replace(&mut rt, "palette-search", "Lancement");
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    assert_eq!(rt.state.selected.as_deref(), Some("demo-launch"));
    rt.key(Key::Command('k').into());
    rt.frame(0.0);
    replace(&mut rt, "palette-search", "Changer de thème");
    let dark = rt.state.dark;
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    assert_ne!(rt.state.dark, dark);
}
#[test]
fn mouse_selection_keeps_subtrees_and_batch_actions_undo_once() {
    let mut rt = app("lists");
    rt.physical_input(PhysicalInput::Modifiers(Mods {
        cmd: true,
        logo: true,
        ..Default::default()
    }));
    rt.frame(0.0);
    tap(&mut rt, "handle-demo-list-3");
    tap(&mut rt, "handle-demo-list-6");
    rt.physical_input(PhysicalInput::Modifiers(Mods::default()));
    rt.frame(0.0);
    assert_eq!(rt.state.selection.ids.len(), 2);
    assert_eq!(rt.state.selected_indices().len(), 4);
    let before = rt.state.current().unwrap().blocks.clone();
    rt.state.undo.clear();
    tap(&mut rt, "selection-duplicate");
    assert_eq!(rt.state.blocks.len(), before.len() + 4);
    assert_eq!(rt.state.undo.len(), 1);
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.0);
    assert_eq!(rt.state.current().unwrap().blocks, before);
}
#[test]
fn board_drag_changes_status_and_cancellation_keeps_records() {
    let mut rt = app("board");
    let page = "demo-record-0";
    drag(&mut rt, &format!("record-{page}"), "board-drop-Done");
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == page)
            .unwrap()
            .properties
            .status,
        Status::Done
    );
    assert_eq!(rt.state.selected.as_deref(), Some("demo-tracker"));
    let source = "record-demo-record-1";
    reveal(&mut rt, source);
    let (x, y) = rt.find(source).unwrap();
    let (dx, dy) = rt.find("board-drop-Todo").unwrap();
    let status = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-record-1")
        .unwrap()
        .properties
        .status;
    rt.pointer_down(x, y);
    rt.pointer_move(dx, dy);
    rt.frame(0.0);
    rt.key(Key::Escape.into());
    rt.pointer_up(dx, dy);
    rt.frame(0.0);
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-1")
            .unwrap()
            .properties
            .status,
        status
    );
}
#[test]
fn page_drag_moves_into_parent_refuses_cycles_and_reorders_siblings() {
    let mut rt = app("lists");
    drag(&mut rt, "page-demo-meeting", "tree-inside-demo-ideas");
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-meeting")
            .unwrap()
            .parent_id
            .as_deref(),
        Some("demo-ideas")
    );
    let before = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-ideas")
        .unwrap()
        .parent_id
        .clone();
    rt.update(|s, _| s.drop_page("demo-ideas", "tree-inside-demo-project"));
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-ideas")
            .unwrap()
            .parent_id,
        before
    );
    rt.update(|s, _| s.drop_page("demo-meeting", "tree-before-demo-project"));
    let siblings: Vec<_> = rt
        .state
        .pages
        .iter()
        .filter(|p| p.parent_id.as_deref() == Some("demo-ideas"))
        .map(|p| p.id.as_str())
        .collect();
    assert_eq!(siblings, ["demo-meeting", "demo-project"]);
}
#[test]
fn calendar_picker_bulk_records_and_column_resize_are_real_ui_actions() {
    let mut rt = app("table");
    tap(&mut rt, "select-record-demo-record-0");
    tap(&mut rt, "select-record-demo-record-1");
    tap(&mut rt, "bulk-status-Done");
    assert!(
        rt.state
            .pages
            .iter()
            .filter(|p| p.id == "demo-record-0" || p.id == "demo-record-1")
            .all(|p| p.properties.status == Status::Done)
    );
    tap(&mut rt, "due-demo-record-0");
    tap(&mut rt, "pick-date-2026-11-25");
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .properties
            .due
            .as_deref(),
        Some("2026-11-25")
    );
    let key = "resize-column-name";
    reveal(&mut rt, key);
    let (x, y) = rt.find(key).unwrap();
    rt.pointer_down(x, y);
    rt.pointer_move(x + 80.0, y);
    rt.pointer_up(x + 80.0, y);
    rt.frame(0.0);
    assert!(
        rt.state
            .current()
            .unwrap()
            .collection
            .as_ref()
            .unwrap()
            .width("name", 190.0)
            > 240.0
    );
    tap(&mut rt, "view-calendar");
    assert!(rt.find("collection-calendar").is_some());
    drag(
        &mut rt,
        "calendar-record-demo-record-0",
        "calendar-day-2026-11-26",
    );
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-record-0")
            .unwrap()
            .properties
            .due
            .as_deref(),
        Some("2026-11-26"),
        "drag: {:?}, page: {:?}",
        rt.state
            .workspace
            .entity_drag
            .as_ref()
            .map(|d| (&d.id, d.active, &d.target, d.point)),
        rt.state.selected
    );
}
#[test]
fn relation_column_selector_and_calculated_cells_work_through_forms() {
    let mut rt = app("table");
    tap(&mut rt, "collection-columns");
    replace(&mut rt, "column-name", "Liés");
    tap(&mut rt, "column-kind-Relation");
    assert!(
        matches!(&rt.state.panel, Panel::Columns(f) if f.kind == ColumnKind::Relation),
        "the type choice must be reachable"
    );
    reveal(&mut rt, "relation-base-demo-launch");
    tap(&mut rt, "relation-base-demo-launch");
    assert!(
        matches!(&rt.state.panel, Panel::Columns(f) if f.config.target_base.as_deref() == Some("demo-launch")),
        "the target base must be reachable"
    );
    tap(&mut rt, "save-column");
    let column = rt
        .state
        .current()
        .unwrap()
        .collection
        .as_ref()
        .unwrap()
        .columns
        .last()
        .unwrap()
        .clone();
    assert_eq!(column.kind, ColumnKind::Relation);
    tap(&mut rt, &format!("cell-demo-record-0-{}", column.id));
    tap(&mut rt, "palette-item-demo-launch-record-0");
    assert!(
        matches!(rt.state.pages.iter().find(|p|p.id=="demo-record-0").unwrap().properties.custom.get(&column.id),Some(CellValue::Relation(ids))if ids==&vec!["demo-launch-record-0".to_string()])
    );
    let inverse = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-launch")
        .unwrap()
        .collection
        .as_ref()
        .unwrap()
        .columns
        .iter()
        .find(|c| c.config.reverse_of.as_deref() == Some(&column.id))
        .unwrap()
        .clone();
    let target = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-launch-record-0")
        .unwrap();
    assert_eq!(
        folio::calculations::related(&rt.state.pages, target, &inverse).len(),
        1
    );
    rt.update(|s, _| s.set_related("demo-launch-record-0", &inverse.id, "demo-record-0", false));
    let record = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-record-0")
        .unwrap();
    assert!(folio::calculations::related(&rt.state.pages, record, &column).is_empty());
}
#[test]
fn mentions_block_links_and_toc_keep_targets_and_expand_closed_parents() {
    let mut rt = app("lists");
    let id = rt.state.blocks[0].data.id.clone();
    tap(&mut rt, &input_key(&id));
    rt.key(Key::Text("@".into()).into());
    rt.frame(0.0);
    assert!(matches!(rt.state.panel, Panel::Palette(_)));
    replace(&mut rt, "palette-search", "Lancement");
    rt.key(Key::Enter.into());
    rt.frame(0.0);
    let block = rt.state.blocks.iter().find(|b| b.data.id == id).unwrap();
    assert!(
        block
            .data
            .html
            .contains("https://folio.local/page/demo-launch")
    );
    assert!(
        rt.state
            .backlinks("demo-launch")
            .iter()
            .any(|p| p.id == "demo-lists")
    );
    rt.update(|s, _| {
        s.select("demo-launch");
        s.title = rune::TextEdit::new("Lancement renommé");
        s.edit();
        s.select("demo-lists");
    });
    rt.frame(0.0);
    assert!(
        rt.state
            .current()
            .unwrap()
            .plain_text
            .contains("Lancement renommé")
    );
    rt.update(|s, tx| {
        s.follow_reference("https://folio.local/page/demo-lists#block=demo-list-12", tx);
    });
    rt.frame(0.0);
    assert!(!rt.state.blocks[11].data.collapsed);
    assert_eq!(rt.state.active.as_deref(), Some("demo-list-12"));
}
#[test]
fn embedded_base_shares_data_and_creates_records_in_its_source() {
    let mut rt = app("lists");
    let id = rt.state.blocks[0].data.id.clone();
    rt.update(|s, tx| {
        s.open_palette(Mode::Embed {
            block: id.clone(),
            base: None,
        });
        s.palette_choose(Action::Base("demo-launch".into()), tx);
        s.palette_choose(Action::View(None), tx);
    });
    rt.frame(0.0);
    let embed = rt
        .state
        .blocks
        .iter()
        .find(|b| b.data.kind == Kind::DatabaseView)
        .unwrap()
        .data
        .id
        .clone();
    tap(
        &mut rt,
        &format!("embed-status-{embed}-demo-launch-record-0"),
    );
    assert_eq!(
        rt.state
            .pages
            .iter()
            .find(|p| p.id == "demo-launch-record-0")
            .unwrap()
            .properties
            .status,
        Status::InProgress
    );
    tap(&mut rt, &format!("embed-new-{embed}"));
    assert_eq!(
        rt.state.current().unwrap().parent_id.as_deref(),
        Some("demo-launch")
    );
}
#[test]
fn worker_backup_restore_and_history_are_ordered_and_survive_restart() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("worker.db");
    let backup = temp.path().join("copy.folio");
    let worker = crate::worker::Worker::start(path.clone(), std::sync::Arc::new(|| {}));
    let mut page = folio::model::Page::new(None);
    page.title = "Version initiale".into();
    worker.save(page.clone(), 1).unwrap();
    worker
        .workspace(Request::Checkpoint(page.id.clone()))
        .unwrap();
    page.title = "Version récente".into();
    worker.save(page.clone(), 2).unwrap();
    worker.workspace(Request::Backup(backup.clone())).unwrap();
    for _ in 0..12 {
        if let crate::worker::Event::Workspace(Ok(Response::BackupDone(_))) = worker
            .events
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap()
        {
            break;
        }
    }
    let bundle = folio::workspace::Notebook::read(&backup).unwrap();
    assert!(bundle.pages.iter().any(|p| p.title == "Version récente"));
    let old = bundle
        .history
        .iter()
        .find(|h| h.page_id == page.id && h.page.title == "Version initiale")
        .unwrap();
    worker
        .workspace(Request::RestoreVersion {
            page: page.id.clone(),
            version: old.id.clone(),
        })
        .unwrap();
    drop(worker);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let store = folio::storage::Store::open(&path).await.unwrap();
        assert_eq!(
            store.load_one(&page.id).await.unwrap().unwrap().title,
            "Version initiale"
        );
        assert!(
            store
                .history(Some(&page.id))
                .await
                .unwrap()
                .iter()
                .any(|h| h.page.title == "Version récente")
        );
    });
}

#[test]
fn imported_images_live_in_the_notebook_resize_and_export_with_their_bytes() {
    use base64::Engine;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("Point.png");
    let bytes=base64::engine::general_purpose::STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=").unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let mut rt = app("lists");
    rt.update(|s, _| {
        s.workspace.images = Some(rune::ImageCache::new(std::sync::Arc::new(|| {})));
        s.import_files(std::slice::from_ref(&path));
    });
    rt.frame(0.0);
    let image = rt
        .state
        .blocks
        .iter()
        .find(|b| b.data.kind == Kind::Image)
        .unwrap()
        .data
        .clone();
    let asset = image.asset.as_ref().unwrap();
    assert_eq!(
        std::fs::read(folio::workspace::asset_path(&rt.state.asset_folder(), &asset.id).unwrap())
            .unwrap(),
        bytes
    );
    let handle = format!("image-resize-{}", image.id);
    reveal(&mut rt, &handle);
    let (x, y) = rt.find(&handle).unwrap();
    rt.state.undo.clear();
    rt.pointer_down(x, y);
    rt.pointer_move(x - 180.0, y);
    rt.pointer_up(x - 180.0, y);
    rt.frame(0.0);
    assert!(
        rt.state
            .blocks
            .iter()
            .find(|b| b.data.id == image.id)
            .unwrap()
            .data
            .asset
            .as_ref()
            .unwrap()
            .width_percent
            < 100
    );
    assert_eq!(rt.state.undo.len(), 1);
    rt.update(|s, tx| s.undo_document(false, tx));
    rt.frame(0.0);
    assert_eq!(
        rt.state
            .blocks
            .iter()
            .find(|b| b.data.id == image.id)
            .unwrap()
            .data
            .asset
            .as_ref()
            .unwrap()
            .width_percent,
        100
    );
    let export = folio::export::markdown(rt.state.current().unwrap(), &rt.state.pages);
    assert!(export.contents.contains("![Point\\.png](folio-assets/"));
    let destination = temp.path().join("export");
    folio::workspace::export_assets(&export.assets, &rt.state.asset_folder(), &destination)
        .unwrap();
    assert_eq!(
        std::fs::read(destination.join(folio::workspace::asset_export_name(asset))).unwrap(),
        bytes
    );
}
#[test]
fn formulas_and_rollup_configuration_can_be_created_from_the_properties_form() {
    let mut rt = app("lists");
    rt.update(|s, _| s.demo_workspace());
    rt.frame(0.0);
    tap(&mut rt, "page-demo-projects-base");
    tap(&mut rt, "collection-columns");
    replace(&mut rt, "column-name", "Fiches restantes");
    tap(&mut rt, "column-kind-Rollup");
    tap(&mut rt, "rollup-relation-demo-project-tasks");
    tap(&mut rt, "aggregate-Remaining");
    tap(&mut rt, "save-column");
    let field = rt
        .state
        .current()
        .unwrap()
        .collection
        .as_ref()
        .unwrap()
        .columns
        .last()
        .unwrap();
    assert_eq!(field.kind, ColumnKind::Rollup);
    assert_eq!(field.config.aggregate, folio::fields::Aggregate::Remaining);
    tap(&mut rt, "collection-columns");
    replace(&mut rt, "column-name", "Charge doublée");
    tap(&mut rt, "column-kind-Formula");
    replace(
        &mut rt,
        "column-expression",
        "prop(\"Charge totale (h)\") * 2",
    );
    tap(&mut rt, "save-column");
    let field = rt
        .state
        .current()
        .unwrap()
        .collection
        .as_ref()
        .unwrap()
        .columns
        .last()
        .unwrap();
    assert_eq!(field.kind, ColumnKind::Formula);
    let project = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-workspace-project")
        .unwrap();
    assert!(folio::calculations::value(project, field, &rt.state.pages).unwrap() > 0.0);
}

#[test]
fn native_backup_and_restore_dialogs_preview_before_replacing_the_notebook() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("dialog.db");
    let backup = temp.path().join("snapshot.folio");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut original = folio::model::Page::new(None);
    original.title = "Avant sauvegarde".into();
    runtime.block_on(async {
        let store = folio::storage::Store::open(&path).await.unwrap();
        store.save(&original).await.unwrap();
    });
    let mut state = State::new();
    state.worker = Some(crate::worker::Worker::start(
        path.clone(),
        std::sync::Arc::new(|| {}),
    ));
    loop {
        let event = state
            .worker
            .as_ref()
            .unwrap()
            .events
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let ready = matches!(
            &event,
            crate::worker::Event::Workspace(Ok(Response::Preferences(_)))
        );
        state.accept(event);
        if ready {
            break;
        }
    }
    let mut rt = Runtime::new(
        state,
        crate::ui::view,
        (1200.0, 1000.0),
        Env::new(false, 1.0),
    );
    rt.frame(0.0);
    rt.update(|s, tx| s.backup_dialog(tx));
    rt.frame(0.0);
    assert!(rt.file_dialog().is_some());
    rt.resolve_file_dialog(Some(backup.clone())).unwrap();
    rt.frame(0.0);
    loop {
        let event = rt
            .state
            .worker
            .as_ref()
            .unwrap()
            .events
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let done = matches!(
            &event,
            crate::worker::Event::Workspace(Ok(Response::BackupDone(_)))
        );
        rt.update(|s, _| s.accept(event));
        if done {
            break;
        }
    }
    rt.state.title = rune::TextEdit::new("Après sauvegarde");
    rt.state.edit();
    rt.update(|s, tx| s.restore_dialog(tx));
    rt.frame(0.0);
    rt.resolve_file_dialog(Some(backup)).unwrap();
    rt.frame(0.0);
    loop {
        let event = rt
            .state
            .worker
            .as_ref()
            .unwrap()
            .events
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let done = matches!(
            &event,
            crate::worker::Event::Workspace(Ok(Response::Preview(_)))
        );
        rt.update(|s, _| s.accept(event));
        if done {
            break;
        }
    }
    rt.frame(0.0);
    assert_eq!(rt.state.current().unwrap().title, "Après sauvegarde");
    assert!(rt.find("confirm-restore").is_some());
    tap(&mut rt, "confirm-restore");
    assert!(rt.state.workspace.busy);
    loop {
        let event = rt
            .state
            .worker
            .as_ref()
            .unwrap()
            .events
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let done = matches!(
            &event,
            crate::worker::Event::Workspace(Ok(Response::Restored { .. }))
        );
        rt.update(|s, _| s.accept(event));
        if done {
            break;
        }
    }
    assert_eq!(rt.state.current().unwrap().title, "Avant sauvegarde");
    assert!(!rt.state.workspace.busy);
    drop(rt);
    assert!(
        std::fs::read_dir(path.with_extension("backups"))
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| folio::workspace::Notebook::read(&entry.path())
                .is_ok_and(|b| b.pages.iter().any(|p| p.title == "Après sauvegarde")))
    );
}

#[test]
fn a_record_created_in_a_linked_view_updates_the_inverse_relation_and_rollup() {
    let mut rt = app("lists");
    rt.update(|s, _| s.demo_workspace());
    rt.frame(0.0);
    tap(&mut rt, "embed-new-demo-workspace-3");
    let created = rt.state.selected.clone().unwrap();
    assert_eq!(
        rt.state.current().unwrap().parent_id.as_deref(),
        Some("demo-tracker")
    );
    let project = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-workspace-project")
        .unwrap();
    let base = rt
        .state
        .pages
        .iter()
        .find(|p| p.id == "demo-projects-base")
        .unwrap();
    let progress = base
        .collection
        .as_ref()
        .unwrap()
        .columns
        .iter()
        .find(|c| c.id == "demo-project-progress")
        .unwrap();
    assert_eq!(
        folio::calculations::value(project, progress, &rt.state.pages).unwrap(),
        25.0
    );
    let csv = folio::export::csv_with_context("Projets", &[base], &[project], &rt.state.pages);
    assert!(csv.contents.contains(&created));
}
