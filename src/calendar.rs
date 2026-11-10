use crate::state::{Panel, State};
use chrono::{Datelike, Months, NaiveDate};
use rune::TextEdit;
#[derive(Clone)]
pub struct DatePicker {
    pub record: String,
    pub column: Option<String>,
    pub anchor: String,
    pub month: NaiveDate,
    pub value: TextEdit,
    pub error: Option<String>,
}
pub fn month(today: &str) -> NaiveDate {
    NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        .with_day(1)
        .unwrap()
}
pub fn days(month: NaiveDate) -> Vec<Option<NaiveDate>> {
    let first = month.with_day(1).unwrap();
    let next = first.checked_add_months(Months::new(1)).unwrap_or(first);
    let count = (next - first).num_days().clamp(28, 31) as u32;
    let mut days = vec![None; first.weekday().num_days_from_monday() as usize];
    days.extend((1..=count).map(|d| first.with_day(d)));
    while !days.len().is_multiple_of(7) {
        days.push(None);
    }
    days
}
impl State {
    pub fn calendar_month(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&format!("{}-01", self.workspace.calendar_month), "%Y-%m-%d")
            .unwrap_or_else(|_| month(&self.today))
    }
    pub fn step_calendar(&mut self, forward: bool) {
        let current = self.calendar_month();
        let next = if forward {
            current.checked_add_months(Months::new(1))
        } else {
            current.checked_sub_months(Months::new(1))
        };
        if let Some(next) = next {
            self.workspace.calendar_month = next.format("%Y-%m").to_string();
        }
    }
    pub fn open_date(&mut self, record: &str, column: Option<&str>, anchor: &str) {
        let Some(page) = self.pages.iter().find(|p| p.id == record) else {
            return;
        };
        let date = if let Some(column) = column {
            match page.properties.custom.get(column) {
                Some(folio::fields::CellValue::Date(v)) => Some(v.as_str()),
                _ => None,
            }
        } else {
            page.properties.due.as_deref()
        };
        self.panel = Panel::Date(Box::new(DatePicker {
            record: record.into(),
            column: column.map(str::to_string),
            anchor: anchor.into(),
            month: month(date.unwrap_or(&self.today)),
            value: TextEdit::new(date.unwrap_or("")),
            error: None,
        }));
    }
    pub fn set_due(&mut self, record: &str, column: Option<&str>, value: Option<&str>) {
        if value.is_some_and(|v| !folio::collection::valid_date(v)) {
            return;
        }
        if let Some(page) = self
            .pages
            .iter_mut()
            .find(|p| p.id == record && !p.archived)
        {
            if let Some(column) = column {
                if let Some(date) = value {
                    page.properties
                        .custom
                        .insert(column.into(), folio::fields::CellValue::Date(date.into()));
                } else {
                    page.properties.custom.remove(column);
                }
            } else {
                page.properties.due = value.map(str::to_string);
            }
            page.updated_at = folio::model::now();
            self.persist(record);
        }
    }
    pub fn pick_date(&mut self, date: Option<&str>) {
        let Panel::Date(form) = &self.panel else {
            return;
        };
        let (record, column) = (form.record.clone(), form.column.clone());
        if record == "__selection__" {
            let ids: Vec<_> = self.workspace.selected_records.iter().cloned().collect();
            for id in ids {
                self.set_due(&id, column.as_deref(), date);
            }
            self.workspace.selected_records.clear();
        } else {
            self.set_due(&record, column.as_deref(), date);
        }
        self.panel = Panel::Closed;
    }
}
