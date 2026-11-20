//! A handle press becomes a reorder after travelling six points. Only the drop edits data.
use crate::state::{Panel, State};
use rune::{Drag, Phase, Pick, ScrollPos, Tx};
use std::{cell::Cell, collections::BTreeSet, rc::Rc, time::Duration};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropPosition {
    Before(String),
    After(String),
    Inside(String),
}

struct DropPlan {
    indices: BTreeSet<usize>,
    levels: Vec<u8>,
    slot: usize,
}

#[derive(Default)]
pub enum BlockDrag {
    #[default]
    Idle,
    Pressed(DragSession),
    Dragging {
        session: DragSession,
        point: [f32; 2],
        target: Option<DropPosition>,
    },
    /// Rune can deliver a click after a drag released over its own handle.
    Released(String),
}

pub struct DragSession {
    page_id: String,
    block_id: String,
    attempt: String,
    viewport: Option<[f32; 4]>,
    scroll: Rc<Cell<ScrollPos>>,
}

impl BlockDrag {
    fn session(&self) -> Option<&DragSession> {
        match self {
            Self::Pressed(s) | Self::Dragging { session: s, .. } => Some(s),
            _ => None,
        }
    }
    pub fn source(&self) -> Option<&str> {
        match self {
            Self::Dragging { session, .. } => Some(&session.block_id),
            _ => None,
        }
    }
    pub fn point(&self) -> Option<[f32; 2]> {
        match self {
            Self::Dragging { point, .. } => Some(*point),
            _ => None,
        }
    }
    pub fn target(&self) -> Option<&DropPosition> {
        match self {
            Self::Dragging { target, .. } => target.as_ref(),
            _ => None,
        }
    }
    pub fn scroll_feed(&self) -> Option<Rc<Cell<ScrollPos>>> {
        self.session().map(|s| s.scroll.clone())
    }
    pub fn set_viewport(&mut self, rect: [f32; 4]) {
        match self {
            Self::Pressed(s) | Self::Dragging { session: s, .. } => s.viewport = Some(rect),
            _ => {}
        }
    }
}

impl State {
    pub fn block_handle_click(&mut self, id: &str, tx: &Tx) {
        if self.select_block(id, tx.mods()) {
            return;
        }
        if !tx.by_keys()
            && matches!(&self.block_drag, BlockDrag::Dragging { session, .. } if session.block_id == id)
        {
            return;
        }
        if !tx.by_keys() && matches!(&self.block_drag, BlockDrag::Released(source) if source == id)
        {
            self.block_drag = BlockDrag::Idle;
            return;
        }
        self.block_drag = BlockDrag::Idle;
        self.active = Some(id.into());
        self.selection.anchor = Some(id.into());
        self.panel = Panel::Block(id.into());
    }

    pub fn drag_block(&mut self, id: &str, drag: &Drag, tx: &mut Tx) {
        if drag.phase == Phase::Down {
            let Some(page) = self
                .current()
                .filter(|p| !p.archived && p.collection.is_none() && p.dashboard.is_none())
            else {
                return;
            };
            if !self.blocks.iter().any(|b| b.data.id == id) {
                return;
            }
            self.block_drag = BlockDrag::Pressed(DragSession {
                page_id: page.id.clone(),
                block_id: id.into(),
                attempt: uuid::Uuid::new_v4().to_string(),
                viewport: None,
                scroll: Rc::new(Cell::new(ScrollPos::default())),
            });
            if self.selection.anchor.is_none() {
                self.selection.anchor = self.active.clone();
            }
            self.active = Some(id.into());
            self.panel = Panel::Closed;
            return;
        }
        let valid = self
            .block_drag
            .session()
            .is_some_and(|s| s.block_id == id && Some(&s.page_id) == self.selected.as_ref());
        if !valid {
            return;
        }
        if drag.cancelled() {
            self.block_drag = BlockDrag::Released(id.into());
            return;
        }
        let gesture = std::mem::take(&mut self.block_drag);
        self.block_drag = match gesture {
            BlockDrag::Pressed(session) if drag.dx.hypot(drag.dy) >= 6.0 => BlockDrag::Dragging {
                session,
                point: drag.window_point(),
                target: None,
            },
            BlockDrag::Dragging {
                session, target, ..
            } => BlockDrag::Dragging {
                session,
                point: drag.window_point(),
                target,
            },
            other => other,
        };
        let BlockDrag::Dragging { session, point, .. } = &self.block_drag else {
            if drag.phase == Phase::Up {
                self.block_drag = BlockDrag::Idle;
            }
            return;
        };
        let (x, y) = (point[0], point[1]);
        let attempt = session.attempt.clone();
        let drop = drag.phase == Phase::Up;
        // Pick the last presented frame, including clips and scrolling, without tracking row rects.
        tx.element_at_in_with(
            "block-editor",
            x,
            y,
            Pick::default().transparent(true),
            move |s: &mut State, picked, tx| {
                s.block_drop_target(&attempt, picked.as_ref().and_then(|p| p.key()), drop, tx);
            },
        );
        if !drop {
            self.scroll_block_drag(tx);
        }
    }

    fn scroll_block_drag(&self, tx: &mut Tx) {
        let BlockDrag::Dragging {
            session,
            point: [x, y],
            ..
        } = &self.block_drag
        else {
            return;
        };
        let Some([left, top, width, height]) = session.viewport else {
            return;
        };
        if *x < left || *x > left + width || *y < top || *y > top + height {
            return;
        }
        let position = session.scroll.get();
        let direction = if *y < top + 36.0 {
            -1.0
        } else if *y > top + height - 36.0 {
            1.0
        } else {
            return;
        };
        let next = (position.offset[1] + direction * 12.0).clamp(0.0, position.max[1]);
        if (next - position.offset[1]).abs() > 0.1 {
            tx.scroll_to_instant("document-scroll", position.offset[0], next);
            tx.drag_repeat_after(Duration::from_millis(16));
        }
    }

    pub fn can_nest_into(&self, index: usize) -> bool {
        use folio::document::Kind;
        self.blocks[index].data.indent < folio::outline::MAX_INDENT
            && !matches!(
                self.blocks[index].data.kind,
                Kind::Code
                    | Kind::Divider
                    | Kind::PageLink
                    | Kind::DatabaseView
                    | Kind::Image
                    | Kind::Attachment
                    | Kind::TableOfContents
            )
    }

    fn block_drop_target(&mut self, attempt: &str, key: Option<&str>, drop: bool, tx: &mut Tx) {
        let Some(session) = self.block_drag.session() else {
            return;
        };
        if session.attempt != attempt || Some(&session.page_id) != self.selected.as_ref() {
            return;
        }
        let source = session.block_id.clone();
        let candidate = if key == Some("block-drop-indicator") {
            self.block_drag.target().cloned()
        } else {
            key.and_then(|key| {
                if key == "drop-end" {
                    self.blocks
                        .iter()
                        .rfind(|b| b.data.indent == 0)
                        .or_else(|| self.blocks.last())
                        .map(|b| DropPosition::After(b.data.id.clone()))
                } else if let Some(id) = key.strip_prefix("drop-before-") {
                    Some(DropPosition::Before(id.into()))
                } else if let Some(id) = key.strip_prefix("drop-inside-") {
                    Some(DropPosition::Inside(id.into()))
                } else {
                    key.strip_prefix("drop-after-")
                        .map(|id| DropPosition::After(id.into()))
                }
            })
        }
        .filter(|target| self.drop_plan(&source, target).is_some());
        if !drop {
            if let BlockDrag::Dragging { target, .. } = &mut self.block_drag {
                *target = candidate;
            }
            return;
        }
        self.block_drag = BlockDrag::Released(source.clone());
        if let Some(target) = candidate {
            self.reorder_block(&source, &target, tx);
        }
    }

    fn drop_plan(&self, id: &str, target: &DropPosition) -> Option<DropPlan> {
        let from = self.blocks.iter().position(|b| b.data.id == id)?;
        let (target_id, after, inside) = match target {
            DropPosition::Before(id) => (id, false, false),
            DropPosition::After(id) => (id, true, false),
            DropPosition::Inside(id) => (id, true, true),
        };
        let to = self.blocks.iter().position(|b| b.data.id == *target_id)?;
        let indices = if self.selection.ids.contains(id) && self.selection.ids.len() > 1 {
            self.selected_indices()
        } else {
            (from..self.block_end(from)).collect()
        };
        if indices.contains(&to) || (inside && !self.can_nest_into(to)) {
            return None;
        }
        let level = self.blocks[to].data.indent + u8::from(inside);
        let mut end = 0;
        let mut base = 0;
        let mut levels = Vec::with_capacity(indices.len());
        for &i in &indices {
            if i >= end {
                end = self.block_end(i);
                base = self.blocks[i].data.indent;
            }
            let next = level + self.blocks[i].data.indent.saturating_sub(base);
            if next > folio::outline::MAX_INDENT {
                return None;
            }
            levels.push(next);
        }
        Some(DropPlan {
            indices,
            levels,
            slot: if after { self.block_end(to) } else { to },
        })
    }

    fn reorder_block(&mut self, id: &str, target: &DropPosition, tx: &mut Tx) {
        let Some(plan) = self.drop_plan(id, target) else {
            return;
        };
        let insert = plan.slot - plan.indices.iter().filter(|&&i| i < plan.slot).count();
        let mut moved = Vec::with_capacity(plan.indices.len());
        let mut remaining = Vec::new();
        let mut levels = plan.levels.into_iter();
        for (i, mut block) in self.blocks.drain(..).enumerate() {
            if plan.indices.contains(&i) {
                block.data.indent = levels.next().expect("one level per moved block");
                moved.push(block);
            } else {
                remaining.push(block);
            }
        }
        remaining.splice(insert..insert, moved);
        self.blocks = remaining;
        if let DropPosition::Inside(parent) = target
            && let Some(parent) = self.blocks.iter_mut().find(|b| b.data.id == *parent)
        {
            parent.data.collapsed = false;
        }
        self.active = Some(id.into());
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        tx.focus(format!("handle-{id}"), false);
        tx.scroll_into_view(format!("block-{id}"));
        tx.instant();
    }
}
