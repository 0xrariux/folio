//! A handle press becomes a reorder after travelling six points. Only the drop edits data.
use crate::state::{Panel, State};
use rune::{Drag, Phase, Pick, ScrollPos, Tx};
use std::{cell::Cell, rc::Rc, time::Duration};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropPosition {
    Before(String),
    After(String),
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

    fn block_drop_target(&mut self, attempt: &str, key: Option<&str>, drop: bool, tx: &mut Tx) {
        let end = self
            .blocks
            .iter()
            .rfind(|b| b.data.indent == 0)
            .or_else(|| self.blocks.last())
            .map(|b| b.data.id.clone());
        let BlockDrag::Dragging {
            session, target, ..
        } = &mut self.block_drag
        else {
            return;
        };
        if session.attempt != attempt || Some(&session.page_id) != self.selected.as_ref() {
            return;
        }
        if key != Some("block-drop-indicator") {
            *target = key.and_then(|key| {
                if key == "drop-end" {
                    end.map(DropPosition::After)
                } else if let Some(id) = key.strip_prefix("drop-before-") {
                    Some(DropPosition::Before(id.into()))
                } else {
                    key.strip_prefix("drop-after-")
                        .map(|id| DropPosition::After(id.into()))
                }
            });
        }
        if !drop {
            return;
        }
        let source = session.block_id.clone();
        let target = target.clone();
        self.block_drag = BlockDrag::Released(source.clone());
        if let Some(target) = target {
            self.reorder_block(&source, &target, tx);
        }
    }

    fn reorder_block(&mut self, id: &str, target: &DropPosition, tx: &mut Tx) {
        let Some(from) = self.blocks.iter().position(|b| b.data.id == id) else {
            return;
        };
        let (target_id, after) = match target {
            DropPosition::Before(id) => (id, false),
            DropPosition::After(id) => (id, true),
        };
        let Some(to) = self.blocks.iter().position(|b| b.data.id == *target_id) else {
            return;
        };
        let end = self.block_end(from);
        if (from..end).contains(&to) {
            return;
        }
        let slot = if after { self.block_end(to) } else { to };
        if self.selection.ids.contains(id) && self.selection.ids.len() > 1 {
            self.reorder_selection(slot);
            tx.focus(format!("handle-{id}"), false);
            tx.instant();
            return;
        }
        if slot == from || slot == end {
            return;
        }
        let old_level = self.blocks[from].data.indent;
        let new_level = self.blocks[to].data.indent;
        let deepest = self.blocks[from..end]
            .iter()
            .map(|b| b.data.indent.saturating_sub(old_level))
            .max()
            .unwrap_or(0);
        if new_level.saturating_add(deepest) > folio::outline::MAX_INDENT {
            return;
        }
        for b in &mut self.blocks[from..end] {
            b.data.indent = new_level + b.data.indent.saturating_sub(old_level);
        }
        if slot < from {
            self.blocks[slot..end].rotate_right(end - from);
        } else {
            self.blocks[from..slot].rotate_left(end - from);
        }
        self.active = Some(id.into());
        self.panel = Panel::Closed;
        self.commit_blocks(true);
        tx.focus(format!("handle-{id}"), false);
        tx.scroll_into_view(format!("block-{id}"));
        tx.instant();
    }
}
