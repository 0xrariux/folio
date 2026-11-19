use crate::{
    command_palette::Mode,
    editor::EditorBlock,
    state::{Panel, Pending, State},
    worker::Event,
};
use folio::{
    collection::{Collection, Sort, View},
    document::{Kind, NoteBlock},
    fields::CellValue,
    model::Page,
};
use rune::{Env, Key, KeyEvent, Mods, Runtime, TextEdit};
use std::time::Instant;

fn base(count: usize) -> State {
    let mut s = State::demo();
    let mut base = Page::new(None);
    base.id = "large-base".into();
    base.title = "Grande base".into();
    base.wide = true;
    base.collection = Some(Collection {
        view: View::Table,
        sort: Sort::Title,
        ..Collection::default()
    });
    s.pages = vec![base];
    for index in 0..count {
        let mut page = Page::new(Some("large-base".into()));
        page.id = format!("record-{index:05}");
        page.title = format!("Fiche {index:05}");
        s.pages.push(page);
    }
    s.selected = None;
    s.select("large-base");
    s.collapsed.insert("large-base".into());
    s
}
fn app(s: State) -> Runtime<State> {
    let mut rt = Runtime::new(s, crate::ui::view, (1200.0, 820.0), Env::new(false, 1.0));
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
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("Missing {key}"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
}
fn mounted(rt: &Runtime<State>) -> usize {
    rt.elements()
        .iter()
        .filter(|el| {
            el.key
                .as_deref()
                .is_some_and(|k| k.starts_with("table-row-"))
        })
        .count()
}

#[test]
fn menu_triggers_open_on_mouse_down_once_and_keep_keyboard_activation() {
    for (key, expected) in [
        ("page-menu", Panel::Page),
        ("sidebar-more", Panel::SidebarMore),
        ("customize-navigation", Panel::SidebarNavigation),
    ] {
        let mut rt = app(State::demo());
        reveal(&mut rt, key);
        let (x, y) = rt.find(key).unwrap();
        rt.pointer_down(x, y);
        rt.frame(0.);
        assert_eq!(
            std::mem::discriminant(&rt.state.panel),
            std::mem::discriminant(&expected),
            "{key} did not open on mouse down"
        );
        rt.pointer_up(x, y);
        rt.frame(0.);
        assert_eq!(
            std::mem::discriminant(&rt.state.panel),
            std::mem::discriminant(&expected),
            "{key} activated again on release"
        );
        rt.key(Key::Escape.into());
        rt.frame(0.);
        assert!(matches!(rt.state.panel, Panel::Closed));
        rt.update(|_, tx| tx.focus(key, false));
        rt.frame(0.);
        rt.key(Key::Text(" ".into()).into());
        rt.frame(0.);
        assert_eq!(
            std::mem::discriminant(&rt.state.panel),
            std::mem::discriminant(&expected),
            "{key} lost keyboard activation"
        );
    }
}

#[test]
fn virtual_table_reaches_unmounted_rows_and_keeps_selection_after_reordering() {
    let mut rt = app(base(1_000));
    assert!(mounted(&rt) < 100);
    assert!(rt.find("select-record-record-00999").is_none());
    // Scroll to the stable virtual row key, even though its controls do not exist yet.
    reveal(&mut rt, "table-row-record-00999");
    tap(&mut rt, "select-record-record-00999");
    assert!(rt.state.workspace.selected_records.contains("record-00999"));
    rt.update(|s, _| s.pages.last_mut().unwrap().title = "AAAA".into());
    reveal(&mut rt, "table-row-record-00999");
    tap(&mut rt, "select-record-record-00999");
    assert!(rt.state.workspace.selected_records.is_empty());
    tap(&mut rt, "select-all-records");
    assert_eq!(rt.state.workspace.selected_records.len(), 1_000);
    rt.update(|s, _| s.collection_search = TextEdit::new("Fiche 00001"));
    rt.frame(0.0);
    assert_eq!(rt.state.collection_items().len(), 1);
    tap(&mut rt, "select-all-records");
    assert_eq!(rt.state.workspace.selected_records.len(), 999);
    assert!(!rt.state.workspace.selected_records.contains("record-00001"));
    assert!(rt.state.workspace.selected_records.contains("record-00999"));
}

#[test]
fn virtual_table_preserves_focused_control_when_scrolled_away() {
    let mut rt = app(base(1_000));
    reveal(&mut rt, "table-row-record-00000");
    rt.update(|_, tx| tx.focus("select-record-record-00000", true));
    rt.frame(0.0);
    reveal(&mut rt, "table-row-record-00999");
    // Offscreen pinned rows are absent from the inspector, but retain keyboard focus.
    rt.key(Key::Text(" ".into()).into());
    rt.frame(0.0);
    assert!(rt.state.workspace.selected_records.contains("record-00000"));
    assert!(mounted(&rt) < 100);
}

#[test]
fn unicode_search_is_consistent_for_pages_commands_links_and_record_properties() {
    let mut s = base(2);
    s.pages[1].title = "Réunion Straße ＡＰＩ".into();
    s.pages[2].title = "Reunion".into();
    assert!(s.pages[1].matches("  REUNION  "));
    assert!(s.pages[1].matches("STRASSE"));
    assert!(s.pages[1].matches("api"));
    assert!(!s.pages[2].matches("réunion"));
    s.collection_search = TextEdit::new("reunion");
    assert_eq!(s.collection_items().len(), 2);
    s.collection_search = TextEdit::new("réunion");
    assert_eq!(s.collection_items().len(), 1);
    s.pages[2].properties.tags = vec!["Échéance".into()];
    s.collection_search = TextEdit::new("echeance");
    assert_eq!(s.collection_items()[0].id, "record-00001");
    s.pages[2]
        .properties
        .custom
        .insert("notes".into(), CellValue::Text("À préparer".into()));
    s.collection_search = TextEdit::new("preparer");
    assert_eq!(s.collection_items()[0].id, "record-00001");
    s.open_palette(Mode::Commands);
    s.palette_edit().text = "INSERT".into();
    assert!(
        s.palette_items()
            .iter()
            .any(|item| item.title == "Insert a database in the page")
    );
    s.open_palette(Mode::Mention {
        block: "unused".into(),
        start: 0,
        end: 0,
    });
    s.palette_edit().text = "reunion".into();
    assert_eq!(s.palette_items().len(), 2);
    s.palette_edit().text = "réunion".into();
    assert_eq!(s.palette_items().len(), 1);
    s.open_link_picker("unused".into(), false);
    let Panel::Link(form) = &mut s.panel else {
        unreachable!()
    };
    form.query.text = "reunion".into();
    assert_eq!(s.link_targets().len(), 2);
}

#[test]
fn headings_remain_editable_and_f6_cycles_between_navigation_and_main() {
    let mut s = base(0);
    s.pages[0].collection = None;
    s.pages[0].blocks = vec![NoteBlock::text(Kind::ToggleHeading2, "Échéances du projet")];
    s.selected = None;
    s.select("large-base");
    let block = s.blocks[0].data.id.clone();
    let mut rt = app(s);
    let elements = rt.elements();
    let heading = elements
        .iter()
        .find(|el| el.key.as_deref() == Some("page-heading"))
        .unwrap();
    assert_eq!(
        (heading.role, heading.level, heading.label.as_deref()),
        ("heading", Some(1), Some("Grande base"))
    );
    let heading = elements
        .iter()
        .find(|el| el.key.as_deref() == Some(&format!("block-{block}")))
        .unwrap();
    assert_eq!(
        (heading.role, heading.level, heading.label.as_deref()),
        ("heading", Some(3), Some("Échéances du projet"))
    );
    rt.update(|_, tx| tx.focus("page-title", true));
    rt.frame(0.0);
    rt.key(Key::Function(6).into());
    rt.frame(0.0);
    let focused = rt.elements().into_iter().find(|el| el.focused).unwrap();
    assert!(focused.path.contains("sidebar"), "{}", focused.path);
    rt.key(KeyEvent {
        key: Key::Function(6),
        mods: Mods {
            shift: true,
            ..Mods::default()
        },
    });
    rt.frame(0.0);
    assert!(
        rt.elements()
            .iter()
            .any(|el| el.key.as_deref() == Some("page-title") && el.focused)
    );
    rt.key(Key::Command('a').into());
    rt.key(Key::Text("Titre modifié".into()).into());
    rt.frame(0.0);
    assert_eq!(rt.state.title.text, "Titre modifié");
}

#[test]
fn saving_announces_completion_and_current_errors() {
    let mut rt = app(base(0));
    rt.state.pending.insert(
        "large-base".into(),
        Pending {
            revision: 2,
            error: None,
        },
    );
    rt.update(|s, tx| {
        crate::poll_updates(
            s,
            vec![Event::Saved {
                id: "large-base".into(),
                revision: 1,
            }],
            tx,
        );
    });
    assert!(
        !rt.accessibility_json()
            .to_string()
            .contains("All changes saved")
    );
    rt.update(|s, tx| {
        crate::poll_updates(
            s,
            vec![Event::Saved {
                id: "large-base".into(),
                revision: 2,
            }],
            tx,
        );
    });
    assert!(
        rt.accessibility_json()
            .to_string()
            .contains("All changes saved")
    );
    rt.state.pending.insert(
        "large-base".into(),
        Pending {
            revision: 3,
            error: None,
        },
    );
    rt.update(|s, tx| {
        crate::poll_updates(
            s,
            vec![Event::SaveFailed {
                id: "large-base".into(),
                revision: 3,
                message: "Disque plein".into(),
            }],
            tx,
        );
    });
    assert!(
        rt.accessibility_json()
            .to_string()
            .contains("Save failed: Disque plein")
    );
}

#[test]
fn measure_large_views() {
    for count in [1_000, 10_000] {
        let s = base(count);
        let start = Instant::now();
        let mut rt = app(s);
        let first = start.elapsed();
        let start = Instant::now();
        rt.update(|s, _| s.dark = !s.dark);
        rt.frame(0.0);
        let update = start.elapsed();
        eprintln!(
            "table: {count} records, first frame {first:?}, theme update {update:?}, {} mounted rows",
            mounted(&rt)
        );
        assert!(mounted(&rt) < 100);
    }
    let mut s = base(0);
    s.pages[0].collection = None;
    s.selected = None;
    s.select("large-base");
    s.blocks = (0..500)
        .map(|i| {
            EditorBlock::from(NoteBlock::text(
                Kind::Text,
                &format!("Paragraphe {i} à mesurer."),
            ))
        })
        .collect();
    s.commit_blocks(false);
    let start = Instant::now();
    let mut rt = app(s);
    let first = start.elapsed();
    let start = Instant::now();
    rt.update(|s, _| s.dark = !s.dark);
    rt.frame(0.0);
    eprintln!(
        "document: 500 blocks, first frame {first:?}, theme update {:?}",
        start.elapsed()
    );
}
