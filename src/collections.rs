use crate::state::{Panel, Phase, State};
use folio::{
    collection::{Collection, Priority, Sort, Status, View, parse_tags, valid_date},
    model::{Page, now},
};
use rune::{TextEdit, Tx};
use rune_kit::search::Fold;

#[derive(Clone, Copy)]
pub enum PropertyField {
    Status,
    Priority,
}

#[derive(Clone)]
pub struct PropertyForm {
    pub id: String,
    pub anchor: String,
    pub due: TextEdit,
    pub tags: TextEdit,
    pub error: Option<String>,
}

impl State {
    pub fn create_collection(&mut self) {
        if !matches!(self.phase, Phase::Ready) {
            return;
        }
        let mut page = Page::new(None);
        page.title = "Nouvelle base de données".into();
        page.emoji = "🗂️".into();
        page.collection = Some(Collection::default());
        page.wide = true;
        self.add_page(page);
    }

    pub fn create_record(&mut self, status: Option<Status>, tx: &mut Tx) {
        let Some(parent) = self
            .current()
            .filter(|p| p.collection.is_some() && !p.archived)
        else {
            return;
        };
        let mut page = Page::new(Some(parent.id.clone()));
        page.properties.status = status
            .or_else(|| parent.collection.as_ref().and_then(|c| c.status))
            .unwrap_or_default();
        self.add_page(page);
        tx.focus("page-title", false);
        tx.scroll_to_y("document-scroll", 0.0);
    }

    pub fn collection_items(&self) -> Vec<&Page> {
        let Some(parent) = self.current() else {
            return Vec::new();
        };
        let Some(settings) = &parent.collection else {
            return Vec::new();
        };
        let query = self.collection_search.text.trim();
        let mut items: Vec<_> = self
            .pages
            .iter()
            .filter(|p| {
                p.parent_id.as_deref() == Some(&parent.id)
                    && !p.archived
                    && settings.filter().matches(p, &self.today)
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
            let order = match settings.sort {
                Sort::Title => a
                    .display_title()
                    .to_lowercase()
                    .cmp(&b.display_title().to_lowercase()),
                Sort::Updated => b.updated_at.cmp(&a.updated_at),
                Sort::Due => match (&a.properties.due, &b.properties.due) {
                    (Some(a), Some(b)) => a.cmp(b),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                },
                Sort::Priority => b.properties.priority.cmp(&a.properties.priority),
            };
            order
                .then_with(|| a.display_title().cmp(b.display_title()))
                .then_with(|| a.id.cmp(&b.id))
        });
        items
    }

    fn change_collection(&mut self, change: impl FnOnce(&mut Collection)) {
        let Some(id) = self.selected.clone() else {
            return;
        };
        let Some(page) = self.pages.iter_mut().find(|p| p.id == id) else {
            return;
        };
        let Some(collection) = &mut page.collection else {
            return;
        };
        change(collection);
        collection.sync_view();
        page.updated_at = now();
        self.panel = Panel::Closed;
        self.persist(&id);
    }

    pub fn collection_view(&mut self, view: View) {
        self.change_collection(|c| c.view = view);
    }
    pub fn collection_sort(&mut self, sort: Sort) {
        self.change_collection(|c| c.sort = sort);
    }

    pub fn set_status(&mut self, id: &str, status: Status) {
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.properties.status = status;
            page.updated_at = now();
            self.persist(id);
        }
        self.panel = Panel::Closed;
    }

    pub fn set_priority(&mut self, id: &str, priority: Priority) {
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.properties.priority = priority;
            page.updated_at = now();
            self.persist(id);
        }
        self.panel = Panel::Closed;
    }

    pub fn open_properties(&mut self, id: &str, anchor: &str) {
        if let Some(page) = self.pages.iter().find(|p| p.id == id) {
            self.panel = Panel::Properties(Box::new(PropertyForm {
                id: id.into(),
                anchor: anchor.into(),
                due: TextEdit::new(page.properties.due.as_deref().unwrap_or("")),
                tags: TextEdit::new(page.properties.tags.join(", ")),
                error: None,
            }));
        }
    }

    pub fn property_edit(&mut self, due: bool) -> &mut TextEdit {
        let Panel::Properties(form) = &mut self.panel else {
            panic!("the properties form owns its fields");
        };
        if due { &mut form.due } else { &mut form.tags }
    }

    pub fn save_properties(&mut self) {
        let Panel::Properties(form) = &self.panel else {
            return;
        };
        let due = form.due.text.trim().to_string();
        let tags = parse_tags(&form.tags.text);
        let error = if !due.is_empty() && !valid_date(&due) {
            Some("Échéance invalide. Utilisez une date réelle au format AAAA-MM-JJ.".into())
        } else {
            tags.as_ref().err().cloned()
        };
        if let Some(error) = error {
            if let Panel::Properties(form) = &mut self.panel {
                form.error = Some(error);
            }
            return;
        }
        let id = form.id.clone();
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.properties.due = (!due.is_empty()).then_some(due);
            page.properties.tags = tags.unwrap_or_default();
            page.updated_at = now();
            self.persist(&id);
        }
        self.panel = Panel::Closed;
    }
}
