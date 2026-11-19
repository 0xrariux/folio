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
        if let Some(previous) = self
            .selected
            .clone()
            .filter(|previous| previous != id && self.settings.unlocked.contains(previous))
        {
            if let Some(locked) = self.settings.locked_pages.remove(&previous)
                && let Some(page) = self.pages.iter_mut().find(|p| p.id == previous)
            {
                *page = locked;
            }
            self.settings.unlocked.remove(&previous);
            if let Some(worker) = &self.worker {
                let _ = worker.security(folio::vault::Action::LockPage(previous));
            }
        }
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
        if self
            .selected
            .as_ref()
            .is_some_and(|id| self.protected_page(id))
        {
            self.export_message = Some(
                "Duplicating a protected page is disabled to avoid creating a plain-text copy."
                    .into(),
            );
            return;
        }
        let Some(mut copy) = self.current().cloned() else {
            return;
        };
        copy.id = uuid::Uuid::new_v4().to_string();
        copy.title = format!("{} (copy)", copy.display_title());
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

    pub fn set_icon_color(&mut self, color: folio::icons::IconColor) {
        if self.settings.locked || self.current().is_none_or(|p| p.protected.is_some()) {
            return;
        }
        let Some(page) = self
            .pages
            .iter_mut()
            .find(|p| Some(&p.id) == self.selected.as_ref())
        else {
            return;
        };
        page.icon_color = color;
        page.updated_at = now();
        let id = page.id.clone();
        self.persist(&id);
    }
}

impl State {
    pub fn confirm_delete(&mut self) {
        if self.workspace.busy || self.settings.busy || self.settings.locked {
            return;
        }
        let Panel::ConfirmDelete(id) = self.panel.clone() else {
            return;
        };
        let Some(page) = self.pages.iter().find(|p| p.id == id) else {
            return;
        };
        if page.protected.is_some() {
            self.export_message = Some("Unlock the page before deleting it.".into());
            self.panel = Panel::Closed;
            return;
        }
        let parent = page.parent_id.clone();
        if self.demo {
            self.apply_deleted(&id, parent);
        } else {
            self.edit();
            self.workspace.busy = true;
            self.workspace_request(crate::workspace_worker::Request::DeletePage(id));
        }
    }
    pub fn apply_deleted(&mut self, id: &str, parent: Option<String>) {
        self.pages.retain(|p| p.id != id);
        for page in &mut self.pages {
            if page.parent_id.as_deref() == Some(id) {
                page.parent_id = parent.clone();
            }
        }
        self.pending.remove(id);
        self.settings.unlocked.remove(id);
        self.settings.locked_pages.remove(id);
        self.history.retain(|p| p != id);
        self.history_at = self.history_at.min(self.history.len().saturating_sub(1));
        self.workspace.busy = false;
        self.panel = Panel::Closed;
        if self.selected.as_deref() == Some(id) {
            self.selected = None;
            self.title = TextEdit::new("");
            self.blocks.clear();
            self.active = None;
            self.undo.clear();
            self.redo.clear();
            self.selection.clear();
            self.workspace.versions.clear();
            if let Some(next) = self
                .pages
                .iter()
                .find(|p| !p.archived)
                .map(|p| p.id.clone())
            {
                self.select(&next);
            }
        }
        self.export_message =
            Some("Page permanently deleted from the notebook. Earlier backups are kept.".into());
    }
}

impl State {
    pub(crate) fn can_customize_sidebar(&self) -> bool {
        !self.workspace.busy
            && !self.settings.busy
            && !self.settings.locked
            && (self.demo || self.workspace.preferences_loaded)
    }

    pub fn move_sidebar_shortcut(&mut self, shortcut: folio::workspace::SidebarShortcut, up: bool) {
        if !self.can_customize_sidebar() {
            return;
        }
        let shortcuts = &mut self.workspace.preferences.sidebar_shortcuts;
        let Some(from) = shortcuts.iter().position(|item| *item == shortcut) else {
            return;
        };
        let to = if up {
            from.checked_sub(1)
        } else {
            from.checked_add(1)
        };
        let Some(to) = to.filter(|to| *to < shortcuts.len()) else {
            return;
        };
        shortcuts.swap(from, to);
        self.save_preferences();
    }

    pub fn show_sidebar_shortcut(
        &mut self,
        shortcut: folio::workspace::SidebarShortcut,
        show: bool,
    ) {
        use folio::workspace::SidebarShortcut;
        if !self.can_customize_sidebar() {
            return;
        }
        let shortcuts = &mut self.workspace.preferences.sidebar_shortcuts;
        if show {
            if shortcuts.contains(&shortcut) {
                return;
            }
            shortcuts.push(shortcut);
        } else {
            let Some(index) = shortcuts.iter().position(|item| *item == shortcut) else {
                return;
            };
            shortcuts.remove(index);
            if matches!(
                (shortcut, self.filter),
                (SidebarShortcut::Favorites, Filter::Favorites)
                    | (SidebarShortcut::Archives, Filter::Archive)
            ) {
                self.filter = Filter::All;
            }
        }
        self.save_preferences();
    }

    pub fn reset_sidebar_shortcuts(&mut self) {
        if !self.can_customize_sidebar() {
            return;
        }
        self.workspace.preferences.sidebar_shortcuts =
            folio::workspace::SidebarShortcut::defaults();
        self.save_preferences();
    }
}

impl State {
    pub fn drop_sidebar_shortcut(&mut self, source: &str, target: &str) {
        use folio::workspace::SidebarShortcut;
        if !self.can_customize_sidebar() {
            return;
        }
        let Some(source) = source
            .strip_prefix("shortcut:")
            .and_then(SidebarShortcut::from_key)
        else {
            return;
        };
        let (target, after) = if let Some(key) = target.strip_prefix("shortcut-before-") {
            (key, false)
        } else if let Some(key) = target.strip_prefix("shortcut-after-") {
            (key, true)
        } else {
            return;
        };
        let Some(target) = SidebarShortcut::from_key(target).filter(|target| *target != source)
        else {
            return;
        };
        let shortcuts = &mut self.workspace.preferences.sidebar_shortcuts;
        let Some(from) = shortcuts.iter().position(|item| *item == source) else {
            return;
        };
        if !shortcuts.contains(&target) {
            return;
        }
        shortcuts.remove(from);
        let to = shortcuts.iter().position(|item| *item == target).unwrap() + usize::from(after);
        shortcuts.insert(to, source);
        if from != to {
            self.save_preferences();
        }
    }
}
