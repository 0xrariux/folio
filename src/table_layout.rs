use crate::state::State;
use rune::{Drag, Phase};
impl State {
    pub fn resize_column(&mut self, id: &str, default: f32, drag: &mut Drag) {
        let Some(page_id) = self.selected.clone() else {
            return;
        };
        if let Some(settings) = self
            .pages
            .iter_mut()
            .find(|p| p.id == page_id)
            .and_then(|p| p.collection.as_mut())
        {
            if drag.phase == Phase::Down {
                drag.data = settings.width(id, default);
            }
            let width = if drag.cancelled() {
                drag.data
            } else {
                (drag.data + drag.dx).clamp(70.0, 500.0)
            };
            settings.layout.widths.insert(id.into(), width as u16);
            if drag.phase == Phase::Up {
                settings.sync_view();
            }
        }
        if drag.phase == Phase::Up {
            self.persist(&page_id);
        }
    }
    pub fn move_column(&mut self, id: &str, down: bool) {
        let Some(page_id) = self.selected.clone() else {
            return;
        };
        if let Some(settings) = self
            .pages
            .iter_mut()
            .find(|p| p.id == page_id)
            .and_then(|p| p.collection.as_mut())
        {
            let all: Vec<_> = folio::collection::BuiltinColumn::ALL
                .into_iter()
                .map(|c| format!("{c:?}"))
                .chain(settings.columns.iter().map(|c| c.id.clone()))
                .collect();
            let mut order = settings.layout.order.clone();
            order.retain(|id| all.contains(id));
            for id in all {
                if !order.contains(&id) {
                    order.push(id);
                }
            }
            if let Some(index) = order.iter().position(|key| key == id) {
                let next = if down {
                    index.checked_add(1).filter(|&i| i < order.len())
                } else {
                    index.checked_sub(1)
                };
                if let Some(next) = next {
                    order.swap(index, next);
                }
            }
            settings.layout.order = order;
            settings.sync_view();
        }
        self.persist(&page_id);
    }
}
