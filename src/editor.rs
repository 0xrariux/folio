use folio::{
    document::{Kind, NoteBlock, join_rich, rich_slice},
    model::now,
};
use rune::{Key, KeyEvent, Mark, TextEdit, Tx};

use crate::state::{Panel, State};

pub struct EditorBlock {
    pub data: NoteBlock,
    pub edit: TextEdit,
}
impl From<NoteBlock> for EditorBlock {
    fn from(data: NoteBlock) -> Self {
        Self {
            edit: TextEdit::from_html(&data.html),
            data,
        }
    }
}

#[derive(Clone)]
pub struct Snapshot {
    pub blocks: Vec<NoteBlock>,
    pub active: Option<String>,
}

pub fn input_key(id: &str) -> String {
    format!("block-input-{id}")
}

impl State {
    pub fn editor_mut(&mut self, id: &str) -> &mut TextEdit {
        // Rune also reads this while syncing focus; gestures choose the active block.
        &mut self
            .blocks
            .iter_mut()
            .find(|b| b.data.id == id)
            .expect("a mounted block owns its editor")
            .edit
    }

    pub fn focus_active(&mut self, tx: &mut Tx) {
        let visible = self.visible_blocks();
        if let Some(index) = self
            .blocks
            .iter()
            .position(|b| Some(&b.data.id) == self.active.as_ref())
            && !visible[index]
        {
            self.active = (0..index)
                .rev()
                .find(|&i| visible[i])
                .map(|i| self.blocks[i].data.id.clone());
        }
        if let Some(id) = &self.active {
            let kind = self
                .blocks
                .iter()
                .find(|b| b.data.id == *id)
                .map(|b| b.data.kind);
            tx.focus(
                match kind {
                    Some(Kind::PageLink) => format!("page-link-{id}"),
                    Some(
                        Kind::Divider
                        | Kind::DatabaseView
                        | Kind::Image
                        | Kind::Attachment
                        | Kind::TableOfContents,
                    ) => format!("handle-{id}"),
                    _ => input_key(id),
                },
                false,
            );
            tx.scroll_into_view(format!("block-{id}"));
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            blocks: self.current().map(|p| p.blocks.clone()).unwrap_or_default(),
            active: self.active.clone(),
        }
    }

    fn bound_history(history: &mut Vec<Snapshot>) {
        let mut bytes: usize = history
            .iter()
            .flat_map(|s| &s.blocks)
            .map(|b| b.html.len() + b.id.len())
            .sum();
        while history.len() > 1 && (history.len() > 80 || bytes > 16 * 1024 * 1024) {
            let removed = history.remove(0);
            bytes = bytes.saturating_sub(
                removed
                    .blocks
                    .iter()
                    .map(|b| b.html.len() + b.id.len())
                    .sum(),
            );
        }
    }

    pub fn commit_blocks(&mut self, record: bool) {
        if self.settings.locked || self.current().is_some_and(|p| p.protected.is_some()) {
            return;
        }
        let Some(id) = self.selected.clone() else {
            return;
        };
        if record {
            self.normalize_blocks();
        }
        for block in &mut self.blocks {
            block.data.html = block.edit.to_html();
        }
        let next: Vec<_> = self.blocks.iter().map(|b| b.data.clone()).collect();
        if self.current().is_none_or(|p| p.blocks == next) {
            return;
        }
        if record {
            self.undo.push(self.snapshot());
            Self::bound_history(&mut self.undo);
            self.redo.clear();
        }
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.blocks = next;
            page.sync_document();
            page.updated_at = now();
        }
        self.persist(&id);
    }

    pub fn block_changed(&mut self, id: &str) {
        self.apply_list_shortcut(id);
        self.commit_blocks(true);
        let query = self
            .blocks
            .iter()
            .find(|b| b.data.id == id)
            .map(|b| b.edit.text.as_str())
            .unwrap_or("");
        let code = self
            .blocks
            .iter()
            .any(|b| b.data.id == id && b.data.kind == Kind::Code);
        if !code && query.starts_with('/') && !query.contains('\n') && query.len() < 60 {
            self.panel = Panel::Slash {
                id: id.into(),
                selected: 0,
                typed: true,
            };
        } else if matches!(&self.panel, Panel::Slash { id: panel_id, .. } if panel_id == id) {
            self.panel = Panel::Closed;
        }
    }

    pub fn mark(&mut self, mark: Mark) {
        if let Some(block) = self
            .blocks
            .iter_mut()
            .find(|b| Some(&b.data.id) == self.active.as_ref())
        {
            block.edit.toggle_mark(mark);
        }
        self.commit_blocks(true);
    }

    pub fn add_block(&mut self, after: Option<&str>, kind: Kind, slash: bool, tx: &mut Tx) {
        let mut block = EditorBlock::from(NoteBlock::new(kind));
        if slash {
            block.edit.insert("/");
        }
        let id = block.data.id.clone();
        let index = after
            .and_then(|id| self.blocks.iter().position(|b| b.data.id == id))
            .map_or(self.blocks.len(), |i| {
                block.data.indent = self.blocks[i].data.indent;
                self.block_end(i)
            });
        self.blocks.insert(index, block);
        self.active = Some(id.clone());
        self.commit_blocks(true);
        self.panel = if slash {
            Panel::Slash {
                id,
                selected: 0,
                typed: true,
            }
        } else {
            Panel::Closed
        };
        self.focus_active(tx);
    }

    pub fn slash_options(&self) -> Vec<Kind> {
        let query = match &self.panel {
            Panel::Slash {
                id, typed: true, ..
            } => self
                .blocks
                .iter()
                .find(|b| b.data.id == *id)
                .map(|b| b.edit.text.strip_prefix('/').unwrap_or(""))
                .unwrap_or(""),
            _ => "",
        };
        Kind::ALL
            .into_iter()
            .filter(|kind| kind.matches(query.trim()))
            .collect()
    }

    pub fn choose_kind(&mut self, kind: Kind, tx: &mut Tx) {
        let Panel::Slash { id, typed, .. } = self.panel.clone() else {
            return;
        };
        if kind == Kind::DatabaseView {
            self.open_palette(crate::command_palette::Mode::Embed {
                block: id,
                base: None,
            });
            tx.focus("palette-search", false);
            return;
        }
        if matches!(kind, Kind::Image | Kind::Attachment) {
            if !typed
                && self
                    .blocks
                    .iter()
                    .any(|b| b.data.id == id && !b.edit.text.is_empty())
            {
                self.add_block(Some(&id), kind, false, tx);
            } else if let Some(b) = self.blocks.iter_mut().find(|b| b.data.id == id) {
                b.data.kind = kind;
                if typed {
                    b.edit = TextEdit::new("").rich();
                }
                self.commit_blocks(true);
            }
            self.panel = Panel::Closed;
            self.attachment_dialog(tx);
            return;
        }
        if kind == Kind::PageLink {
            self.open_link_picker(id, typed);
            tx.focus("link-search", false);
            return;
        }
        if !typed
            && matches!(kind, Kind::Divider | Kind::TableOfContents)
            && self
                .blocks
                .iter()
                .any(|b| b.data.id == id && !b.edit.text.is_empty())
        {
            self.add_block(Some(&id), kind, false, tx);
            return;
        }
        if typed && let Some(block) = self.blocks.iter_mut().find(|b| b.data.id == id) {
            block.edit = TextEdit::new("").rich();
        }
        self.set_list_kind(&id, kind, tx);
    }

    pub fn split_block(&mut self, id: &str, tx: &mut Tx) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        let block = &self.blocks[index];
        let (a, b) = block.edit.range();
        if block.data.kind == Kind::Text
            && block.edit.text.trim().is_empty()
            && self.block_end(index) == index + 1
        {
            return;
        }
        if block.edit.text.is_empty()
            && (block.data.kind.is_list() || block.data.kind == Kind::Quote)
        {
            if block.data.indent > 0 {
                self.indent_block(id, true, tx);
                return;
            }
            self.blocks[index].data.kind = Kind::Text;
            self.blocks[index].data.checked = false;
            self.blocks[index].data.number_start = None;
            self.blocks[index].data.collapsed = false;
            self.commit_blocks(true);
            return;
        }
        if block.data.kind.is_toggle() {
            let (a, b) = block.edit.range();
            let right = rich_slice(&block.edit, b..block.edit.text.len());
            let left = rich_slice(&block.edit, 0..a);
            if block.data.indent < folio::outline::MAX_INDENT {
                let mut child = NoteBlock::new(Kind::Text);
                child.indent = block.data.indent + 1;
                let mut child = EditorBlock::from(child);
                child.edit = right;
                child.edit.set_caret(0);
                self.active = Some(child.data.id.clone());
                self.blocks[index].edit = left;
                self.blocks[index].data.collapsed = false;
                self.blocks.insert(index + 1, child);
                self.commit_blocks(true);
                self.focus_active(tx);
                return;
            }
        }
        let left = rich_slice(&block.edit, 0..a);
        let right = rich_slice(&block.edit, b..block.edit.text.len());
        let kind = match block.data.kind {
            Kind::Heading1 | Kind::Heading2 | Kind::Heading3 | Kind::Callout | Kind::Divider => {
                Kind::Text
            }
            kind => kind,
        };
        let mut next = EditorBlock::from(NoteBlock::new(kind));
        next.data.indent = block.data.indent;
        next.edit = right;
        next.edit.set_caret(0);
        self.blocks[index].edit = left;
        self.active = Some(next.data.id.clone());
        let end = self.block_end(index);
        self.blocks.insert(end, next);
        self.commit_blocks(true);
        self.focus_active(tx);
    }

    pub fn merge_previous(&mut self, id: &str, tx: &mut Tx) -> bool {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return false;
        };
        if self.blocks[index].edit.range() != (0, 0) {
            return false;
        }
        if self.blocks[index].data.indent > 0 {
            return self.indent_block(id, true, tx);
        }
        if self.blocks[index].data.kind.is_list() {
            self.set_list_kind(id, Kind::Text, tx);
            return true;
        }
        let sections = folio::outline::task_sections(self.blocks.iter().map(|b| &b.data));
        let order: Vec<_> = sections.order().collect();
        let Some(position) = order.iter().position(|&i| i == index) else {
            return false;
        };
        let visible = self.visible_blocks();
        let Some(previous) = order[..position]
            .iter()
            .rev()
            .copied()
            .find(|&i| visible[i])
        else {
            return false;
        };
        // A merge must not swallow a parent or a hidden child into another block.
        if self.block_end(index) > index + 1
            || self.block_end(previous) > previous + 1
            || self.blocks[previous].data.indent != self.blocks[index].data.indent
            || sections.completed.contains(&previous) != sections.completed.contains(&index)
        {
            return false;
        }
        if self.blocks[previous].data.kind == Kind::Divider {
            self.blocks.remove(previous);
            self.commit_blocks(true);
            self.focus_active(tx);
            return true;
        }
        let caret = self.blocks[previous].edit.text.len();
        let joined = join_rich(&self.blocks[previous].edit, &self.blocks[index].edit, "");
        self.blocks[previous].edit = joined;
        self.blocks[previous].edit.set_caret(caret);
        self.active = Some(self.blocks[previous].data.id.clone());
        self.blocks.remove(index);
        self.commit_blocks(true);
        self.focus_active(tx);
        true
    }

    pub fn move_block(&mut self, id: &str, down: bool) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        let end = self.block_end(index);
        let Some(sibling) = self.block_sibling(index, down) else {
            return;
        };
        if down {
            let next_end = self.block_end(sibling);
            self.blocks[index..next_end].rotate_left(end - index);
        } else {
            self.blocks[sibling..end].rotate_right(end - index);
        }
        self.active = Some(id.into());
        self.commit_blocks(true);
        self.panel = Panel::Closed;
    }

    pub fn duplicate_block(&mut self, id: &str, tx: &mut Tx) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        let end = self.block_end(index);
        let copies: Vec<_> = self.blocks[index..end]
            .iter()
            .map(|b| {
                let mut data = b.data.clone();
                data.html = b.edit.to_html();
                data.id = uuid::Uuid::new_v4().to_string();
                EditorBlock::from(data)
            })
            .collect();
        self.active = copies.first().map(|b| b.data.id.clone());
        self.blocks.splice(end..end, copies);
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        self.focus_active(tx);
    }

    pub fn remove_block(&mut self, id: &str, tx: &mut Tx) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        let end = self.block_end(index);
        self.blocks.drain(index..end);
        if self.blocks.is_empty() {
            self.blocks
                .push(EditorBlock::from(NoteBlock::new(Kind::Text)));
        }
        self.active = Some(
            self.blocks[index.saturating_sub(1).min(self.blocks.len() - 1)]
                .data
                .id
                .clone(),
        );
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        self.focus_active(tx);
    }

    pub fn check_task(&mut self, id: &str) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        let end = self.block_end(index);
        let checked = !self.blocks[index].data.checked;
        self.blocks[index].data.checked = checked;
        for child in &mut self.blocks[index + 1..end] {
            if child.data.kind == Kind::Todo {
                child.data.checked = checked;
            }
        }
        self.commit_blocks(true);
    }

    pub fn undo_document(&mut self, redo: bool, tx: &mut Tx) {
        self.selection.clear();
        let snapshot = if redo {
            self.redo.pop()
        } else {
            self.undo.pop()
        };
        let Some(snapshot) = snapshot else {
            return;
        };
        let current = self.snapshot();
        if redo {
            self.undo.push(current);
        } else {
            self.redo.push(current);
        }
        self.blocks = snapshot.blocks.into_iter().map(EditorBlock::from).collect();
        self.active = snapshot
            .active
            .filter(|id| self.blocks.iter().any(|b| b.data.id == *id))
            .or_else(|| self.blocks.first().map(|b| b.data.id.clone()));
        self.panel = Panel::Closed;
        self.commit_blocks(false);
        self.focus_active(tx);
    }

    pub fn block_key(&mut self, id: &str, event: &KeyEvent, tx: &mut Tx) -> bool {
        if self.handle_selection_key(event, tx) {
            return true;
        }
        if matches!(&self.panel, Panel::Slash { id: menu_id, .. } if menu_id != id) {
            self.panel = Panel::Closed;
        }
        self.active = Some(id.into());
        if self
            .blocks
            .iter()
            .any(|b| b.data.id == id && b.edit.is_composing())
        {
            return false;
        }
        if matches!(&event.key, Key::Text(value) if value == "@")
            && self
                .blocks
                .iter()
                .any(|b| b.data.id == id && b.data.kind != Kind::Code)
        {
            let (start, _) = self.editor_mut(id).range();
            self.editor_mut(id).insert("@");
            self.block_changed(id);
            self.open_palette(crate::command_palette::Mode::Mention {
                block: id.into(),
                start,
                end: start + 1,
            });
            tx.focus("palette-search", false);
            return true;
        }
        if let Panel::Slash { selected, .. } = self.panel {
            let options = self.slash_options();
            match event.key {
                Key::Down | Key::Up => {
                    let count = options.len().max(1);
                    if let Panel::Slash {
                        selected: index, ..
                    } = &mut self.panel
                    {
                        *index = if event.key == Key::Down {
                            (selected + 1) % count
                        } else {
                            (selected + count - 1) % count
                        };
                        if let Some(kind) = options.get(*index) {
                            tx.scroll_into_view(format!("slash-{kind:?}"));
                        }
                    }
                    return true;
                }
                Key::Enter => {
                    if let Some(kind) = options.get(selected).copied() {
                        self.choose_kind(kind, tx);
                    }
                    return true;
                }
                Key::Escape => {
                    self.panel = Panel::Closed;
                    return true;
                }
                _ => {}
            }
        }
        match event.key {
            Key::Up | Key::Down
                if !event.mods.cmd && !event.mods.alt && !event.mods.ctrl && !event.mods.shift =>
            {
                self.focus_neighbor(id, event.key == Key::Down, tx)
            }
            Key::Tab if !event.mods.cmd && !event.mods.alt && !event.mods.ctrl => {
                let structural = self.blocks.iter().any(|b| {
                    b.data.id == id
                        && (b.data.kind.is_list() || b.data.indent > 0 || b.data.kind == Kind::Text)
                });
                if structural {
                    self.indent_block(id, event.mods.shift, tx);
                }
                structural
            }
            Key::Up | Key::Down if event.mods.alt && event.mods.shift => {
                self.move_block(id, event.key == Key::Down);
                self.focus_active(tx);
                true
            }
            Key::Command('7' | '8' | '9') if event.mods.shift => {
                let kind = match event.key {
                    Key::Command('7') => Kind::Numbered,
                    Key::Command('8') => Kind::Bullet,
                    _ => Kind::Toggle,
                };
                self.set_list_kind(id, kind, tx);
                true
            }
            Key::Enter if event.mods.cmd => {
                if self
                    .blocks
                    .iter()
                    .any(|b| b.data.id == id && b.data.kind.is_toggle())
                {
                    self.toggle_block(id, tx);
                    true
                } else if self
                    .blocks
                    .iter()
                    .any(|b| b.data.id == id && b.data.kind == Kind::Todo)
                {
                    self.check_task(id);
                    true
                } else {
                    false
                }
            }
            Key::Command('z') => {
                self.undo_document(event.mods.shift, tx);
                true
            }
            Key::Command('y') => {
                self.undo_document(true, tx);
                true
            }
            Key::Command('d') => {
                self.duplicate_block(id, tx);
                true
            }
            Key::Enter
                if !event.mods.shift && !event.mods.alt && !event.mods.cmd && !event.mods.ctrl =>
            {
                if self
                    .blocks
                    .iter()
                    .any(|b| b.data.id == id && b.data.kind == Kind::Code)
                {
                    return false;
                }
                self.split_block(id, tx);
                true
            }
            Key::Backspace => self.merge_previous(id, tx),
            _ => false,
        }
    }
}
