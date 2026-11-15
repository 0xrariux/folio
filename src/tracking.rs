use crate::state::{Panel, Phase, State};
use folio::{
    collection::{Priority, Status},
    dashboard::{DashboardConfig, Deadline, Scope, TrackingFilter},
    model::{Page, now},
};
use rune_kit::search::Fold;

impl State {
    pub fn open_dashboard(&mut self) {
        if !matches!(self.phase, Phase::Ready) {
            return;
        }
        if let Some(id) = self
            .pages
            .iter()
            .find(|p| p.dashboard.is_some() && !p.archived)
            .map(|p| p.id.clone())
        {
            self.select(&id);
            return;
        }
        let mut page = Page::new(None);
        page.title = "Vue d’ensemble".into();
        page.emoji = "📊".into();
        page.wide = true;
        page.dashboard = Some(DashboardConfig::default());
        self.add_page(page);
    }
    pub fn tracking_sources(&self) -> Vec<&Page> {
        let Some(current) = self.current() else {
            return Vec::new();
        };
        if let Some(config) = &current.dashboard {
            self.pages
                .iter()
                .filter(|p| p.collection.is_some() && !p.archived && config.scope.contains(&p.id))
                .collect()
        } else if current.collection.is_some() && !current.archived {
            vec![current]
        } else {
            Vec::new()
        }
    }
    pub fn tracking_filter(&self) -> TrackingFilter {
        self.current()
            .and_then(|p| {
                p.dashboard
                    .as_ref()
                    .map(|d| d.filter.clone())
                    .or_else(|| p.collection.as_ref().map(|c| c.filter()))
            })
            .unwrap_or_default()
    }
    pub fn tracking_items(&self) -> Vec<&Page> {
        if self.current().is_some_and(|p| p.collection.is_some()) {
            return self.collection_items();
        }
        let sources = self.tracking_sources();
        let filter = self.tracking_filter();
        let query = self.collection_search.text.trim();
        let mut items: Vec<_> = self
            .pages
            .iter()
            .filter(|p| {
                !p.archived
                    && p.parent_id
                        .as_ref()
                        .is_some_and(|id| sources.iter().any(|s| s.id == *id))
                    && filter.matches(p, &self.today)
                    && (p.matches(query)
                        || p.properties
                            .tags
                            .iter()
                            .any(|tag| Fold::for_language("fr").contains(tag, query))
                        || p.properties
                            .custom
                            .values()
                            .any(|v| Fold::for_language("fr").contains(&v.display(), query)))
            })
            .collect();
        items.sort_by(|a, b| {
            a.properties
                .due
                .as_deref()
                .unwrap_or("9999-12-31")
                .cmp(b.properties.due.as_deref().unwrap_or("9999-12-31"))
                .then_with(|| a.title.cmp(&b.title))
                .then_with(|| a.id.cmp(&b.id))
        });
        items
    }
    fn change_tracking(&mut self, change: impl FnOnce(&mut TrackingFilter)) {
        let Some(id) = self.selected.clone() else {
            return;
        };
        let mut filter = self.tracking_filter();
        change(&mut filter);
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            if let Some(dashboard) = &mut page.dashboard {
                dashboard.filter = filter;
            } else if let Some(collection) = &mut page.collection {
                collection.status = filter.status;
                collection.priority = filter.priority;
                collection.deadline = filter.deadline;
                collection.sync_view();
            } else {
                return;
            }
            page.updated_at = now();
            self.persist(&id);
        }
        self.panel = Panel::Closed;
    }
    pub fn tracking_status(&mut self, status: Option<Status>) {
        self.change_tracking(|f| f.status = status);
    }
    pub fn tracking_priority(&mut self, priority: Option<Priority>) {
        self.change_tracking(|f| f.priority = priority);
    }
    pub fn tracking_deadline(&mut self, deadline: Deadline) {
        self.change_tracking(|f| f.deadline = deadline);
    }
    pub fn clear_tracking(&mut self) {
        self.collection_search = rune::TextEdit::new("");
        self.change_tracking(|f| *f = TrackingFilter::default());
    }
    pub fn dashboard_scope(&mut self, source: Option<&str>) {
        let all: Vec<_> = self
            .pages
            .iter()
            .filter(|p| p.collection.is_some() && !p.archived)
            .map(|p| p.id.clone())
            .collect();
        let Some(id) = self.selected.clone() else {
            return;
        };
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            let Some(config) = &mut page.dashboard else {
                return;
            };
            if let Some(source) = source {
                if !all.iter().any(|id| id == source) {
                    return;
                }
                let mut selected = match &config.scope {
                    Scope::All => all,
                    Scope::Selected(ids) => ids.clone(),
                };
                if selected.iter().any(|id| id == source) {
                    selected.retain(|id| id != source);
                } else {
                    selected.push(source.into());
                }
                config.scope = Scope::Selected(selected);
            } else {
                config.scope = Scope::All;
            }
            page.updated_at = now();
            self.persist(&id);
        }
    }
}
