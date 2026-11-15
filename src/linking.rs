use crate::{
    editor::EditorBlock,
    state::{Panel, State},
};
use folio::{
    document::{Kind, NoteBlock},
    model::{Page, now},
};
use rune::{TextEdit, Tx};
use rune_kit::search::Fold;

#[derive(Clone)]
pub struct LinkForm {
    pub id: String,
    pub typed: bool,
    pub query: TextEdit,
}

impl State {
    pub fn open_link_picker(&mut self, id: String, typed: bool) {
        self.panel = Panel::Link(Box::new(LinkForm {
            id,
            typed,
            query: TextEdit::new(""),
        }));
    }

    pub fn link_query(&mut self) -> &mut TextEdit {
        let Panel::Link(form) = &mut self.panel else {
            panic!("the link picker owns its search field");
        };
        &mut form.query
    }

    pub fn link_targets(&self) -> Vec<&Page> {
        let Panel::Link(form) = &self.panel else {
            return Vec::new();
        };
        let query = form.query.text.trim();
        self.pages
            .iter()
            .filter(|p| {
                !p.archived
                    && Some(&p.id) != self.selected.as_ref()
                    && Fold::for_language("fr").contains(&p.title, query)
            })
            .collect()
    }

    pub fn link_page(&mut self, target: &str, tx: &mut Tx) {
        let Panel::Link(form) = self.panel.clone() else {
            return;
        };
        let Some(page) = self.pages.iter().find(|p| p.id == target && !p.archived) else {
            return;
        };
        let title = page.display_title().to_string();
        let Some(index) = self.blocks.iter().position(|b| b.data.id == form.id) else {
            return;
        };
        let insert = !form.typed
            && !self.blocks[index].edit.text.is_empty()
            && self.blocks[index].data.kind != Kind::PageLink;
        let mut block = if insert {
            EditorBlock::from(NoteBlock::new(Kind::PageLink))
        } else {
            EditorBlock::from(self.blocks[index].data.clone())
        };
        block.data.kind = Kind::PageLink;
        block.data.target_id = Some(target.into());
        block.data.indent = self.blocks[index].data.indent;
        block.edit = TextEdit::new(title).rich();
        self.active = Some(block.data.id.clone());
        if insert {
            let end = self.block_end(index);
            self.blocks.insert(end, block);
        } else {
            self.blocks[index] = block;
        }
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        tx.focus(
            format!("page-link-{}", self.active.as_deref().unwrap_or("")),
            false,
        );
    }

    pub fn backlinks(&self, target: &str) -> Vec<&Page> {
        self.pages
            .iter()
            .filter(|p| {
                !p.archived
                    && p.id != target
                    && p.blocks
                        .iter()
                        .any(|b| (b.kind == Kind::PageLink && b.target_id.as_deref() == Some(target)) || TextEdit::from_html(&b.html).marks().iter().any(|m| matches!(&m.mark, rune::Mark::Link(url) if url.strip_prefix("https://folio.local/page/").is_some_and(|v| v.split('#').next() == Some(target)))))
            })
            .collect()
    }

    pub fn refresh_references(&mut self, target: &str) {
        let Some(title) = self
            .pages
            .iter()
            .find(|p| p.id == target)
            .map(|p| p.display_title().to_string())
        else {
            return;
        };
        let mut changed = Vec::new();
        for page in &mut self.pages {
            let mut dirty = false;
            for block in &mut page.blocks {
                if block.kind == Kind::PageLink && block.target_id.as_deref() == Some(target) {
                    block.html = TextEdit::new(&title).rich().to_html();
                    dirty = true;
                }
                if let Some(html) = renamed_mentions(&block.html, target, &title) {
                    block.html = html;
                    dirty = true;
                }
            }
            if dirty {
                if self.selected.as_ref() == Some(&page.id) {
                    for editor in &mut self.blocks {
                        if let Some(block) = page.blocks.iter().find(|b| b.id == editor.data.id)
                            && block.html != editor.data.html
                        {
                            let focus = editor.edit.focused;
                            let caret = editor.edit.focus;
                            editor.data.html = block.html.clone();
                            editor.edit = TextEdit::from_html(&block.html);
                            editor.edit.focused = focus;
                            let mut at = caret.min(editor.edit.text.len());
                            while !editor.edit.text.is_char_boundary(at) {
                                at -= 1;
                            }
                            editor.edit.set_caret(at);
                        }
                    }
                }
                page.sync_document();
                page.updated_at = now();
                changed.push(page.id.clone());
            }
        }
        for id in changed {
            self.persist(&id);
        }
    }
}

fn renamed_mentions(html: &str, target: &str, title: &str) -> Option<String> {
    let url = crate::command_palette::reference_url(target, None);
    if !html.contains(&folio::document::escape(&url)) {
        return None;
    }
    let mut edit = TextEdit::from_html(html);
    let mut ranges: Vec<_> = edit
        .marks()
        .iter()
        .filter_map(|m| {
            matches!(&m.mark, rune::Mark::Link(link) if link == &url).then_some((m.start, m.end))
        })
        .collect();
    ranges.sort_unstable();
    ranges.dedup();
    if ranges.is_empty() {
        return None;
    }
    for (start, end) in ranges.into_iter().rev() {
        let marks = edit
            .marks()
            .iter()
            .filter(|m| m.start <= start && m.end >= end)
            .map(|m| rune::MarkRange::new(0..title.len(), m.mark.clone()))
            .collect();
        let fragment = TextEdit::from_parts(title, marks, vec![rune::Block::Paragraph]);
        let left = folio::document::rich_slice(&edit, 0..start);
        let right = folio::document::rich_slice(&edit, end..edit.text.len());
        edit = folio::document::join_rich(
            &folio::document::join_rich(&left, &fragment, ""),
            &right,
            "",
        );
    }
    Some(edit.to_html())
}
