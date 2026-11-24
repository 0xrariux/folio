use crate::{
    editor::{EditorBlock, input_key},
    state::{Panel, State},
};
use folio::{
    document::{Kind, NoteBlock, join_rich, rich_slice},
    outline::{self, MAX_INDENT},
};
use rune::{ClipboardContent, TextEdit, Tx};

impl State {
    /// Compact only empty paragraph leaves; never discard a heading, task, asset
    /// or an empty parent that owns content. Keep the active editor in each run.
    pub fn normalize_blocks(&mut self) {
        let ends = outline::subtree_ends(self.blocks.iter().map(|b| &b.data));
        let mut keep = vec![true; self.blocks.len()];
        let mut previous: Option<usize> = None;
        for (i, block) in self.blocks.iter().enumerate() {
            let blank = block.data.kind == Kind::Text
                && block.edit.text.trim().is_empty()
                && !block.edit.is_composing()
                && block.data.target_id.is_none()
                && block.data.asset.is_none()
                && block.data.embedded.is_none()
                && ends[i] == i + 1;
            if !blank {
                previous = None;
                continue;
            }
            if let Some(p) = previous
                && self.blocks[p].data.indent == block.data.indent
            {
                if self.active.as_deref() == Some(&block.data.id) {
                    keep[p] = false;
                } else {
                    keep[i] = false;
                    continue;
                }
            }
            previous = Some(i);
        }
        let mut index = 0;
        self.blocks.retain(|_| {
            let retain = keep[index];
            index += 1;
            retain
        });
        self.selection
            .ids
            .retain(|id| self.blocks.iter().any(|b| b.data.id == *id));
        if self
            .selection
            .anchor
            .as_ref()
            .is_some_and(|id| !self.blocks.iter().any(|b| b.data.id == *id))
        {
            self.selection.anchor = self.active.clone();
        }
        let progress = outline::task_progress(self.blocks.iter().map(|b| &b.data));
        for (block, (done, total)) in self.blocks.iter_mut().zip(progress) {
            if total > 0 {
                block.data.checked = done == total;
            }
        }
    }

    pub fn focus_neighbor(&mut self, id: &str, down: bool, tx: &mut Tx) -> bool {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return false;
        };
        let edit = &self.blocks[index].edit;
        let boundary = if down { edit.text.len() } else { 0 };
        if edit.range() != (boundary, boundary) {
            return false;
        }
        let visible = self.visible_blocks();
        let order = self.block_order();
        let Some(position) = order.iter().position(|&i| i == index) else {
            return false;
        };
        let next = if down {
            order[position + 1..].iter().copied().find(|&i| visible[i])
        } else {
            order[..position]
                .iter()
                .rev()
                .copied()
                .find(|&i| visible[i])
        };
        let Some(next) = next else {
            return false;
        };
        let caret = if down {
            0
        } else {
            self.blocks[next].edit.text.len()
        };
        self.blocks[next].edit.set_caret(caret);
        self.active = Some(self.blocks[next].data.id.clone());
        self.panel = Panel::Closed;
        self.focus_active(tx);
        true
    }

    pub fn block_end(&self, index: usize) -> usize {
        let level = self.blocks[index].data.indent;
        (index + 1..self.blocks.len())
            .find(|&i| self.blocks[i].data.indent <= level)
            .unwrap_or(self.blocks.len())
    }

    /// A sibling in the same displayed section. Completed root entities may sit
    /// between active ones in storage, but must not become their hidden neighbours.
    pub fn block_sibling(&self, index: usize, down: bool) -> Option<usize> {
        let sections = outline::task_sections(self.blocks.iter().map(|b| &b.data));
        let completed = sections.completed.binary_search(&index).is_ok();
        let level = self.blocks[index].data.indent;
        let mut next = if down { self.block_end(index) } else { index };
        loop {
            let candidate = if down {
                self.blocks.get(next)?;
                next
            } else {
                (0..next)
                    .rev()
                    .find(|&i| self.blocks[i].data.indent <= level)?
            };
            if self.blocks[candidate].data.indent != level {
                return None;
            }
            if sections.completed.binary_search(&candidate).is_ok() == completed {
                return Some(candidate);
            }
            next = if down {
                self.block_end(candidate)
            } else {
                candidate
            };
        }
    }

    pub fn toggle_block(&mut self, id: &str, tx: &mut Tx) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        if !self.blocks[index].data.kind.is_toggle() && self.block_end(index) == index + 1 {
            return;
        }
        self.blocks[index].data.collapsed = !self.blocks[index].data.collapsed;
        self.active = Some(id.into());
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        tx.focus(format!("toggle-{id}"), false);
    }

    pub fn fold_all(&mut self, collapsed: bool, tx: &mut Tx) {
        let ends = outline::subtree_ends(self.blocks.iter().map(|b| &b.data));
        for (i, b) in self.blocks.iter_mut().enumerate() {
            if b.data.kind.is_toggle() || ends[i] > i + 1 {
                b.data.collapsed = collapsed;
            }
        }
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        self.focus_active(tx);
    }

    pub fn add_child(&mut self, id: &str, tx: &mut Tx) {
        let kind = self
            .blocks
            .iter()
            .find(|b| b.data.id == id)
            .map_or(Kind::Text, |b| {
                if b.data.kind == Kind::Todo {
                    Kind::Todo
                } else {
                    Kind::Text
                }
            });
        self.add_child_kind(id, kind, tx);
    }

    pub fn add_task(&mut self, tx: &mut Tx) {
        let empty = self
            .blocks
            .last_mut()
            .filter(|b| {
                b.data.kind == Kind::Text
                    && b.data.indent == 0
                    && b.edit.text.trim().is_empty()
                    && !b.edit.is_composing()
                    && b.data.target_id.is_none()
                    && b.data.asset.is_none()
                    && b.data.embedded.is_none()
            })
            .map(|b| {
                b.data.checked = false;
                b.data.id.clone()
            });
        if let Some(id) = empty {
            self.set_list_kind(&id, Kind::Todo, tx);
        } else {
            self.add_block(None, Kind::Todo, false, tx);
        }
    }

    pub fn add_child_kind(&mut self, id: &str, kind: Kind, tx: &mut Tx) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        if self.blocks[index].data.indent >= MAX_INDENT {
            return;
        }
        let mut child = NoteBlock::new(kind);
        child.indent = self.blocks[index].data.indent + 1;
        self.blocks[index].data.collapsed = false;
        self.active = Some(child.id.clone());
        let end = self.block_end(index);
        self.blocks.insert(end, EditorBlock::from(child));
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        self.focus_active(tx);
    }

    pub fn indent_block(&mut self, id: &str, outdent: bool, tx: &mut Tx) -> bool {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return false;
        };
        let level = self.blocks[index].data.indent;
        let end = self.block_end(index);
        if outdent {
            if level == 0 {
                return false;
            }
            // Insert after the parent's remaining children instead of adopting them.
            let parent = (0..index)
                .rev()
                .find(|&i| self.blocks[i].data.indent < level);
            let destination = parent.map_or(end, |p| self.block_end(p));
            for b in &mut self.blocks[index..end] {
                b.data.indent = b.data.indent.saturating_sub(1);
            }
            self.blocks[index..destination].rotate_left(end - index);
        } else {
            if index == 0
                || level >= MAX_INDENT
                || self.blocks[index..end]
                    .iter()
                    .any(|b| b.data.indent >= MAX_INDENT)
            {
                return false;
            }
            let Some(previous) = self.block_sibling(index, false) else {
                return false;
            };
            let destination = self.block_end(previous);
            self.blocks[previous].data.collapsed = false;
            for b in &mut self.blocks[index..end] {
                b.data.indent += 1;
            }
            self.blocks[destination..end].rotate_right(end - index);
        }
        self.active = Some(id.into());
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        self.focus_active(tx);
        true
    }

    pub fn set_list_kind(&mut self, id: &str, kind: Kind, tx: &mut Tx) {
        if let Some(b) = self.blocks.iter_mut().find(|b| b.data.id == id) {
            b.data.kind = kind;
            b.data.target_id = None;
            b.data.number_start = None;
            b.data.collapsed = false;
        }
        if kind == Kind::Todo {
            self.convert_subtasks(id);
        }
        self.active = Some(id.into());
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        self.focus_active(tx);
    }

    fn convert_subtasks(&mut self, id: &str) {
        let Some(index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        let end = self.block_end(index);
        for child in &mut self.blocks[index + 1..end] {
            // References, media and code retain their original behavior and data.
            if !matches!(
                child.data.kind,
                Kind::Text
                    | Kind::Heading1
                    | Kind::Heading2
                    | Kind::Heading3
                    | Kind::Bullet
                    | Kind::Numbered
                    | Kind::Todo
                    | Kind::Toggle
                    | Kind::ToggleHeading1
                    | Kind::ToggleHeading2
                    | Kind::ToggleHeading3
                    | Kind::Quote
                    | Kind::Callout
            ) || child.data.asset.is_some()
                || child.data.embedded.is_some()
                || child.data.target_id.is_some()
            {
                continue;
            }
            if child.data.kind != Kind::Todo {
                child.data.checked = false;
            }
            child.data.kind = Kind::Todo;
            child.data.number_start = None;
        }
    }

    pub fn restart_numbering(&mut self, id: &str, restart: bool) {
        if let Some(b) = self
            .blocks
            .iter_mut()
            .find(|b| b.data.id == id && b.data.kind == Kind::Numbered)
        {
            b.data.number_start = restart.then_some(1);
        }
        self.panel = Panel::Closed;
        self.commit_blocks(true);
    }

    /// The current contiguous list, including its nested lists and tasks.
    pub fn task_scope(&self, id: &str) -> std::ops::Range<usize> {
        let Some(mut index) = self.blocks.iter().position(|b| b.data.id == id) else {
            return 0..0;
        };
        while self.blocks[index].data.indent > 0 {
            let level = self.blocks[index].data.indent;
            let Some(parent) = (0..index)
                .rev()
                .find(|&i| self.blocks[i].data.indent < level)
            else {
                break;
            };
            index = parent;
        }
        let level = self.blocks[index].data.indent;
        if !self.blocks[index].data.kind.is_list() {
            return index..self.block_end(index);
        }
        let mut start = index;
        while start > 0 {
            let Some(prev) = (0..start)
                .rev()
                .find(|&i| self.blocks[i].data.indent <= level)
            else {
                break;
            };
            if self.blocks[prev].data.indent != level || !self.blocks[prev].data.kind.is_list() {
                break;
            }
            start = prev;
        }
        let mut end = self.block_end(index);
        while end < self.blocks.len()
            && self.blocks[end].data.indent == level
            && self.blocks[end].data.kind.is_list()
        {
            end = self.block_end(end);
        }
        start..end
    }

    pub fn check_list(&mut self, id: &str, checked: bool) {
        let scope = self.task_scope(id);
        for b in &mut self.blocks[scope] {
            if b.data.kind == Kind::Todo {
                b.data.checked = checked;
            }
        }
        self.panel = Panel::Closed;
        self.commit_blocks(true);
    }

    pub fn apply_list_shortcut(&mut self, id: &str) {
        let Some(b) = self
            .blocks
            .iter_mut()
            .find(|b| b.data.id == id && b.data.kind == Kind::Text && !b.edit.is_composing())
        else {
            return;
        };
        if let Some((kind, prefix, checked, start)) = shortcut(&b.edit.text)
            && prefix == b.edit.text.len()
            && b.edit.range() == (prefix, prefix)
        {
            b.data.kind = kind;
            b.data.checked = checked;
            b.data.number_start = start;
            let focused = b.edit.focused;
            b.edit = TextEdit::new("").rich();
            b.edit.focused = focused;
            if kind == Kind::Divider {
                let mut next = NoteBlock::new(Kind::Text);
                next.indent = b.data.indent;
                self.active = Some(next.id.clone());
                let index = self
                    .blocks
                    .iter()
                    .position(|b| b.data.id == id)
                    .expect("existing block");
                let end = self.block_end(index);
                self.blocks.insert(end, EditorBlock::from(next));
            }
            if kind == Kind::Todo {
                self.convert_subtasks(id);
            }
        }
    }

    pub fn paste_blocks(&mut self, id: &str, content: &ClipboardContent, tx: &mut Tx) -> bool {
        let Some(index) = self
            .blocks
            .iter()
            .position(|b| b.data.id == id && b.data.kind != Kind::Code)
        else {
            return false;
        };
        // Rune sanitizes HTML; its legacy importer keeps rich marks and list levels.
        let mut pasted = if let Some(html) = content.html() {
            folio::document::import_legacy(html)
        } else {
            content
                .text()
                .replace("\r\n", "\n")
                .replace('\r', "\n")
                .split('\n')
                .map(|line| {
                    let leading = line
                        .chars()
                        .take_while(|c| matches!(c, ' ' | '\t'))
                        .map(|c| if c == '\t' { 4 } else { 1 })
                        .sum::<usize>();
                    let trimmed = line.trim_start_matches([' ', '\t']);
                    let (kind, prefix, checked, start) =
                        shortcut(trimmed).unwrap_or((Kind::Text, 0, false, None));
                    let value = if prefix > 0 {
                        &trimmed[prefix..]
                    } else if leading >= 4 {
                        trimmed
                    } else {
                        line
                    };
                    let mut b = NoteBlock::text(kind, value);
                    b.checked = checked;
                    b.number_start = start;
                    b.indent = (leading / 4).min(MAX_INDENT as usize) as u8;
                    b
                })
                .collect()
        };
        if pasted.len() <= 1 {
            if let Some(html) = content.html() {
                let mut fragment = TextEdit::from_html(html);
                let plain = content.text();
                if fragment.marks().is_empty() && !plain.is_empty() {
                    fragment = TextEdit::new(plain).rich();
                } else if !plain.is_empty() && plain.trim() == fragment.text.trim() {
                    // HTML collapses edge whitespace; keep the clipboard's exact text there.
                    let start = fragment.text.len() - fragment.text.trim_start().len();
                    let end = fragment.text.trim_end().len().max(start);
                    let core = rich_slice(&fragment, start..end);
                    let leading = &plain[..plain.len() - plain.trim_start().len()];
                    let trailing = &plain[plain.trim_end().len()..];
                    fragment = join_rich(&TextEdit::new(leading), &core, "");
                    fragment = join_rich(&fragment, &TextEdit::new(trailing), "");
                }
                let original = &self.blocks[index].edit;
                let (a, b) = original.range();
                let left = rich_slice(original, 0..a);
                let right = rich_slice(original, b..original.text.len());
                let mut edit = join_rich(&join_rich(&left, &fragment, ""), &right, "");
                edit.focused = original.focused;
                edit.set_caret(a + fragment.text.len());
                self.blocks[index].edit = edit;
                self.active = Some(id.into());
                self.commit_blocks(true);
                return true;
            }
            return false;
        }
        if content.text().ends_with('\n')
            && pasted.last().is_some_and(|b| b.plain_text().is_empty())
        {
            pasted.pop();
        }
        if pasted.len() <= 1 {
            return false;
        }
        let mut counters = [0_u32; MAX_INDENT as usize + 1];
        for b in &mut pasted {
            let level = b.indent.min(MAX_INDENT) as usize;
            counters[level + 1..].fill(0);
            if b.kind == Kind::Numbered {
                let expected = counters[level].saturating_add(1);
                let next = b.number_start.unwrap_or(expected);
                if next == expected {
                    b.number_start = None;
                }
                counters[level] = next;
            } else {
                counters[level] = 0;
            }
        }
        let original = &self.blocks[index];
        let original_data = original.data.clone();
        let (a, b) = original.edit.range();
        let left = rich_slice(&original.edit, 0..a);
        let right = rich_slice(&original.edit, b..original.edit.text.len());
        let base_kind = original.data.kind;
        let base_indent = original.data.indent;
        let empty = original.edit.text.is_empty();
        for data in &mut pasted {
            data.indent = (base_indent + data.indent).min(MAX_INDENT);
            if data.kind == Kind::Text && base_kind.is_list() && !base_kind.is_toggle() {
                data.kind = base_kind;
            }
        }
        let first = pasted.first_mut().expect("at least two blocks");
        first.id = id.into();
        if !empty {
            first.kind = base_kind;
            first.checked = original_data.checked;
            first.collapsed = original_data.collapsed;
            first.number_start = original_data.number_start;
        }
        first.html = join_rich(&left, &TextEdit::from_html(&first.html), "").to_html();
        let last = pasted.last_mut().expect("at least two blocks");
        let mut tail = join_rich(&TextEdit::from_html(&last.html), &right, "");
        let caret = tail.text.len() - right.text.len();
        tail.set_caret(caret);
        last.html = tail.to_html();
        self.active = Some(last.id.clone());
        let last_id = last.id.clone();
        // Existing children stay with the first item, before newly pasted siblings.
        let end = self.block_end(index);
        let first = pasted.remove(0);
        self.blocks[index] = EditorBlock::from(first);
        self.blocks
            .splice(end..end, pasted.into_iter().map(EditorBlock::from));
        if let Some(last) = self.blocks.iter_mut().find(|b| b.data.id == last_id) {
            last.edit = tail;
        }
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        tx.focus(input_key(&last_id), false);
        tx.scroll_into_view(format!("block-{last_id}"));
        true
    }

    pub fn visible_blocks(&self) -> Vec<bool> {
        outline::visibility(self.blocks.iter().map(|b| &b.data))
    }

    pub fn block_order(&self) -> Vec<usize> {
        outline::task_sections(self.blocks.iter().map(|b| &b.data))
            .order()
            .collect()
    }
}

// Shared by interactive Markdown shortcuts and plain-text list paste.
fn shortcut(text: &str) -> Option<(Kind, usize, bool, Option<u32>)> {
    for (prefix, kind, checked) in [
        (">### ", Kind::ToggleHeading3, false),
        (">## ", Kind::ToggleHeading2, false),
        ("># ", Kind::ToggleHeading1, false),
        ("- [x] ", Kind::Todo, true),
        ("- [ ] ", Kind::Todo, false),
        ("[x] ", Kind::Todo, true),
        ("[ ] ", Kind::Todo, false),
        ("[] ", Kind::Todo, false),
        ("### ", Kind::Heading3, false),
        ("## ", Kind::Heading2, false),
        ("# ", Kind::Heading1, false),
        ("- ", Kind::Bullet, false),
        ("* ", Kind::Bullet, false),
        ("+ ", Kind::Bullet, false),
        ("> ", Kind::Toggle, false),
        ("\" ", Kind::Quote, false),
        ("``` ", Kind::Code, false),
        ("--- ", Kind::Divider, false),
    ] {
        if text.starts_with(prefix) {
            return Some((kind, prefix.len(), checked, None));
        }
    }
    let (number, _) = text.split_once(". ")?;
    if number.is_empty() || number.len() > 4 || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let prefix_len = number.len() + 2;
    let number: u32 = number.parse().ok()?;
    (number > 0).then_some((Kind::Numbered, prefix_len, false, Some(number)))
}
