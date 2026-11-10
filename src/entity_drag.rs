use crate::state::State;
use rune::{Drag, Phase, Pick, Tx};
use std::{cell::Cell, rc::Rc};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Card,
    Page,
    Calendar,
}
pub struct Session {
    pub id: String,
    pub kind: Kind,
    pub page: Option<String>,
    pub active: bool,
    pub point: [f32; 2],
    pub target: Option<String>,
    pub released: bool,
    pub scroll_key: String,
    pub viewport: Option<[f32; 4]>,
    pub scroll: Rc<Cell<rune::ScrollPos>>,
}
impl State {
    pub fn entity_click(&mut self, id: &str) -> bool {
        if self
            .workspace
            .entity_drag
            .as_ref()
            .is_some_and(|d| d.id == id && (d.active || d.released))
        {
            self.workspace.entity_drag = None;
            return true;
        }
        false
    }
    pub fn drag_entity(&mut self, id: &str, kind: Kind, drag: &Drag, tx: &mut Tx) {
        if drag.phase == Phase::Down {
            let scroll_key = match kind {
                Kind::Page => "pages-scroll".into(),
                Kind::Calendar => "document-scroll".into(),
                Kind::Card => self
                    .pages
                    .iter()
                    .find(|p| p.id == id)
                    .map_or("document-scroll".into(), |p| {
                        format!("board-scroll-{:?}", p.properties.status)
                    }),
            };
            self.workspace.entity_drag = Some(Session {
                id: id.into(),
                kind,
                page: self.selected.clone(),
                active: false,
                point: drag.window_point(),
                target: None,
                released: false,
                scroll_key,
                viewport: None,
                scroll: Rc::new(Cell::new(rune::ScrollPos::default())),
            });
            return;
        }
        let Some(session) = &mut self.workspace.entity_drag else {
            return;
        };
        if session.id != id || session.page != self.selected {
            return;
        }
        if drag.cancelled() {
            session.active = false;
            session.released = true;
            return;
        }
        session.active |= drag.dx.hypot(drag.dy) >= 6.0;
        session.point = drag.window_point();
        if !session.active {
            return;
        }
        let drop = drag.phase == Phase::Up;
        let point = session.point;
        let source = id.to_string();
        if drop {
            session.released = true;
        }
        tx.element_at_in_with(
            "folio",
            point[0],
            point[1],
            Pick::default().transparent(true),
            move |s: &mut State, picked, _| {
                let target = picked.as_ref().and_then(|p| p.key()).map(str::to_string);
                let Some(session) = &mut s.workspace.entity_drag else {
                    return;
                };
                if session.id != source || session.page != s.selected {
                    return;
                }
                session.target = target.clone();
                if session.kind == Kind::Card
                    && let Some(status) =
                        target.as_ref().and_then(|v| v.strip_prefix("board-drop-"))
                {
                    session.scroll_key = format!("board-scroll-{status}");
                }
                if !drop {
                    return;
                }
                session.active = false;
                let Some(target) = target else {
                    return;
                };
                match kind {
                    Kind::Card => {
                        let status = match target.strip_prefix("board-drop-") {
                            Some("Todo") => Some(folio::collection::Status::Todo),
                            Some("InProgress") => Some(folio::collection::Status::InProgress),
                            Some("Done") => Some(folio::collection::Status::Done),
                            _ => None,
                        };
                        if let Some(status) = status {
                            s.set_status(&source, status);
                        }
                    }
                    Kind::Calendar => {
                        if let Some(date) = target.strip_prefix("calendar-day-") {
                            s.set_due(&source, None, Some(date));
                        }
                    }
                    Kind::Page => s.drop_page(&source, &target),
                }
            },
        );
        if !drop {
            if let Some(session) = &self.workspace.entity_drag
                && let Some([left, top, width, height]) = session.viewport
            {
                let [x, y] = session.point;
                if x >= left && x <= left + width && y >= top && y <= top + height {
                    let direction = if y < top + 32.0 {
                        -1.0
                    } else if y > top + height - 32.0 {
                        1.0
                    } else {
                        0.0
                    };
                    if direction != 0.0 {
                        let scroll = session.scroll.get();
                        let next = (scroll.offset[1] + direction * 12.0).clamp(0.0, scroll.max[1]);
                        tx.scroll_to_instant(&session.scroll_key, scroll.offset[0], next);
                    }
                }
            }
            tx.drag_repeat_after(std::time::Duration::from_millis(16));
        }
    }
    pub fn drop_page(&mut self, source: &str, target: &str) {
        let (target_id, mode) = if target == "tree-root" {
            (None, 0)
        } else if let Some(id) = target.strip_prefix("tree-inside-") {
            (Some(id), 0)
        } else if let Some(id) = target.strip_prefix("tree-before-") {
            (Some(id), -1)
        } else if let Some(id) = target.strip_prefix("tree-after-") {
            (Some(id), 1)
        } else {
            return;
        };
        let Some(index) = self
            .pages
            .iter()
            .position(|p| p.id == source && !p.archived)
        else {
            return;
        };
        if target_id == Some(source) {
            return;
        }
        let parent = if let Some(id) = target_id {
            let Some(target) = self.pages.iter().find(|p| p.id == id && !p.archived) else {
                return;
            };
            if mode == 0 {
                Some(id.to_string())
            } else {
                target.parent_id.clone()
            }
        } else {
            None
        };
        if parent.as_deref() == Some(source)
            || parent
                .as_ref()
                .is_some_and(|id| self.ancestors(id).iter().any(|p| p == source))
        {
            return;
        }
        self.pages[index].parent_id = parent.clone();
        let mut siblings: Vec<_> = self
            .pages
            .iter()
            .filter(|p| p.parent_id == parent && p.id != source)
            .map(|p| (p.order, p.id.clone()))
            .collect();
        siblings.sort_by_key(|(order, _)| *order);
        let at = if mode == 0 {
            siblings.len()
        } else {
            siblings
                .iter()
                .position(|(_, id)| Some(id.as_str()) == target_id)
                .map_or(siblings.len(), |i| i + usize::from(mode > 0))
        };
        siblings.insert(at, (0, source.to_string()));
        let ids: Vec<_> = siblings
            .into_iter()
            .enumerate()
            .map(|(i, (_, id))| {
                if let Some(p) = self.pages.iter_mut().find(|p| p.id == id) {
                    p.order = i as i64 * 1024;
                }
                id
            })
            .collect();
        if let Some(parent) = parent {
            self.collapsed.remove(&parent);
        }
        for id in ids {
            self.persist(&id);
        }
        self.pages.sort_by_key(|p| p.order);
        self.save_preferences();
    }
}
