use crate::{
    editor::EditorBlock,
    state::{Filter, Panel, Phase, State},
};
use folio::{
    model::{Page, now},
    templates::Template,
};
use rune::TextEdit;
use std::collections::HashSet;
impl State {
    pub fn select(&mut self, id: &str) {
        if self.selected.as_deref() == Some(id) {
            self.panel = Panel::Closed;
            return;
        }
        if !self.pages.iter().any(|p| p.id == id) {
            return;
        }
        if !self.history.is_empty() {
            self.history.truncate(self.history_at + 1);
        }
        self.history.push(id.into());
        self.history_at = self.history.len() - 1;
        self.load_page(id);
    }
    fn load_page(&mut self, id: &str) {
        self.selection.clear();
        self.workspace.selected_records.clear();
        self.workspace.entity_drag = None;
        self.block_drag = crate::block_drag::BlockDrag::default();
        if let Some(page) = self.pages.iter().find(|p| p.id == id) {
            self.title = TextEdit::new(&page.title);
            self.blocks = page.blocks.iter().cloned().map(EditorBlock::from).collect();
            self.active = self.blocks.first().map(|b| b.data.id.clone());
            self.selected = Some(page.id.clone());
            self.panel = Panel::Closed;
            self.undo.clear();
            self.redo.clear();
            self.collection_search = TextEdit::new("");
        }
        for id in self.ancestors(id) {
            self.collapsed.remove(&id);
        }
        self.save_preferences();
    }
    pub fn travel(&mut self, forward: bool) {
        let next = if forward {
            self.history_at
                .checked_add(1)
                .filter(|n| *n < self.history.len())
        } else {
            self.history_at.checked_sub(1)
        };
        if let Some(next) = next {
            self.history_at = next;
            let id = self.history[next].clone();
            self.load_page(&id);
        }
    }
    pub fn create(&mut self, child: bool) {
        if !matches!(self.phase, Phase::Ready) {
            return;
        }
        let parent = if child {
            self.current().filter(|p| !p.archived).map(|p| p.id.clone())
        } else {
            None
        };
        self.add_page(Page::new(parent));
    }
    pub fn create_from_template(&mut self, template: Template) {
        if matches!(self.phase, Phase::Ready) {
            self.add_page(template.create());
        }
    }
    pub(crate) fn add_page(&mut self, page: Page) {
        let mut page = page;
        page.order = self
            .pages
            .iter()
            .map(|p| p.order)
            .max()
            .unwrap_or(0)
            .saturating_add(1024);
        let id = page.id.clone();
        self.pages.push(page);
        self.filter = Filter::All;
        self.search = TextEdit::new("");
        self.select(&id);
        self.persist(&id);
    }
    pub fn duplicate_page(&mut self) {
        let Some(mut copy) = self.current().cloned() else {
            return;
        };
        copy.id = uuid::Uuid::new_v4().to_string();
        copy.title = format!("{} (copie)", copy.display_title());
        copy.favorite = false;
        copy.archived = false;
        copy.updated_at = now();
        for block in &mut copy.blocks {
            block.id = uuid::Uuid::new_v4().to_string();
        }
        self.add_page(copy);
    }
    pub fn ancestors(&self, id: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut seen = HashSet::from([id.to_string()]);
        let mut parent = self
            .pages
            .iter()
            .find(|p| p.id == id)
            .and_then(|p| p.parent_id.clone());
        while let Some(id) = parent {
            if !seen.insert(id.clone()) {
                break;
            }
            result.push(id.clone());
            parent = self
                .pages
                .iter()
                .find(|p| p.id == id)
                .and_then(|p| p.parent_id.clone());
        }
        result.reverse();
        result
    }
    pub fn has_ancestor_cycle(&self, id: &str) -> bool {
        let mut seen = HashSet::new();
        let mut current = Some(id);
        while let Some(id) = current {
            if !seen.insert(id) {
                return true;
            }
            current = self
                .pages
                .iter()
                .find(|p| p.id == id)
                .and_then(|p| p.parent_id.as_deref());
        }
        false
    }
    pub fn can_move_to(&self, target: &str) -> bool {
        self.selected.as_deref() != Some(target)
            && !self
                .ancestors(target)
                .iter()
                .any(|id| Some(id) == self.selected.as_ref())
            && self.pages.iter().any(|p| p.id == target && !p.archived)
    }
    pub fn move_page(&mut self, parent: Option<String>) {
        if parent.as_deref().is_some_and(|id| !self.can_move_to(id)) {
            return;
        }
        let Some(id) = self.selected.clone() else {
            return;
        };
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.parent_id = parent;
            page.updated_at = now();
        }
        self.panel = Panel::Closed;
        for id in self.ancestors(&id) {
            self.collapsed.remove(&id);
        }
        self.persist(&id);
    }
    pub fn appearance(&mut self, emoji: Option<&str>, cover: bool, wide: bool) {
        let Some(id) = self.selected.clone() else {
            return;
        };
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            if let Some(emoji) = emoji {
                page.emoji = emoji.into();
            }
            if cover {
                page.cover = (page.cover + 1) % 4;
            }
            if wide {
                page.wide = !page.wide;
            }
            page.updated_at = now();
        }
        self.panel = Panel::Closed;
        self.persist(&id);
    }
}
