use crate::{
    collection::{Priority, Status, valid_date},
    fields::{CellValue, ColumnKind},
    model::Page,
};
use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Deadline {
    #[default]
    All,
    Overdue,
    NextWeek,
    NoDate,
}
impl Deadline {
    pub const ALL: [Self; 4] = [Self::All, Self::Overdue, Self::NextWeek, Self::NoDate];
    pub fn name(self) -> &'static str {
        match self {
            Self::All => "Toutes les échéances",
            Self::Overdue => "En retard",
            Self::NextWeek => "Dans les 7 jours",
            Self::NoDate => "Sans échéance",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackingFilter {
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub deadline: Deadline,
}
impl TrackingFilter {
    pub fn matches(&self, page: &Page, today: &str) -> bool {
        self.status.is_none_or(|v| page.properties.status == v)
            && self.priority.is_none_or(|v| page.properties.priority == v)
            && match self.deadline {
                Deadline::All => true,
                Deadline::Overdue => overdue(page, today),
                Deadline::NextWeek => due_soon(page, today),
                Deadline::NoDate => page.properties.due.is_none(),
            }
    }
    pub fn active(&self) -> bool {
        self.status.is_some() || self.priority.is_some() || self.deadline != Deadline::All
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    #[default]
    All,
    Selected(Vec<String>),
}
impl Scope {
    pub fn contains(&self, id: &str) -> bool {
        match self {
            Self::All => true,
            Self::Selected(ids) => ids.iter().any(|v| v == id),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardConfig {
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub filter: TrackingFilter,
}

pub fn local_today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
pub fn date(value: &str) -> Option<NaiveDate> {
    valid_date(value)
        .then(|| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .flatten()
}
pub fn overdue(page: &Page, today: &str) -> bool {
    page.properties.status != Status::Done
        && page
            .properties
            .due
            .as_deref()
            .and_then(date)
            .zip(date(today))
            .is_some_and(|(due, today)| due < today)
}
pub fn due_soon(page: &Page, today: &str) -> bool {
    page.properties.status != Status::Done
        && page
            .properties
            .due
            .as_deref()
            .and_then(date)
            .zip(date(today))
            .is_some_and(|(due, today)| {
                today
                    .checked_add_days(Days::new(7))
                    .is_some_and(|end| due >= today && due <= end)
            })
}

#[derive(Debug, Default)]
pub struct Summary {
    pub total: usize,
    pub done: usize,
    pub in_progress: usize,
    pub overdue: usize,
    pub soon: usize,
    pub high: usize,
}
impl Summary {
    pub fn from_pages(items: &[&Page], today: &str) -> Self {
        let mut stats = Self {
            total: items.len(),
            ..Self::default()
        };
        for page in items {
            stats.done += usize::from(page.properties.status == Status::Done);
            stats.in_progress += usize::from(page.properties.status == Status::InProgress);
            stats.overdue += usize::from(overdue(page, today));
            stats.soon += usize::from(due_soon(page, today));
            stats.high += usize::from(
                page.properties.priority == Priority::High
                    && page.properties.status != Status::Done,
            );
        }
        stats
    }
    pub fn progress(&self) -> usize {
        (self.done * 100).checked_div(self.total).unwrap_or(0)
    }
}

pub struct FieldSummary {
    pub collection_id: String,
    pub column_id: String,
    pub label: String,
    pub base: String,
    pub kind: ColumnKind,
    pub sum: Option<f64>,
    pub filled: usize,
    pub checked: usize,
    pub records: usize,
}
pub fn field_summaries(sources: &[&Page], items: &[&Page]) -> Vec<FieldSummary> {
    let mut result = Vec::new();
    for source in sources {
        let Some(collection) = &source.collection else {
            continue;
        };
        let records: Vec<_> = items
            .iter()
            .filter(|p| p.parent_id.as_deref() == Some(&source.id))
            .collect();
        for column in collection
            .columns
            .iter()
            .filter(|c| c.visible && matches!(c.kind, ColumnKind::Number | ColumnKind::Checkbox))
        {
            let mut summary = FieldSummary {
                collection_id: source.id.clone(),
                column_id: column.id.clone(),
                label: column.name.clone(),
                base: source.display_title().into(),
                kind: column.kind,
                sum: Some(0.0),
                filled: 0,
                checked: 0,
                records: records.len(),
            };
            for page in &records {
                match page.properties.custom.get(&column.id) {
                    Some(value) if column.kind == ColumnKind::Number => {
                        if let Some(n) = value.number() {
                            summary.filled += 1;
                            summary.sum = summary
                                .sum
                                .and_then(|sum| (sum + n).is_finite().then_some(sum + n));
                        }
                    }
                    Some(CellValue::Checkbox(value)) if column.kind == ColumnKind::Checkbox => {
                        summary.filled += 1;
                        summary.checked += usize::from(*value);
                    }
                    _ => {}
                }
            }
            result.push(summary);
        }
    }
    result
}

pub fn calculated_summaries(
    sources: &[&Page],
    items: &[&Page],
    pages: &[Page],
) -> Vec<FieldSummary> {
    let mut fields = field_summaries(sources, items);
    for source in sources {
        for column in source
            .collection
            .iter()
            .flat_map(|c| &c.columns)
            .filter(|c| c.visible && matches!(c.kind, ColumnKind::Formula | ColumnKind::Rollup))
        {
            let records: Vec<_> = items
                .iter()
                .filter(|p| p.parent_id.as_deref() == Some(&source.id))
                .collect();
            let values: Vec<_> = records
                .iter()
                .filter_map(|p| crate::calculations::value(p, column, pages).ok())
                .collect();
            let total = values.iter().sum::<f64>();
            fields.push(FieldSummary {
                collection_id: source.id.clone(),
                column_id: column.id.clone(),
                label: column.name.clone(),
                base: source.display_title().into(),
                kind: column.kind,
                sum: total.is_finite().then_some(total),
                filled: values.len(),
                checked: 0,
                records: records.len(),
            });
        }
    }
    fields
}
