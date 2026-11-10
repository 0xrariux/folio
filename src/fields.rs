use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
            Self::Text => "Texte",
            Self::Number => "Nombre",
            Self::Date => "Date",
            Self::Checkbox => "Case à cocher",
            Self::Relation => "Relation",
            Self::Rollup => "Agrégation",
            Self::Formula => "Formule",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
            Self::Checkbox(v) => if *v { "Oui" } else { "Non" }.into(),
            Self::Relation(ids) => format!("{} liens", ids.len()),
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
                return Err("Ce texte doit faire au plus 1 000 caractères.".into());
            }
            Ok(Some(CellValue::Text(text.into())))
        }
        ColumnKind::Number => {
            let value = text.trim().replace(',', ".");
            if value.len() > 64 || value.parse::<f64>().ok().is_none_or(|v| !v.is_finite()) {
                return Err("Saisissez un nombre valide et fini, par exemple 12,5.".into());
            }
            Ok(Some(CellValue::Number(value)))
        }
        ColumnKind::Date => {
            if !crate::collection::valid_date(text.trim()) {
                return Err("Utilisez une date réelle au format AAAA-MM-JJ.".into());
            }
            Ok(Some(CellValue::Date(text.trim().into())))
        }
        ColumnKind::Checkbox => {
            Err("Une case à cocher se modifie directement dans le tableau.".into())
        }
        ColumnKind::Relation | ColumnKind::Rollup | ColumnKind::Formula => {
            Err("Cette propriété utilise son sélecteur dédié.".into())
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
            Self::Count => "Nombre de fiches",
            Self::Done => "Fiches terminées",
            Self::Progress => "Progression (%)",
            Self::Sum => "Somme d’une propriété",
            Self::Remaining => "Fiches restantes",
        }
    }
}

pub fn format_number(number: f64) -> String {
    if number.abs() >= 1e12 {
        return format!("{number:.2e}").replace('.', ",");
    }
    let rounded = format!("{number:.2}");
    let value = rounded.trim_end_matches('0').trim_end_matches('.');
    if value == "-0" {
        "0".into()
    } else {
        value.replace('.', ",")
    }
}
