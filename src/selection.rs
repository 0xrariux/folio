use crate::{
    editor::EditorBlock,
    state::{Panel, State},
};
use folio::document::{Kind, NoteBlock};
use rune::{Mods, Tx};
use std::collections::BTreeSet;
#[derive(Default)]
pub struct Selection {
    pub ids: BTreeSet<String>,
    pub anchor: Option<String>,
    pub clipboard: Vec<NoteBlock>,
}
impl Selection {
    pub fn clear(&mut self) {
        self.ids.clear();
        self.anchor = None;
    }
}
impl State {
    pub fn select_block(&mut self, id: &str, mods: Mods) -> bool {
        if !mods.shift && !mods.cmd && !mods.ctrl {
            return false;
        }
        if mods.shift {
            let anchor = self
                .selection
                .anchor
                .as_ref()
                .or(self.active.as_ref())
                .cloned()
                .unwrap_or_else(|| id.into());
            if let (Some(a), Some(b)) = (
                self.blocks.iter().position(|b| b.data.id == anchor),
                self.blocks.iter().position(|b| b.data.id == id),
            ) {
                let visible = self.visible_blocks();
                let order = self.block_order();
                if let (Some(a), Some(b)) = (
                    order.iter().position(|&i| i == a),
                    order.iter().position(|&i| i == b),
                ) {
                    self.selection.ids = order[a.min(b)..=a.max(b)]
                        .iter()
                        .filter(|&&i| visible[i])
                        .map(|&i| self.blocks[i].data.id.clone())
                        .collect();
                }
            }
        } else {
            if self.selection.ids.is_empty() {
                self.selection.anchor = Some(id.into());
            }
            if !self.selection.ids.insert(id.into()) {
                self.selection.ids.remove(id);
            }
        }
        if self.selection.anchor.is_none() {
            self.selection.anchor = Some(id.into());
        }
        self.active = Some(id.into());
        self.panel = Panel::Closed;
        true
    }
    pub fn selected_indices(&self) -> BTreeSet<usize> {
        let mut result = BTreeSet::new();
        for (i, b) in self.blocks.iter().enumerate() {
            if self.selection.ids.contains(&b.data.id) {
                result.extend(i..self.block_end(i));
            }
        }
        result
    }
    pub fn copy_selected(&mut self) -> String {
        let indices = self.selected_indices();
        self.selection.clipboard = indices
            .iter()
            .map(|&i| {
                let mut b = self.blocks[i].data.clone();
                b.html = self.blocks[i].edit.to_html();
                b
            })
            .collect();
        self.selection
            .clipboard
            .iter()
            .map(NoteBlock::plain_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn paste_selected(&mut self, tx: &mut Tx) {
        if self.selection.clipboard.is_empty() {
            return;
        }
        let at = self
            .active
            .as_ref()
            .and_then(|id| self.blocks.iter().position(|b| &b.data.id == id))
            .map_or(self.blocks.len(), |i| self.block_end(i));
        let copies: Vec<_> = self
            .selection
            .clipboard
            .iter()
            .cloned()
            .map(|mut b| {
                b.id = uuid::Uuid::new_v4().to_string();
                EditorBlock::from(b)
            })
            .collect();
        self.selection.ids = copies.iter().map(|b| b.data.id.clone()).collect();
        self.active = copies.first().map(|b| b.data.id.clone());
        self.blocks.splice(at..at, copies);
        self.commit_blocks(true);
        self.focus_active(tx);
    }
    pub fn duplicate_selected(&mut self, tx: &mut Tx) {
        self.copy_selected();
        if let Some(i) = self.selected_indices().last().copied() {
            self.active = Some(self.blocks[i].data.id.clone());
        }
        self.paste_selected(tx);
    }
    pub fn delete_selected(&mut self, tx: &mut Tx) {
        let indices = self.selected_indices();
        if indices.is_empty() {
            return;
        }
        for i in indices.into_iter().rev() {
            self.blocks.remove(i);
        }
        if self.blocks.is_empty() {
            self.blocks
                .push(EditorBlock::from(NoteBlock::new(Kind::Text)));
        }
        self.selection.clear();
        self.active = self.blocks.first().map(|b| b.data.id.clone());
        self.commit_blocks(true);
        self.focus_active(tx);
    }
    pub fn transform_selected(&mut self, kind: Kind) {
        for i in self.selected_indices() {
            self.blocks[i].data.kind = kind;
            self.blocks[i].data.collapsed = false;
            self.blocks[i].data.number_start = None;
        }
        self.commit_blocks(true);
    }
    pub fn move_selected(&mut self, down: bool) {
        let indices = self.selected_indices();
        if indices.is_empty() {
            return;
        }
        let first = *indices.first().unwrap();
        let last = *indices.last().unwrap();
        if (!down && first == 0) || (down && last + 1 == self.blocks.len()) {
            return;
        }
        // Treat selected blocks as one group, preserving their order and every subtree.
        let slot = if down {
            self.block_end(last + 1)
        } else {
            let level = self.blocks[first].data.indent;
            (0..first)
                .rev()
                .find(|&i| self.blocks[i].data.indent <= level)
                .unwrap_or(0)
        };
        self.reorder_selection(slot);
    }
    pub fn reorder_selection(&mut self, slot: usize) {
        let indices = self.selected_indices();
        if indices.is_empty() || indices.contains(&slot) {
            return;
        }
        let insert = slot.saturating_sub(indices.iter().filter(|&&i| i < slot).count());
        let mut selected = Vec::new();
        let mut remaining = Vec::new();
        for (i, b) in self.blocks.drain(..).enumerate() {
            if indices.contains(&i) {
                selected.push(b);
            } else {
                remaining.push(b);
            }
        }
        remaining.splice(insert..insert, selected);
        self.blocks = remaining;
        self.commit_blocks(true);
    }
}

impl State {
    pub fn handle_selection_key(&mut self, event: &rune::KeyEvent, tx: &mut Tx) -> bool {
        if self.selection.ids.is_empty() {
            return false;
        }
        match event.key {
            rune::Key::Escape => {
                self.selection.clear();
                true
            }
            rune::Key::Backspace | rune::Key::Delete => {
                self.delete_selected(tx);
                true
            }
            rune::Key::Command('a') => {
                self.selection.ids = self
                    .blocks
                    .iter()
                    .zip(self.visible_blocks())
                    .filter(|(_, visible)| *visible)
                    .map(|(b, _)| b.data.id.clone())
                    .collect();
                true
            }
            rune::Key::Command('c') => {
                let plain = self.copy_selected();
                tx.copy_rich(
                    rune::ClipboardContent::new(plain)
                        .with_html(folio::outline::html(&self.selection.clipboard)),
                );
                true
            }
            rune::Key::Command('d') => {
                self.duplicate_selected(tx);
                true
            }
            rune::Key::Command('v') => {
                let page = self.selected.clone();
                tx.read_clipboard(move |s: &mut State, content, tx| {
                    if s.selected != page {
                        return;
                    }
                    let saved = s
                        .selection
                        .clipboard
                        .iter()
                        .map(NoteBlock::plain_text)
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !s.selection.clipboard.is_empty() && content.text() == saved {
                        s.paste_selected(tx);
                    } else if let Some(id) = s.active.clone() {
                        s.selection.clear();
                        if !s.paste_blocks(&id, content, tx) {
                            s.editor_mut(&id).insert(content.text());
                            s.block_changed(&id);
                        }
                    }
                });
                true
            }
            rune::Key::Up | rune::Key::Down if event.mods.alt && event.mods.shift => {
                self.move_selected(event.key == rune::Key::Down);
                true
            }
            _ => false,
        }
    }
}
