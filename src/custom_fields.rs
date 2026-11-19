use crate::state::{Panel, State};
use folio::{
    collection::BuiltinColumn,
    fields::{CellValue, Column, ColumnKind, parse_cell},
    model::now,
};
use rune::TextEdit;

#[derive(Clone)]
pub struct ColumnForm {
    pub collection_id: String,
    pub id: Option<String>,
    pub name: TextEdit,
    pub kind: ColumnKind,
    pub error: Option<String>,
    pub config: folio::fields::Calculation,
    pub expression: TextEdit,
}
#[derive(Clone)]
pub struct CellForm {
    pub record_id: String,
    pub column_id: String,
    pub anchor: String,
    pub value: TextEdit,
    pub error: Option<String>,
}

impl State {
    pub fn open_columns(&mut self) {
        let Some(id) = self
            .current()
            .filter(|p| p.collection.is_some())
            .map(|p| p.id.clone())
        else {
            return;
        };
        self.panel = Panel::Columns(Box::new(ColumnForm {
            collection_id: id,
            id: None,
            name: TextEdit::new(""),
            kind: ColumnKind::Text,
            error: None,
            config: Default::default(),
            expression: TextEdit::new(""),
        }));
    }
    pub fn column_name_edit(&mut self) -> &mut TextEdit {
        let Panel::Columns(form) = &mut self.panel else {
            panic!("a mounted column form owns its name");
        };
        &mut form.name
    }
    pub fn column_kind(&mut self, kind: ColumnKind) {
        if let Panel::Columns(form) = &mut self.panel
            && form.id.is_none()
        {
            if kind == ColumnKind::Relation && form.kind != kind {
                form.config.auto_inverse = true;
            }
            form.kind = kind;
        }
    }
    pub fn edit_column(&mut self, id: &str) {
        let Panel::Columns(form) = &self.panel else {
            return;
        };
        let Some(column) = self
            .pages
            .iter()
            .find(|p| p.id == form.collection_id)
            .and_then(|p| p.collection.as_ref())
            .and_then(|c| c.columns.iter().find(|c| c.id == id))
            .cloned()
        else {
            return;
        };
        if let Panel::Columns(form) = &mut self.panel {
            form.id = Some(column.id);
            form.name = TextEdit::new(column.name);
            form.kind = column.kind;
            form.error = None;
            form.config = column.config.clone();
            form.expression = TextEdit::new(&column.config.expression);
        }
    }
    pub fn save_column(&mut self) {
        let Panel::Columns(form) = &self.panel else {
            return;
        };
        let Some(settings) = self
            .pages
            .iter()
            .find(|p| p.id == form.collection_id)
            .and_then(|p| p.collection.as_ref())
        else {
            return;
        };
        let name = form.name.text.trim().to_string();
        let error = if name.is_empty() || name.chars().count() > 40 {
            Some("Give this column a name of 1 to 40 characters.")
        } else if settings.columns.iter().any(|c| {
            Some(&c.id) != form.id.as_ref() && c.name.to_lowercase() == name.to_lowercase()
        }) {
            Some("A column with this name already exists.")
        } else if form.id.is_none() && settings.columns.len() >= 12 {
            Some("This database already has 12 custom columns.")
        } else {
            None
        };
        if let Some(error) = error {
            if let Panel::Columns(form) = &mut self.panel {
                form.error = Some(error.into());
            }
            return;
        }
        let collection_id = form.collection_id.clone();
        let id = form.id.clone();
        let kind = form.kind;
        let mut config = form.config.clone();
        config.expression = form.expression.text.trim().into();
        let inverse_target =
            if form.id.is_none() && kind == ColumnKind::Relation && config.auto_inverse {
                config.target_base.clone()
            } else {
                None
            };
        if let Some(target) = &inverse_target {
            let count = self
                .pages
                .iter()
                .find(|p| p.id == *target)
                .and_then(|p| p.collection.as_ref())
                .map_or(12, |c| c.columns.len());
            if count + if target == &collection_id { 2 } else { 1 } > 12 {
                if let Panel::Columns(f) = &mut self.panel {
                    f.error =
                        Some("The target database has no room for a reverse relation.".into());
                }
                return;
            }
        }
        let mut created = None;
        if kind == ColumnKind::Relation
            && config.target_base.as_ref().is_none_or(|id| {
                !self
                    .pages
                    .iter()
                    .any(|p| p.id == *id && p.collection.is_some() && !p.archived)
            })
        {
            if let Panel::Columns(f) = &mut self.panel {
                f.error = Some("Choose the database to link.".into());
            }
            return;
        }
        if kind == ColumnKind::Rollup
            && config.relation.as_ref().is_none_or(|id| {
                !settings
                    .columns
                    .iter()
                    .any(|c| c.id == *id && c.kind == ColumnKind::Relation)
            })
        {
            if let Panel::Columns(f) = &mut self.panel {
                f.error = Some("Choose a relation property.".into());
            }
            return;
        }
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == collection_id)
            && let Some(settings) = &mut page.collection
        {
            if let Some(id) = id {
                if let Some(column) = settings.columns.iter_mut().find(|c| c.id == id) {
                    column.name = name;
                    column.config = config;
                }
            } else {
                let mut column = Column::new(&name, kind);
                column.config = config;
                created = Some(column.id.clone());
                settings.columns.push(column);
            }
            page.updated_at = now();
            self.persist(&collection_id);
        }
        if let (Some(target), Some(forward)) = (inverse_target, created) {
            let source_name = self
                .pages
                .iter()
                .find(|p| p.id == collection_id)
                .map_or("Relations".into(), |p| {
                    p.display_title().chars().take(28).collect::<String>()
                });
            if let Some(settings) = self
                .pages
                .iter_mut()
                .find(|p| p.id == target)
                .and_then(|p| p.collection.as_mut())
            {
                let mut name = format!("{source_name} (links)");
                let mut n = 2;
                while settings
                    .columns
                    .iter()
                    .any(|c| c.name.eq_ignore_ascii_case(&name))
                {
                    name = format!("{source_name} (links {n})");
                    n += 1;
                }
                let mut inverse = Column::new(&name, ColumnKind::Relation);
                inverse.config.target_base = Some(collection_id);
                inverse.config.reverse_of = Some(forward);
                settings.columns.push(inverse);
            }
            self.persist(&target);
        }
        self.panel = Panel::Closed;
    }
    pub fn toggle_column(&mut self, id: &str) {
        let Some(collection_id) = self.current().map(|p| p.id.clone()) else {
            return;
        };
        if let Some(column) = self
            .pages
            .iter_mut()
            .find(|p| p.id == collection_id)
            .and_then(|p| p.collection.as_mut())
            .and_then(|c| c.columns.iter_mut().find(|c| c.id == id))
        {
            column.visible = !column.visible;
            if let Some(settings) = self
                .pages
                .iter_mut()
                .find(|p| p.id == collection_id)
                .and_then(|p| p.collection.as_mut())
            {
                settings.sync_view();
            }
            self.persist(&collection_id);
        }
    }
    pub fn toggle_builtin(&mut self, column: BuiltinColumn) {
        let Some(id) = self.selected.clone() else {
            return;
        };
        if let Some(settings) = self
            .pages
            .iter_mut()
            .find(|p| p.id == id)
            .and_then(|p| p.collection.as_mut())
        {
            if settings.hidden.contains(&column) {
                settings.hidden.retain(|c| *c != column);
            } else {
                settings.hidden.push(column);
            }
            settings.sync_view();
            self.persist(&id);
        }
    }
    pub fn field_column(&self, record: &str, column: &str) -> Option<&Column> {
        self.pages
            .iter()
            .find(|p| p.id == record)
            .and_then(|p| p.parent_id.as_deref())
            .and_then(|id| self.pages.iter().find(|p| p.id == id))
            .and_then(|p| p.collection.as_ref())
            .and_then(|c| c.columns.iter().find(|c| c.id == column))
    }
    pub fn open_cell(&mut self, record: &str, column: &str, anchor: &str) {
        let Some(field) = self.field_column(record, column) else {
            return;
        };
        if field.kind == ColumnKind::Relation {
            self.open_palette(crate::command_palette::Mode::Relation {
                record: record.into(),
                column: column.into(),
            });
            return;
        }
        if matches!(field.kind, ColumnKind::Rollup | ColumnKind::Formula) {
            return;
        }
        if field.kind == ColumnKind::Date {
            self.open_date(record, Some(column), anchor);
            return;
        }
        let value = self
            .pages
            .iter()
            .find(|p| p.id == record)
            .and_then(|p| p.properties.custom.get(column))
            .map(CellValue::display)
            .unwrap_or_default();
        self.panel = Panel::Cell(Box::new(CellForm {
            record_id: record.into(),
            column_id: column.into(),
            anchor: anchor.into(),
            value: TextEdit::new(value),
            error: None,
        }));
    }
    pub fn cell_edit(&mut self) -> &mut TextEdit {
        let Panel::Cell(form) = &mut self.panel else {
            panic!("a mounted cell form owns its value");
        };
        &mut form.value
    }
    pub fn save_cell(&mut self) {
        let Panel::Cell(form) = &self.panel else {
            return;
        };
        let result = self
            .field_column(&form.record_id, &form.column_id)
            .ok_or_else(|| "This column is no longer available for this record.".to_string())
            .and_then(|c| parse_cell(c.kind, &form.value.text));
        let value = match result {
            Ok(value) => value,
            Err(error) => {
                if let Panel::Cell(form) = &mut self.panel {
                    form.error = Some(error);
                }
                return;
            }
        };
        let id = form.record_id.clone();
        let column = form.column_id.clone();
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            if let Some(value) = value {
                page.properties.custom.insert(column, value);
            } else {
                page.properties.custom.remove(&column);
            }
            page.updated_at = now();
            self.persist(&id);
        }
        self.panel = Panel::Closed;
    }
    pub fn check_cell(&mut self, record: &str, column: &str) {
        if self
            .field_column(record, column)
            .is_none_or(|c| c.kind != ColumnKind::Checkbox)
        {
            return;
        }
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == record) {
            let checked = matches!(
                page.properties.custom.get(column),
                Some(CellValue::Checkbox(true))
            );
            page.properties
                .custom
                .insert(column.into(), CellValue::Checkbox(!checked));
            page.updated_at = now();
            self.persist(record);
        }
    }
}
