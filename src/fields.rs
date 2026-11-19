use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ColumnKind {
    #[default]
    Text,
    Number,
    Date,
    Checkbox,
    Relation,
    Rollup,
    Formula,
}
impl ColumnKind {
    pub const ALL: [Self; 7] = [
        Self::Text,
        Self::Number,
        Self::Date,
        Self::Checkbox,
        Self::Relation,
        Self::Rollup,
        Self::Formula,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Number => "Number",
            Self::Date => "Date",
            Self::Checkbox => "Checkbox",
            Self::Relation => "Relation",
            Self::Rollup => "Rollup",
            Self::Formula => "Formula",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub id: String,
    pub name: String,
    pub kind: ColumnKind,
    #[serde(default = "visible_default")]
    pub visible: bool,
    #[serde(default)]
    pub config: Calculation,
}
fn visible_default() -> bool {
    true
}
impl Column {
    pub fn new(name: &str, kind: ColumnKind) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            kind,
            visible: true,
            config: Calculation::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CellValue {
    Text(String),
    Number(String),
    Date(String),
    Checkbox(bool),
    Relation(Vec<String>),
}
impl CellValue {
    pub fn display(&self) -> String {
        match self {
            Self::Text(v) | Self::Number(v) | Self::Date(v) => v.clone(),
            Self::Checkbox(v) => if *v { "Yes" } else { "No" }.into(),
            Self::Relation(ids) => format!("{} links", ids.len()),
        }
    }
    pub fn number(&self) -> Option<f64> {
        if let Self::Number(value) = self {
            value.parse::<f64>().ok().filter(|n| n.is_finite())
        } else {
            None
        }
    }
}

pub fn parse_cell(kind: ColumnKind, text: &str) -> Result<Option<CellValue>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    match kind {
        ColumnKind::Text => {
            if text.chars().count() > 1000 {
                return Err("This text must be at most 1,000 characters long.".into());
            }
            Ok(Some(CellValue::Text(text.into())))
        }
        ColumnKind::Number => {
            let value = text.trim().replace(',', ".");
            if value.len() > 64 || value.parse::<f64>().ok().is_none_or(|v| !v.is_finite()) {
                return Err("Enter a valid, finite number, for example 12.5.".into());
            }
            Ok(Some(CellValue::Number(value)))
        }
        ColumnKind::Date => {
            if !crate::collection::valid_date(text.trim()) {
                return Err("Use a real date in YYYY-MM-DD format.".into());
            }
            Ok(Some(CellValue::Date(text.trim().into())))
        }
        ColumnKind::Checkbox => Err("Edit checkboxes directly in the table.".into()),
        ColumnKind::Relation | ColumnKind::Rollup | ColumnKind::Formula => {
            Err("This property has its own picker.".into())
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Calculation {
    #[serde(default)]
    pub reverse_of: Option<String>,
    #[serde(default)]
    pub auto_inverse: bool,
    pub target_base: Option<String>,
    pub relation: Option<String>,
    pub property: Option<String>,
    pub aggregate: Aggregate,
    pub expression: String,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Aggregate {
    #[default]
    Count,
    Done,
    Progress,
    Sum,
    Remaining,
}
impl Aggregate {
    pub const ALL: [Self; 5] = [
        Self::Count,
        Self::Done,
        Self::Progress,
        Self::Sum,
        Self::Remaining,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Count => "Record count",
            Self::Done => "Completed records",
            Self::Progress => "Progress (%)",
            Self::Sum => "Sum of a property",
            Self::Remaining => "Remaining records",
        }
    }
}

pub fn format_number(number: f64) -> String {
    if number.abs() >= 1e12 {
        return format!("{number:.2e}");
    }
    let rounded = format!("{number:.2}");
    let value = rounded.trim_end_matches('0').trim_end_matches('.');
    if value == "-0" {
        "0".into()
    } else {
        value.into()
    }
}
