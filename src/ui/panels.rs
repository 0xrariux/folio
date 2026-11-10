use super::{Palette, action, label};
use crate::{
    editor::input_key,
    state::{Panel, State},
};
use folio::icons::controls as g;
use folio::templates::Template;
use rune::{Align, Color, El, ScrollSpec, Side, col, el, row};

pub(super) fn surface(key: &str, anchor: impl Into<String>, width: f32, p: Palette) -> El {
    col()
        .key(key)
        .role("menu")
        .instant_changes()
        .w(width)
        .padding(6.0)
        .gap(2.0)
        .radius(9.0)
        .fill(p.bg)
        .border(1.0, p.line)
        .shadow(8.0, 30.0, Color::BLACK.alpha(0.18))
        .anchored_fit(anchor, Side::Bottom, Align::Start, 5.0)
}
pub(super) fn item(
    key: impl Into<String>,
    title: &str,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    action(key, title, p, f).w_full().justify_start().h(34.0)
}
pub(super) fn icon_item(
    key: impl Into<String>,
    title: &str,
    glyph: &'static rune::Icon,
    p: Palette,
    f: impl Fn(&mut State, &mut rune::Tx) + 'static,
) -> El {
    super::icons::action(key, title, glyph, p, f)
        .w_full()
        .justify_start()
        .h(34.0)
}
fn slash(s: &State, p: Palette) -> El {
    let Panel::Slash { id, selected, .. } = &s.panel else {
        return el();
    };
    let options = s.slash_options();
    let mut entries = col().gap(2.0);
    for (index, kind) in options.iter().enumerate() {
        let kind = *kind;
        entries = entries.child(
            row()
                .key(format!("slash-{kind:?}"))
                .role("menuitem")
                .label(kind.name())
                .focusable(true)
                .h(48.0)
                .px(9.0)
                .gap(10.0)
                .center_y()
                .radius(5.0)
                .fill(if index == *selected { p.tint } else { p.bg })
                .hover(p.ink.alpha(0.045))
                .on_press_tx(move |s: &mut State, tx| s.choose_kind(kind, tx))
                .child(
                    el().size(31.0, 31.0)
                        .radius(5.0)
                        .border(1.0, p.line)
                        .center()
                        .child(rune::icon(g::block(kind), 19.0).color(p.ink).a11y_hidden()),
                )
                .child(
                    col()
                        .gap(3.0)
                        .child(label(kind.name(), 12.0, p.ink))
                        .child(label(kind.description(), 10.0, p.faint)),
                ),
        );
    }
    if options.is_empty() {
        entries = entries.child(label("Aucun type de bloc trouvé", 12.0, p.faint).padding(12.0));
    }
    surface("slash-menu", input_key(id), 330.0, p)
        .child(
            row()
                .px(8.0)
                .center_y()
                .child(label("INSÉRER UN BLOC", 9.0, p.faint))
                .child(el().grow(1.0))
                .child(super::icons::control(
                    "close-slash",
                    "Fermer les commandes de blocs",
                    &g::CLOSE,
                    p,
                    |s, _| s.panel = Panel::Closed,
                )),
        )
        .child(
            col()
                .h((options.len().max(1) as f32 * 50.0).min(310.0))
                .min_h(0.0)
                .shrink()
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(entries),
        )
        .child(label("↑ ↓  choisir    ↵  insérer    esc  fermer", 9.0, p.faint).padding(8.0))
}
fn page_menu(s: &State, p: Palette) -> El {
    let archived = s.current().is_some_and(|p| p.archived);
    let wide = s.current().is_some_and(|p| p.wide);
    let mut menu = surface("page-actions", "page-menu", 260.0, p)
        .child(icon_item(
            "duplicate-page",
            "Dupliquer la page",
            &g::COPY,
            p,
            |s, _| s.duplicate_page(),
        ))
        .child(icon_item(
            "move-page",
            "Déplacer vers…",
            &g::ARROW_UP_RIGHT,
            p,
            |s, _| s.panel = Panel::Move,
        ))
        .child(icon_item(
            "page-child",
            "Ajouter une sous-page",
            &g::PLUS,
            p,
            |s, tx| {
                s.create(true);
                tx.focus("page-title", false);
            },
        ))
        .child(el().h(1.0).fill(p.line))
        .child(super::icons::option(
            "wide-page",
            "Pleine largeur",
            wide,
            p,
            |s, _| s.appearance(None, false, true),
        ))
        .child(icon_item(
            "menu-cover",
            "Changer la couverture",
            &g::COVER,
            p,
            |s, _| s.appearance(None, true, false),
        ));
    if s.blocks.iter().any(|b| b.data.kind.is_toggle()) {
        menu = menu
            .child(icon_item(
                "fold-all",
                "Replier tous les tiroirs",
                &g::CHEVRON_RIGHT,
                p,
                |s, tx| s.fold_all(true, tx),
            ))
            .child(icon_item(
                "unfold-all",
                "Déplier tous les tiroirs",
                &g::CHEVRON_DOWN,
                p,
                |s, tx| s.fold_all(false, tx),
            ));
    }
    use crate::exporting::ExportKind;
    menu = menu.child(el().h(1.0).fill(p.line));
    if s.current()
        .is_some_and(|p| p.collection.is_some() || p.dashboard.is_some())
    {
        menu = menu
            .child(icon_item(
                "export-csv-view",
                "Exporter la vue en CSV",
                &g::EXPORT,
                p,
                |s, tx| s.export(ExportKind::CsvView, tx),
            ))
            .child(icon_item(
                "export-csv-all",
                "Toutes les fiches en CSV",
                &g::EXPORT,
                p,
                |s, tx| s.export(ExportKind::CsvAll, tx),
            ));
    } else {
        menu = menu.child(icon_item(
            "export-markdown",
            "Exporter en Markdown",
            &g::EXPORT,
            p,
            |s, tx| s.export(ExportKind::Markdown, tx),
        ));
    }
    menu.child(el().h(1.0).fill(p.line)).child(icon_item(
        "archive-page",
        if archived {
            "Restaurer la page"
        } else {
            "Archiver la page"
        },
        if archived {
            &g::UNDO
        } else {
            &folio::icons::ARCHIVE
        },
        p,
        |s, _| s.archive(),
    ))
}
fn move_menu(s: &State, p: Palette) -> El {
    let mut entries = col().gap(2.0).child(icon_item(
        "move-root",
        "Espace personnel (racine)",
        &folio::icons::FOLDER,
        p,
        |s, _| s.move_page(None),
    ));
    for page in s.pages.iter().filter(|page| s.can_move_to(&page.id)) {
        let id = page.id.clone();
        entries = entries.child(
            super::icons::page_action(
                format!("move-to-{id}"),
                page.display_title(),
                &page.emoji,
                p,
                move |s, _| s.move_page(Some(id.clone())),
            )
            .w_full()
            .justify_start()
            .h(34.0),
        );
    }
    surface("move-menu", "page-menu", 300.0, p)
        .child(label("DÉPLACER LA PAGE VERS", 10.0, p.faint).padding(9.0))
        .child(
            col()
                .h((s.pages.len() as f32 * 36.0).clamp(40.0, 300.0))
                .min_h(0.0)
                .shrink()
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(entries),
        )
}
fn templates(p: Palette) -> El {
    let mut menu = surface("templates-menu", "sidebar-more", 345.0, p)
        .anchored_fit("sidebar-more", Side::Top, Align::Start, 5.0)
        .child(label("COMMENCER AVEC UN MODÈLE", 10.0, p.faint).padding(9.0));
    for (index, template) in Template::ALL.into_iter().enumerate() {
        let glyph = match template {
            Template::Meeting => &folio::icons::NOTE,
            Template::Project => &folio::icons::PROJECT,
            Template::Tasks => &folio::icons::TASKS,
            Template::Tracker => &folio::icons::DATABASE,
            Template::Dashboard => &folio::icons::DASHBOARD,
            Template::Outline => &folio::icons::LIBRARY,
        };
        menu = menu.child(
            col()
                .key(format!("template-{index}"))
                .role("menuitem")
                .label(template.name())
                .focusable(true)
                .padding(12.0)
                .gap(5.0)
                .radius(6.0)
                .hover(p.tint)
                .on_press_tx(move |s: &mut State, tx| {
                    s.create_from_template(template);
                    tx.scroll_to_y("document-scroll", 0.0);
                })
                .child(
                    row()
                        .gap(8.0)
                        .center_y()
                        .child(rune::icon(glyph, 20.0).color(p.accent).a11y_hidden())
                        .child(label(template.name(), 14.0, p.ink)),
                )
                .child(label(template.description(), 11.0, p.faint)),
        );
    }
    menu
}
fn block_menu(s: &State, id: &str, p: Palette) -> El {
    let Some(index) = s.blocks.iter().position(|b| b.data.id == id) else {
        return el();
    };
    let block = &s.blocks[index].data;
    let count = s.block_end(index) - index;
    let convert = id.to_string();
    let mut entries = vec![icon_item(
        "block-convert",
        "Transformer en…",
        &g::TEXT,
        p,
        move |s, _| {
            s.panel = Panel::Slash {
                id: convert.clone(),
                selected: 0,
                typed: false,
            };
        },
    )];
    for (key, title, outdent, glyph) in [
        ("block-indent", "Indenter · Tab", false, &g::CHEVRON_RIGHT),
        (
            "block-outdent",
            "Désindenter · Maj+Tab",
            true,
            &g::CHEVRON_LEFT,
        ),
    ] {
        let id = id.to_string();
        entries.push(icon_item(key, title, glyph, p, move |s, tx| {
            s.indent_block(&id, outdent, tx);
        }));
    }
    if block.indent < folio::outline::MAX_INDENT {
        let id = id.to_string();
        entries.push(icon_item(
            "block-child",
            "Ajouter un sous-élément",
            &g::PLUS,
            p,
            move |s, tx| s.add_child(&id, tx),
        ));
    }
    if block.kind == folio::document::Kind::Numbered {
        let id = id.to_string();
        let restart = block.number_start.is_none();
        entries.push(icon_item(
            "block-numbering",
            if restart {
                "Recommencer à 1"
            } else {
                "Continuer la numérotation"
            },
            &g::NUMBERED_LIST,
            p,
            move |s, tx| {
                s.restart_numbering(&id, restart);
                s.focus_active(tx);
            },
        ));
    }
    let tasks: Vec<_> = s.blocks[s.task_scope(id)]
        .iter()
        .filter(|b| b.data.kind == folio::document::Kind::Todo)
        .collect();
    if !tasks.is_empty() {
        entries.push(
            label(
                format!(
                    "{} / {} tâches terminées",
                    tasks.iter().filter(|b| b.data.checked).count(),
                    tasks.len()
                ),
                10.0,
                p.faint,
            )
            .padding(9.0),
        );
        for (key, title, checked) in [
            ("list-check-all", "Tout cocher dans cette liste", true),
            ("list-uncheck-all", "Tout décocher dans cette liste", false),
        ] {
            let id = id.to_string();
            entries.push(icon_item(key, title, &g::CHECK, p, move |s, tx| {
                s.check_list(&id, checked);
                s.focus_active(tx);
            }));
        }
    }
    for (key, title, down, glyph) in [
        ("block-up", "Monter le groupe", false, &g::ARROW_UP),
        ("block-down", "Descendre le groupe", true, &g::ARROW_DOWN),
    ] {
        let id = id.to_string();
        entries.push(icon_item(key, title, glyph, p, move |s, tx| {
            s.move_block(&id, down);
            s.focus_active(tx);
        }));
    }
    let copy = id.to_string();
    entries.push(icon_item(
        "block-copy",
        if count > 1 {
            "Dupliquer avec les sous-éléments"
        } else {
            "Dupliquer"
        },
        &g::COPY,
        p,
        move |s, tx| s.duplicate_block(&copy, tx),
    ));
    let remove = id.to_string();
    entries.push(icon_item(
        "block-delete",
        &if count > 1 {
            format!("Supprimer le groupe ({count} blocs)")
        } else {
            "Supprimer le bloc".into()
        },
        &g::TRASH,
        p,
        move |s, tx| s.remove_block(&remove, tx),
    ));
    if let Some(page) = &s.selected {
        let url = crate::command_palette::reference_url(page, Some(id));
        entries.push(icon_item(
            "block-copy-link",
            "Copier le lien vers ce bloc",
            &folio::icons::LINK,
            p,
            move |s, tx| {
                tx.copy(&url);
                s.panel = Panel::Closed;
                s.export_message = Some("Lien du bloc copié.".into());
            },
        ));
    }
    surface("block-menu", format!("handle-{id}"), 290.0, p).child(
        col()
            .key("block-menu-scroll")
            .h((entries.len() as f32 * 36.0).min(400.0))
            .min_h(0.0)
            .shrink()
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .child(col().gap(2.0).children(entries)),
    )
}
fn icons(s: &State, p: Palette) -> El {
    let active = s.current().and_then(|p| folio::icons::choice(&p.emoji));
    let mut icons = col().gap(5.0);
    for (row_index, choices) in folio::icons::CHOICES.chunks(5).enumerate() {
        let mut line = row().gap(5.0);
        for (column, choice) in choices.iter().enumerate() {
            let value = choice.value;
            line = line.child(
                super::button(
                    format!("icon-{}", row_index * 5 + column),
                    choice.name,
                    p,
                    move |s, _| s.appearance(Some(value), false, false),
                )
                .size(52.0, 44.0)
                .help(choice.name)
                .fill(if active.is_some_and(|a| a.key == choice.key) {
                    p.tint
                } else {
                    p.bg
                })
                .child(rune::icon(choice.glyph, 23.0).color(p.accent).a11y_hidden()),
            );
        }
        icons = icons.child(line);
    }
    surface("icon-menu", "page-icon", 320.0, p)
        .padding(12.0)
        .gap(8.0)
        .child(label("UNE ICÔNE POUR CETTE PAGE", 9.0, p.faint).padding(8.0))
        .child(icons)
}
pub(super) fn view(s: &State, p: Palette) -> Vec<El> {
    let menu = match &s.panel {
        Panel::Closed => return vec![],
        Panel::SidebarMore => super::sidebar::more_menu(s, p),
        Panel::Workspace => return vec![super::workspace::menu(s, p)],
        Panel::History => return vec![super::workspace::history(s, p)],
        Panel::RestorePreview => return vec![super::workspace::restore(s, p)],
        Panel::Palette(form) => return vec![super::workspace::palette(s, form, p)],
        Panel::Date(form) => super::calendar::picker(s, form, p),
        Panel::CalendarDay(date) => return vec![super::calendar::day(s, date, p)],
        Panel::Slash { .. } => return vec![slash(s, p)],
        Panel::Page => page_menu(s, p),
        Panel::Move => move_menu(s, p),
        Panel::Templates => templates(p),
        Panel::Block(id) => block_menu(s, id, p),
        Panel::Icons => icons(s, p),
        Panel::CollectionFilter => super::collection_panels::filter(s, p),
        Panel::CollectionSort => super::collection_panels::sort(p),
        Panel::Property { id, field, anchor } => {
            super::collection_panels::property(id, *field, anchor, p)
        }
        Panel::Properties(form) => super::collection_panels::properties(form, p),
        Panel::Link(_) => super::collection_panels::links(s, p),
        Panel::Columns(form) => super::custom_fields::columns(s, form, p),
        Panel::Cell(form) => super::custom_fields::cell_form(s, form, p),
        Panel::DashboardSources => super::custom_fields::sources(s, p),
        Panel::Views(form) => super::saved_views::form(s, form, p),
    };
    vec![
        el().key("menu-backdrop")
            .absolute(0.0, 0.0)
            .w_full()
            .h_full()
            .on_press(|s: &mut State| s.panel = Panel::Closed),
        menu,
    ]
}
