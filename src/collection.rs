use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Status {
    #[default]
    Todo,
    InProgress,
    Done,
}
impl Status {
    pub const ALL: [Self; 3] = [Self::Todo, Self::InProgress, Self::Done];
    pub fn name(self) -> &'static str {
        match self {
            Self::Todo => "To do",
            Self::InProgress => "In progress",
            Self::Done => "Done",
        }
    }
    pub fn next(self, forward: bool) -> Self {
        match (self, forward) {
            (Self::Todo, true) => Self::InProgress,
            (Self::InProgress, true) => Self::Done,
            (Self::Done, false) => Self::InProgress,
            (Self::InProgress, false) => Self::Todo,
            _ => self,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}
impl Priority {
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];
    pub fn name(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Properties {
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub custom: std::collections::BTreeMap<String, crate::fields::CellValue>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum View {
    #[default]
    Table,
    Board,
    Dashboard,
    Calendar,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Sort {
    #[default]
    Title,
    Updated,
    Due,
    Priority,
}
impl Sort {
    pub const ALL: [Self; 4] = [Self::Title, Self::Updated, Self::Due, Self::Priority];
    pub fn name(self) -> &'static str {
        match self {
            Self::Title => "Name A → Z",
            Self::Updated => "Recently updated",
            Self::Due => "Due date, earliest first",
            Self::Priority => "Priority, highest first",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    #[serde(default)]
    pub view: View,
    #[serde(default)]
    pub sort: Sort,
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub deadline: crate::dashboard::Deadline,
    #[serde(default)]
    pub columns: Vec<crate::fields::Column>,
    #[serde(default)]
    pub hidden: Vec<BuiltinColumn>,
    #[serde(default)]
    pub views: Vec<SavedView>,
    #[serde(default)]
    pub active_view: Option<String>,
    #[serde(default)]
    pub layout: TableLayout,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedView {
    pub id: String,
    pub name: String,
    pub view: View,
    pub sort: Sort,
    pub filter: crate::dashboard::TrackingFilter,
    #[serde(default)]
    pub layout: Option<TableLayout>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BuiltinColumn {
    Status,
    Priority,
    Due,
    Tags,
}
impl BuiltinColumn {
    pub const ALL: [Self; 4] = [Self::Status, Self::Priority, Self::Due, Self::Tags];
    pub fn name(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Priority => "Priority",
            Self::Due => "Due date",
            Self::Tags => "Tags",
        }
    }
    pub fn width(self) -> f32 {
        match self {
            Self::Status => 112.0,
            Self::Priority => 104.0,
            Self::Due => 102.0,
            Self::Tags => 122.0,
        }
    }
}
impl Collection {
    pub fn active(&self) -> Option<&SavedView> {
        self.views
            .iter()
            .find(|v| Some(&v.id) == self.active_view.as_ref())
    }

    /// Working settings keep old documents readable; an active view follows every change.
    pub fn sync_view(&mut self) {
        let filter = self.filter();
        let layout = self.capture_layout();
        if let Some(view) = self
            .views
            .iter_mut()
            .find(|v| Some(&v.id) == self.active_view.as_ref())
        {
            view.view = self.view;
            view.sort = self.sort;
            view.filter = filter;
            view.layout = Some(layout);
        }
    }

    pub fn select_view(&mut self, id: &str) -> bool {
        self.sync_view();
        let Some(view) = self.views.iter().find(|v| v.id == id).cloned() else {
            return false;
        };
        self.view = view.view;
        self.sort = view.sort;
        self.status = view.filter.status;
        self.priority = view.filter.priority;
        self.deadline = view.filter.deadline;
        if let Some(layout) = view.layout {
            self.apply_layout(layout);
        }
        self.active_view = Some(view.id);
        true
    }

    pub fn save_view(&mut self, name: &str, rename: bool) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 40 {
            return Err("The name must contain 1 to 40 characters.".into());
        }
        if self.views.iter().any(|v| {
            v.name.to_lowercase() == name.to_lowercase()
                && !(rename && Some(&v.id) == self.active_view.as_ref())
        }) {
            return Err("A view with this name already exists.".into());
        }
        if rename {
            let id = self.active_view.as_ref();
            let view = self
                .views
                .iter_mut()
                .find(|v| Some(&v.id) == id)
                .ok_or("Choose a saved view first.")?;
            view.name = name.into();
            return Ok(());
        }
        if self.views.len() >= 20 {
            return Err("This database already has 20 views.".into());
        }
        self.sync_view();
        if self.views.is_empty() {
            self.push_view(if name.to_lowercase() == "main view" {
                "Initial view"
            } else {
                "Main view"
            });
        }
        self.push_view(name);
        Ok(())
    }

    fn push_view(&mut self, name: &str) {
        let id = uuid::Uuid::new_v4().to_string();
        self.views.push(SavedView {
            id: id.clone(),
            name: name.into(),
            view: self.view,
            sort: self.sort,
            filter: self.filter(),
            layout: Some(self.capture_layout()),
        });
        self.active_view = Some(id);
    }

    pub fn remove_view(&mut self, id: &str) -> bool {
        if self.views.len() <= 1 || !self.views.iter().any(|v| v.id == id) {
            return false;
        }
        if self.active_view.as_deref() == Some(id) {
            let next = self.views.iter().find(|v| v.id != id).unwrap().id.clone();
            self.select_view(&next);
        }
        self.views.retain(|v| v.id != id);
        true
    }

    pub fn filter(&self) -> crate::dashboard::TrackingFilter {
        crate::dashboard::TrackingFilter {
            status: self.status,
            priority: self.priority,
            deadline: self.deadline,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableLayout {
    pub order: Vec<String>,
    pub widths: std::collections::BTreeMap<String, u16>,
    pub hidden: Vec<String>,
}
impl Collection {
    pub fn capture_layout(&self) -> TableLayout {
        let mut layout = self.layout.clone();
        layout.hidden = self
            .hidden
            .iter()
            .map(|c| format!("{c:?}"))
            .chain(
                self.columns
                    .iter()
                    .filter(|c| !c.visible)
                    .map(|c| c.id.clone()),
            )
            .collect();
        layout
    }
    pub fn apply_layout(&mut self, layout: TableLayout) {
        self.hidden = BuiltinColumn::ALL
            .into_iter()
            .filter(|c| layout.hidden.contains(&format!("{c:?}")))
            .collect();
        for c in &mut self.columns {
            c.visible = !layout.hidden.contains(&c.id);
        }
        self.layout = layout;
    }
    pub fn width(&self, id: &str, default: f32) -> f32 {
        self.layout
            .widths
            .get(id)
            .copied()
            .map_or(default, |w| w.clamp(70, 500) as f32)
    }
}

pub fn valid_date(value: &str) -> bool {
    if value.len() != 10 || value.as_bytes()[4] != b'-' || value.as_bytes()[7] != b'-' {
        return false;
    }
    if !value
        .bytes()
        .enumerate()
        .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        value[..4].parse::<u32>(),
        value[5..7].parse::<u32>(),
        value[8..].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    year > 0 && day > 0 && day <= days
}

pub fn parse_tags(value: &str) -> Result<Vec<String>, String> {
    let mut tags: Vec<String> = Vec::new();
    for tag in value
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
    {
        if tag.chars().count() > 40 {
            return Err("Each tag must be at most 40 characters long.".into());
        }
        if !tags
            .iter()
            .any(|existing| existing.to_lowercase() == tag.to_lowercase())
        {
            tags.push(tag.into());
        }
    }
    if tags.len() > 12 {
        return Err("Use no more than 12 tags per record.".into());
    }
    Ok(tags)
}
