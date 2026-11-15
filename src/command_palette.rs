use crate::state::{Panel, State};
use folio::{
    document::Kind,
    fields::{CellValue, ColumnKind},
    workspace::EmbeddedView,
};
use rune::{Mark, MarkRange, TextEdit, Tx};
use rune_kit::search::Fold;
#[derive(Clone)]
pub enum Mode {
    Commands,
    Mention {
        block: String,
        start: usize,
        end: usize,
    },
    BlockLink {
        block: String,
        start: usize,
        end: usize,
    },
    Embed {
        block: String,
        base: Option<String>,
    },
    Relation {
        record: String,
        column: String,
    },
}
#[derive(Clone)]
pub struct Form {
    pub query: TextEdit,
    pub selected: usize,
    pub mode: Mode,
}
#[derive(Clone)]
pub enum Action {
    Page(String),
    Block(String, String),
    Command(&'static str),
    Base(String),
    View(Option<String>),
    Relation(String),
}
pub struct Item {
    pub key: String,
    pub title: String,
    pub detail: String,
    pub action: Action,
}
impl State {
    pub fn new_embedded_record(&mut self, id: &str, tx: &mut Tx) {
        let Some(embed) = self
            .blocks
            .iter()
            .find(|b| b.data.id == id)
            .and_then(|b| b.data.embedded.clone())
        else {
            return;
        };
        if !self
            .pages
            .iter()
            .any(|p| p.id == embed.base_id && p.collection.is_some() && !p.archived)
        {
            return;
        }
        let page = folio::model::Page::new(Some(embed.base_id));
        let id = page.id.clone();
        let host = self.selected.clone();
        self.add_page(page);
        if let (Some(column), Some(host)) = (embed.relation_filter, host) {
            self.set_related(&id, &column, &host, true);
        }
        tx.focus("page-title", false);
    }
    pub fn open_palette(&mut self, mode: Mode) {
        self.panel = Panel::Palette(Box::new(Form {
            query: TextEdit::new(""),
            selected: 0,
            mode,
        }));
    }
    pub fn palette_edit(&mut self) -> &mut TextEdit {
        let Panel::Palette(form) = &mut self.panel else {
            unreachable!()
        };
        form.selected = 0;
        &mut form.query
    }
    pub fn palette_items(&self) -> Vec<Item> {
        let Panel::Palette(form) = &self.panel else {
            return Vec::new();
        };
        let q = form.query.text.trim();
        let mut items = Vec::new();
        match &form.mode {
            Mode::Commands => {
                for (key, title) in [
                    ("new", "Nouvelle page"),
                    ("base", "Nouvelle base"),
                    ("templates", "Choisir un modèle"),
                    ("theme", "Changer de thème"),
                    ("backup", "Sauvegarder le carnet complet"),
                    ("restore", "Restaurer un carnet"),
                    ("history", "Historique de cette page"),
                    ("file", "Ajouter une image ou un fichier"),
                    ("embed", "Insérer une base dans la page"),
                    ("block-link", "Insérer un lien vers un bloc"),
                ] {
                    if Fold::for_language("fr").contains(title, q) {
                        items.push(Item {
                            key: key.into(),
                            title: title.into(),
                            detail: "Commande".into(),
                            action: Action::Command(key),
                        });
                    }
                }
                for p in self
                    .pages
                    .iter()
                    .filter(|p| !p.archived && p.matches(q))
                    .take(40)
                {
                    items.push(Item {
                        key: p.id.clone(),
                        title: p.display_title().into(),
                        detail: "Page du carnet".into(),
                        action: Action::Page(p.id.clone()),
                    });
                }
            }
            Mode::Mention { .. } => {
                for p in self
                    .pages
                    .iter()
                    .filter(|p| !p.archived && Fold::for_language("fr").contains(&p.title, q))
                    .take(50)
                {
                    items.push(Item {
                        key: p.id.clone(),
                        title: p.display_title().into(),
                        detail: "Mention de page".into(),
                        action: Action::Page(p.id.clone()),
                    });
                }
            }
            Mode::BlockLink { .. } => {
                for p in self.pages.iter().filter(|p| !p.archived) {
                    for b in p
                        .blocks
                        .iter()
                        .filter(|b| {
                            !b.plain_text().is_empty()
                                && (Fold::for_language("fr").contains(&b.plain_text(), q)
                                    || Fold::for_language("fr").contains(&p.title, q))
                        })
                        .take(20)
                    {
                        items.push(Item {
                            key: format!("{}-{}", p.id, b.id),
                            title: b.plain_text(),
                            detail: p.display_title().into(),
                            action: Action::Block(p.id.clone(), b.id.clone()),
                        });
                        if items.len() >= 60 {
                            break;
                        }
                    }
                    if items.len() >= 60 {
                        break;
                    }
                }
            }
            Mode::Embed { base: None, .. } => {
                for p in self.pages.iter().filter(|p| {
                    !p.archived
                        && p.collection.is_some()
                        && Fold::for_language("fr").contains(&p.title, q)
                }) {
                    items.push(Item {
                        key: p.id.clone(),
                        title: p.display_title().into(),
                        detail: "Base existante".into(),
                        action: Action::Base(p.id.clone()),
                    });
                }
            }
            Mode::Embed {
                base: Some(base), ..
            } => {
                if let Some(settings) = self
                    .pages
                    .iter()
                    .find(|p| p.id == *base)
                    .and_then(|p| p.collection.as_ref())
                {
                    items.push(Item {
                        key: "current".into(),
                        title: "Vue courante de la base".into(),
                        detail: "Même source de données".into(),
                        action: Action::View(None),
                    });
                    for v in settings
                        .views
                        .iter()
                        .filter(|v| Fold::for_language("fr").contains(&v.name, q))
                    {
                        items.push(Item {
                            key: v.id.clone(),
                            title: v.name.clone(),
                            detail: "Vue enregistrée".into(),
                            action: Action::View(Some(v.id.clone())),
                        });
                    }
                }
            }
            Mode::Relation { record, column } => {
                if let Some(field) = self.field_column(record, column) {
                    let selected_ids: Vec<_> = self
                        .pages
                        .iter()
                        .find(|p| p.id == *record)
                        .map_or_else(Vec::new, |page| {
                            folio::calculations::related(&self.pages, page, field)
                                .into_iter()
                                .map(|p| p.id.as_str())
                                .collect()
                        });
                    for p in self.pages.iter().filter(|p| {
                        !p.archived
                            && p.parent_id == field.config.target_base
                            && Fold::for_language("fr").contains(&p.title, q)
                    }) {
                        let selected = selected_ids.contains(&p.id.as_str());
                        items.push(Item {
                            key: p.id.clone(),
                            title: p.display_title().into(),
                            detail: if selected {
                                "Liée — cliquer pour retirer"
                            } else {
                                "Cliquer pour lier"
                            }
                            .into(),
                            action: Action::Relation(p.id.clone()),
                        });
                    }
                }
            }
        }
        items.truncate(60);
        items
    }
    pub fn palette_choose(&mut self, action: Action, tx: &mut Tx) {
        let Panel::Palette(form) = self.panel.clone() else {
            return;
        };
        match (form.mode, action) {
            (Mode::Commands, Action::Command(command)) => {
                self.panel = Panel::Closed;
                match command {
                    "new" => {
                        self.create(false);
                        tx.focus("page-title", false);
                    }
                    "base" => {
                        self.create_collection();
                        tx.focus("page-title", false);
                    }
                    "templates" => self.panel = Panel::Templates,
                    "theme" => self.switch_theme(),
                    "backup" => self.backup_dialog(tx),
                    "restore" => self.restore_dialog(tx),
                    "history" => self.open_history(),
                    "file" => self.attachment_dialog(tx),
                    "embed" => {
                        if let Some(id) = self.active.clone() {
                            self.open_palette(Mode::Embed {
                                block: id,
                                base: None,
                            });
                            tx.focus("palette-search", false);
                        }
                    }
                    "block-link" => {
                        if let Some(id) = self.active.clone() {
                            let (start, end) = self.editor_mut(&id).range();
                            self.open_palette(Mode::BlockLink {
                                block: id,
                                start,
                                end,
                            });
                            tx.focus("palette-search", false);
                        }
                    }
                    _ => {}
                }
            }
            (Mode::Commands, Action::Page(id)) => {
                self.select(&id);
                tx.scroll_to_y("document-scroll", 0.0);
            }
            (Mode::Mention { block, start, end }, Action::Page(page)) => {
                self.insert_reference(&block, start, end, &page, None, tx)
            }
            (Mode::BlockLink { block, start, end }, Action::Block(page, target)) => {
                self.insert_reference(&block, start, end, &page, Some(&target), tx)
            }
            (Mode::Embed { block, base: None }, Action::Base(base)) => {
                self.open_palette(Mode::Embed {
                    block,
                    base: Some(base),
                });
                tx.focus("palette-search", false);
            }
            (
                Mode::Embed {
                    block,
                    base: Some(base_id),
                },
                Action::View(view_id),
            ) => {
                let Some(index) = self.blocks.iter().position(|b| b.data.id == block) else {
                    return;
                };
                let mut data = folio::document::NoteBlock::new(Kind::DatabaseView);
                data.embedded = Some(EmbeddedView {
                    base_id,
                    view_id,
                    relation_filter: None,
                });
                data.indent = self.blocks[index].data.indent;
                if self.blocks[index].edit.text.is_empty()
                    || self.blocks[index].edit.text.starts_with('/')
                {
                    data.id = block;
                    self.blocks[index] = data.into();
                } else {
                    let end = self.block_end(index);
                    self.blocks.insert(end, data.into());
                }
                self.panel = Panel::Closed;
                self.commit_blocks(true);
            }
            (Mode::Relation { record, column }, Action::Relation(target)) => {
                let linked = self
                    .pages
                    .iter()
                    .find(|p| p.id == record)
                    .zip(self.field_column(&record, &column))
                    .is_some_and(|(p, c)| {
                        folio::calculations::related(&self.pages, p, c)
                            .iter()
                            .any(|p| p.id == target)
                    });
                self.set_related(&record, &column, &target, !linked);
            }
            _ => {}
        }
    }
    fn insert_reference(
        &mut self,
        id: &str,
        start: usize,
        end: usize,
        page: &str,
        target: Option<&str>,
        tx: &mut Tx,
    ) {
        let Some(p) = self.pages.iter().find(|p| p.id == page) else {
            return;
        };
        let title = target
            .and_then(|id| p.blocks.iter().find(|b| b.id == id).map(|b| b.plain_text()))
            .unwrap_or_else(|| p.display_title().to_string())
            .replace('\n', " ");
        let url = reference_url(page, target);
        let Some(block) = self.blocks.iter_mut().find(|b| b.data.id == id) else {
            return;
        };
        if start > end
            || end > block.edit.text.len()
            || !block.edit.text.is_char_boundary(start)
            || !block.edit.text.is_char_boundary(end)
        {
            return;
        }
        let left = folio::document::rich_slice(&block.edit, 0..start);
        let right = folio::document::rich_slice(&block.edit, end..block.edit.text.len());
        let fragment = TextEdit::from_parts(
            &title,
            vec![MarkRange::new(0..title.len(), Mark::Link(url))],
            vec![rune::Block::Paragraph],
        );
        block.edit = folio::document::join_rich(
            &folio::document::join_rich(&left, &fragment, ""),
            &right,
            "",
        );
        block.edit.set_caret(start + title.len());
        self.active = Some(id.into());
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        self.focus_active(tx);
    }
    pub fn follow_reference(&mut self, url: &str, tx: &mut Tx) {
        if let Some(rest) = url.strip_prefix("https://folio.local/page/") {
            let (page, block) = rest
                .split_once("#block=")
                .map_or((rest, None), |(p, b)| (p, Some(b)));
            if !self.pages.iter().any(|p| p.id == page) {
                self.export_message = Some("Cette page n’existe plus.".into());
                return;
            }
            self.select(page);
            if let Some(block) = block {
                self.goto_block(block, tx);
            } else {
                tx.scroll_to_y("document-scroll", 0.0);
            }
        } else if let Err(e) = tx.open_url(url) {
            self.export_message = Some(e.to_string());
        }
    }
    pub fn goto_block(&mut self, id: &str, tx: &mut Tx) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            self.export_message = Some("Ce bloc n’existe plus.".into());
            return;
        };
        for i in 0..index {
            if self.block_end(i) > index {
                self.blocks[i].data.collapsed = false;
            }
        }
        self.active = Some(id.into());
        self.commit_blocks(false);
        self.focus_active(tx);
        tx.scroll_into_view(format!("block-{id}"));
    }
}
pub fn reference_url(page: &str, block: Option<&str>) -> String {
    format!(
        "https://folio.local/page/{page}{}",
        block.map_or(String::new(), |b| format!("#block={b}"))
    )
}

impl State {
    pub fn set_related(&mut self, record: &str, column: &str, target: &str, linked: bool) {
        let Some(field) = self
            .field_column(record, column)
            .filter(|c| c.kind == ColumnKind::Relation)
            .cloned()
        else {
            return;
        };
        if !self
            .pages
            .iter()
            .any(|p| p.id == target && p.parent_id == field.config.target_base && !p.archived)
        {
            return;
        }
        let (owner, property, member) = if let Some(reverse) = field.config.reverse_of {
            (target, reverse, record)
        } else {
            (record, column.to_string(), target)
        };
        let Some(page) = self.pages.iter_mut().find(|p| p.id == owner) else {
            return;
        };
        let mut ids = match page.properties.custom.get(&property) {
            Some(CellValue::Relation(ids)) => ids.clone(),
            _ => Vec::new(),
        };
        if ids.iter().any(|id| id == member) == linked {
            return;
        }
        if linked {
            ids.push(member.into());
        } else {
            ids.retain(|id| id != member);
        }
        page.properties
            .custom
            .insert(property, CellValue::Relation(ids));
        page.updated_at = folio::model::now();
        self.persist(owner);
    }
}
