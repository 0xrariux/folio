use crate::state::{Panel, State};
use folio::model::now;
use rune::TextEdit;

#[derive(Clone)]
pub struct ViewForm {
    pub name: TextEdit,
    pub rename: bool,
    pub error: Option<String>,
}

impl State {
    pub fn open_views(&mut self, rename: bool) {
        let Some(settings) = self.current().and_then(|p| p.collection.as_ref()) else {
            return;
        };
        self.panel = Panel::Views(Box::new(ViewForm {
            name: TextEdit::new(if rename {
                settings.active().map_or("", |v| &v.name)
            } else {
                ""
            }),
            rename,
            error: None,
        }));
    }
    pub fn view_name_edit(&mut self) -> &mut TextEdit {
        let Panel::Views(form) = &mut self.panel else {
            unreachable!()
        };
        &mut form.name
    }
    pub fn save_view(&mut self) {
        let Panel::Views(form) = &self.panel else {
            return;
        };
        let (name, rename) = (form.name.text.clone(), form.rename);
        let Some(page) = self
            .pages
            .iter_mut()
            .find(|p| Some(&p.id) == self.selected.as_ref())
        else {
            return;
        };
        let Some(settings) = &mut page.collection else {
            return;
        };
        if let Err(error) = settings.save_view(&name, rename) {
            if let Panel::Views(form) = &mut self.panel {
                form.error = Some(error);
            }
            return;
        }
        page.updated_at = now();
        let id = page.id.clone();
        self.persist(&id);
        self.panel = Panel::Closed;
    }
    pub fn select_view(&mut self, id: &str) {
        self.update_saved_view(|c| c.select_view(id));
    }
    pub fn remove_view(&mut self, id: &str) {
        self.update_saved_view(|c| c.remove_view(id));
    }
    fn update_saved_view(
        &mut self,
        change: impl FnOnce(&mut folio::collection::Collection) -> bool,
    ) {
        let Some(page) = self
            .pages
            .iter_mut()
            .find(|p| Some(&p.id) == self.selected.as_ref())
        else {
            return;
        };
        let Some(settings) = &mut page.collection else {
            return;
        };
        if !change(settings) {
            return;
        }
        page.updated_at = now();
        let id = page.id.clone();
        self.collection_search = TextEdit::new("");
        self.panel = Panel::Closed;
        self.persist(&id);
    }
}
