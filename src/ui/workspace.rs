use super::{Palette, action, icons, label, panels::icon_item};
use crate::{
    command_palette::{Form, Mode},
    state::{Panel, State},
};
use folio::icons::controls as g;
use rune::{El, Key, ScrollSpec, TextStyle, col, el, row, text_input};

pub(super) fn dialog(title: &str, p: Palette, body: El) -> El {
    el().key("workspace-overlay")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .fill(rune::Color::BLACK.alpha(0.28))
        .center()
        .modal_tx(true, |s: &mut State, _| {
            if !s.workspace.busy {
                s.panel = Panel::Closed;
            }
        })
        .child(
            col()
                .w(560.0)
                .padding(18.0)
                .gap(12.0)
                .radius(10.0)
                .fill(p.bg)
                .border(1.0, p.line)
                .shadow(8.0, 35.0, rune::Color::BLACK.alpha(0.2))
                .child(
                    row()
                        .center_y()
                        .child(
                            label(title, 17.0, p.ink)
                                .role("heading")
                                .level(2)
                                .label(title),
                        )
                        .child(el().grow(1.0))
                        .child(icons::control(
                            "close-workspace",
                            "Fermer",
                            &g::CLOSE,
                            p,
                            |s, _| s.panel = Panel::Closed,
                        )),
                )
                .child(body),
        )
}
pub(super) fn menu(s: &State, p: Palette) -> El {
    let body = col().gap(5.0)
        .child(icon_item("workspace-backup", "Sauvegarder le carnet complet…", &g::EXPORT, p, |s, tx| s.backup_dialog(tx)))
        .child(icon_item("workspace-restore", "Restaurer une sauvegarde…", &g::UNDO, p, |s, tx| s.restore_dialog(tx)))
        .child(icon_item("workspace-history", "Historique de cette page", &g::CLOCK, p, |s, _| s.open_history()))
        .child(icon_item("workspace-attach", "Ajouter une image ou un document…", &g::COVER, p, |s, tx| s.attachment_dialog(tx)))
        .child(icon_item("workspace-commands", "Ouvrir les commandes · ⌘K", &g::SEARCH, p, |s, tx| { s.open_palette(Mode::Commands); tx.focus("palette-search", false); }))
        .child(label(if s.demo { "Le mode démo garde les notes en mémoire." } else { "Sauvegardes automatiques à l’ouverture et après une session modifiée. Les 14 dernières sont conservées à côté de la base." }, 11.0, p.faint).w_full().wrap());
    dialog("Votre carnet", p, body)
}
pub(super) fn history(s: &State, p: Palette) -> El {
    let mut rows = col().gap(6.0);
    for version in &s.workspace.versions {
        let id = version.id.clone();
        let date = chrono::DateTime::from_timestamp(version.created_at, 0)
            .map_or_else(String::new, |d| d.format("%d/%m/%Y %H:%M UTC").to_string());
        rows = rows.child(
            row()
                .gap(10.0)
                .center_y()
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(3.0)
                        .child(label(&version.page.title, 12.0, p.ink).truncate())
                        .child(label(
                            format!("{date} · {} blocs", version.page.blocks.len()),
                            10.0,
                            p.faint,
                        )),
                )
                .child(action(
                    format!("restore-version-{id}"),
                    "Restaurer",
                    p,
                    move |s, _| {
                        s.workspace.busy = true;
                        s.restore_version(&id);
                    },
                )),
        );
    }
    if s.workspace.versions.is_empty() {
        rows = rows.child(label(
            "Aucune version antérieure enregistrée.",
            12.0,
            p.faint,
        ));
    }
    dialog("Historique de la page", p, col().gap(12.0)
        .child(label("Les modifications conservent une version précédente par minute, jusqu’à 50 versions. Restaurer conserve aussi l’état actuel.", 11.0, p.muted).w_full().wrap())
        .child(col().h(280.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(rows))
        .child(icons::action("checkpoint", "Créer un point de restauration", &g::PLUS, p, |s, _| s.checkpoint())))
}
pub(super) fn restore(s: &State, p: Palette) -> El {
    let Some(bundle) = &s.workspace.preview else {
        return el();
    };
    dialog("Restaurer ce carnet", p, col().gap(12.0)
        .child(label(format!("{} pages · {} fichiers · {} versions de pages", bundle.pages.len(), bundle.assets.len(), bundle.history.len()), 14.0, p.ink))
        .child(label("Le carnet actuel sera remplacé. Une sauvegarde complète de son état sera créée avant la restauration.", 12.0, p.muted).w_full().wrap())
        .child(row().gap(8.0).child(el().grow(1.0)).child(action("cancel-restore", "Annuler", p, |s, _| { s.workspace.preview = None; s.panel = Panel::Closed; }))
            .child(action("confirm-restore", "Restaurer le carnet", p, |s, _| s.confirm_restore()).fill(p.tint))))
}
pub(super) fn busy(p: Palette) -> El {
    el().key("workspace-busy")
        .absolute(0.0, 0.0)
        .w_full()
        .h_full()
        .fill(rune::Color::BLACK.alpha(0.15))
        .center()
        .modal(false, |_: &mut State| {})
        .child(
            label("Le carnet est en cours de traitement…", 14.0, p.ink)
                .padding(22.0)
                .radius(8.0)
                .fill(p.bg),
        )
}
pub(super) fn palette(s: &State, form: &Form, p: Palette) -> El {
    let title = match &form.mode {
        Mode::Commands => "Pages et commandes",
        Mode::Mention { .. } => "Mentionner une page",
        Mode::BlockLink { .. } => "Lier un bloc",
        Mode::Embed { .. } => "Insérer une vue de base",
        Mode::Relation { .. } => "Lier des fiches",
    };
    let items = s.palette_items();
    let mut rows = col().gap(3.0);
    for (index, item) in items.into_iter().enumerate() {
        let action = item.action;
        rows = rows.child(
            row()
                .key(format!("palette-item-{}", item.key))
                .role("button")
                .label(&item.title)
                .focusable(true)
                .h(47.0)
                .px(10.0)
                .radius(5.0)
                .fill(if index == form.selected { p.tint } else { p.bg })
                .hover(p.tint)
                .on_press_tx(move |s: &mut State, tx| s.palette_choose(action.clone(), tx))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(3.0)
                        .child(
                            label(&item.title, 12.0, p.ink).truncate().text_highlights(
                                rune_kit::search::Fold::for_language("fr")
                                    .find(&item.title, form.query.text.trim())
                                    .into_iter()
                                    .map(|range| {
                                        rune::TextHighlight::new(range, p.accent.alpha(0.22))
                                    }),
                            ),
                        )
                        .child(label(item.detail, 10.0, p.faint).truncate()),
                ),
        );
    }
    dialog(
        title,
        p,
        col()
            .gap(10.0)
            .child(
                text_input(&form.query, TextStyle::new(14.0, 400.0))
                    .key("palette-search")
                    .label("Rechercher une page ou une commande")
                    .single_line()
                    .w_full()
                    .h(36.0)
                    .px(9.0)
                    .border(1.0, p.line)
                    .radius(6.0)
                    .color(p.ink)
                    .caret(p.accent, p.tint)
                    .placeholder("Rechercher…", p.faint)
                    .on_edit(|s: &mut State| s.palette_edit())
                    .on_input_key(|s: &mut State, event, tx| {
                        let items = s.palette_items();
                        match event.key {
                            Key::Up | Key::Down => {
                                if let Panel::Palette(form) = &mut s.panel {
                                    let n = items.len().max(1);
                                    form.selected = if event.key == Key::Down {
                                        (form.selected + 1) % n
                                    } else {
                                        (form.selected + n - 1) % n
                                    };
                                    if let Some(item) = items.get(form.selected) {
                                        tx.scroll_into_view(format!("palette-item-{}", item.key));
                                    }
                                }
                                true
                            }
                            Key::Enter => {
                                if let Panel::Palette(form) = &s.panel
                                    && let Some(item) = items.get(form.selected)
                                {
                                    s.palette_choose(item.action.clone(), tx);
                                }
                                true
                            }
                            Key::Escape => {
                                s.panel = Panel::Closed;
                                s.focus_active(tx);
                                true
                            }
                            _ => false,
                        }
                    }),
            )
            .child(
                col()
                    .key("palette-results")
                    .h(300.0)
                    .min_h(0.0)
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .child(rows),
            )
            .child(label(
                "↑ ↓ choisir · Entrée ouvrir · Échap fermer",
                10.0,
                p.faint,
            )),
    )
}
