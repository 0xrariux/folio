//! A flat, ordered outline: greater indentation belongs to the preceding parent.
use crate::document::{Kind, NoteBlock};

pub const MAX_INDENT: u8 = 6;

/// Exclusive subtree bounds, computed once in linear time for rendering/export.
pub fn subtree_ends<'a>(blocks: impl IntoIterator<Item = &'a NoteBlock>) -> Vec<usize> {
    let levels: Vec<_> = blocks.into_iter().map(|b| b.indent).collect();
    let mut ends = vec![levels.len(); levels.len()];
    let mut stack: Vec<usize> = Vec::new();
    for (index, level) in levels.iter().enumerate() {
        while stack.last().is_some_and(|&i| levels[i] >= *level) {
            ends[stack.pop().expect("nonempty stack")] = index;
        }
        stack.push(index);
    }
    ends
}

pub fn visibility<'a>(blocks: impl IntoIterator<Item = &'a NoteBlock>) -> Vec<bool> {
    let mut hidden_below = None;
    blocks
        .into_iter()
        .map(|b| {
            if hidden_below.is_some_and(|level| b.indent <= level) {
                hidden_below = None;
            }
            let visible = hidden_below.is_none();
            if visible && b.kind.is_toggle() && b.collapsed {
                hidden_below = Some(b.indent);
            }
            visible
        })
        .collect()
}

/// A nested list does not reset its parent's sequence. A new sibling parent does.
pub fn numbers<'a>(blocks: impl IntoIterator<Item = &'a NoteBlock>) -> Vec<u32> {
    let mut counters = [0_u32; MAX_INDENT as usize + 1];
    blocks
        .into_iter()
        .map(|b| {
            let level = b.indent.min(MAX_INDENT) as usize;
            counters[level + 1..].fill(0);
            counters[level] = if b.kind == Kind::Numbered {
                b.number_start
                    .unwrap_or_else(|| counters[level].saturating_add(1))
                    .max(1)
            } else {
                0
            };
            counters[level]
        })
        .collect()
}

fn inner(block: &NoteBlock) -> &str {
    block
        .html
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
        .unwrap_or(&block.html)
}

pub fn html(blocks: &[NoteBlock]) -> String {
    let blocks: Vec<_> = blocks
        .iter()
        .cloned()
        .map(|mut b| {
            b.html = rune::TextEdit::from_html(&b.html).to_html();
            b
        })
        .collect();
    let ends = subtree_ends(&blocks);
    let numbers = numbers(&blocks);
    render_html(&blocks, &ends, &numbers, 0, blocks.len())
}

fn render_html(
    blocks: &[NoteBlock],
    ends: &[usize],
    numbers: &[u32],
    mut index: usize,
    end: usize,
) -> String {
    let mut out = String::new();
    while index < end {
        let block = &blocks[index];
        let next = ends[index].min(end);
        if matches!(block.kind, Kind::Bullet | Kind::Numbered | Kind::Todo) {
            let kind = block.kind;
            let level = block.indent;
            let tag = if kind == Kind::Numbered { "ol" } else { "ul" };
            if kind == Kind::Numbered {
                out.push_str(&format!("<ol start=\"{}\">", numbers[index]));
            } else {
                out.push_str("<ul>");
            }
            loop {
                let b = &blocks[index];
                let next = ends[index].min(end);
                out.push_str("<li>");
                if kind == Kind::Todo {
                    out.push_str(if b.checked {
                        "<input type=\"checkbox\" disabled checked> "
                    } else {
                        "<input type=\"checkbox\" disabled> "
                    });
                }
                out.push_str(inner(b));
                out.push_str(&render_html(blocks, ends, numbers, index + 1, next));
                out.push_str("</li>");
                index = next;
                if index >= end
                    || blocks[index].kind != kind
                    || blocks[index].indent != level
                    || (kind == Kind::Numbered && blocks[index].number_start.is_some())
                {
                    break;
                }
            }
            out.push_str(&format!("</{tag}>\n"));
            continue;
        }
        if block.kind.is_toggle() {
            out.push_str(if block.collapsed {
                "<details><summary>"
            } else {
                "<details open><summary>"
            });
            let heading = match block.kind {
                Kind::ToggleHeading1 => Some("h1"),
                Kind::ToggleHeading2 => Some("h2"),
                Kind::ToggleHeading3 => Some("h3"),
                _ => None,
            };
            if let Some(tag) = heading {
                out.push_str(&format!("<{tag}>{}</{tag}>", inner(block)));
            } else {
                out.push_str(inner(block));
            }
            out.push_str("</summary>\n");
            out.push_str(&render_html(blocks, ends, numbers, index + 1, next));
            out.push_str("</details>\n");
        } else {
            out.push_str(&block.export_html());
            out.push('\n');
            out.push_str(&render_html(blocks, ends, numbers, index + 1, next));
        }
        index = next;
    }
    out
}
