use crate::state::State;
use folio::{
    collection::{Collection, Priority, Status, View},
    dashboard::DashboardConfig,
    fields::{CellValue, Column, ColumnKind},
    model::Page,
};

impl State {
    pub fn demo_lists(&mut self) {
        if !self.demo {
            return;
        }
        let mut page = folio::templates::Template::Outline.create();
        page.id = "demo-lists".into();
        for (i, b) in page.blocks.iter_mut().enumerate() {
            b.id = format!("demo-list-{i}");
        }
        page.sync_document();
        self.pages.push(page);
        self.select("demo-lists");
    }
    pub fn demo_control_catalog(&mut self) {
        if !self.demo {
            return;
        }
        let mut page = Page::new(None);
        page.id = "demo-control-catalog".into();
        page.title = "Interface icons".into();
        page.emoji = "📝".into();
        page.wide = true;
        page.replace_html("<p>Shared icons for navigation, organization, and writing.</p>");
        self.pages.push(page);
        self.select("demo-control-catalog");
    }

    pub fn demo_icon_catalog(&mut self) {
        if !self.demo {
            return;
        }
        let mut page = Page::new(None);
        page.id = "demo-icon-catalog".into();
        page.title = "Page icons".into();
        page.emoji = "🌿".into();
        page.wide = true;
        page.replace_html("<p>Sixteen icons for ideas, notes, and projects.</p>");
        self.pages.push(page);
        self.select("demo-icon-catalog");
    }

    pub fn demo_drag_list(&mut self) {
        if !self.demo {
            return;
        }
        use folio::document::{Kind, NoteBlock};
        let mut page = Page::new(None);
        page.id = "demo-drag-list".into();
        page.title = "Organize your tasks".into();
        page.emoji = "✅".into();
        page.blocks = vec![
            NoteBlock::text(
                Kind::Callout,
                "Drag the handle to the left of a block and drop it on the green marker. Press Escape to cancel.",
            ),
            NoteBlock::text(Kind::Heading2, "Prepare the next release"),
            NoteBlock::text(Kind::Numbered, "Gather ideas and feedback"),
            NoteBlock::text(Kind::Numbered, "Choose the next priorities"),
            NoteBlock::text(Kind::Numbered, "Share a first version"),
            NoteBlock::text(Kind::Todo, "Set up the project"),
            NoteBlock::text(Kind::Todo, "Reorder this list by dragging"),
            NoteBlock::text(
                Kind::Bullet,
                "Content, formatting, and checked tasks move with the block.",
            ),
        ];
        page.blocks[5].checked = true;
        for (i, block) in page.blocks.iter_mut().enumerate() {
            block.id = format!("demo-drag-{i}");
        }
        page.sync_document();
        self.pages.push(page);
        self.select("demo-drag-list");
    }

    pub fn demo_saved_views(&mut self) {
        if !self.demo {
            return;
        }
        use folio::{
            collection::{SavedView, Sort},
            dashboard::{Deadline, TrackingFilter},
        };
        self.select("demo-tracker");
        let settings = self
            .pages
            .iter_mut()
            .find(|p| p.id == "demo-tracker")
            .unwrap()
            .collection
            .as_mut()
            .unwrap();
        settings.view = View::Table;
        settings.views = vec![
            SavedView {
                layout: None,
                id: "demo-view-all".into(),
                name: "All projects".into(),
                view: View::Table,
                sort: Sort::Title,
                filter: TrackingFilter::default(),
            },
            SavedView {
                layout: None,
                id: "demo-view-active".into(),
                name: "In progress".into(),
                view: View::Board,
                sort: Sort::Due,
                filter: TrackingFilter {
                    status: Some(Status::InProgress),
                    ..Default::default()
                },
            },
            SavedView {
                layout: None,
                id: "demo-view-late".into(),
                name: "Needs attention".into(),
                view: View::Dashboard,
                sort: Sort::Due,
                filter: TrackingFilter {
                    deadline: Deadline::Overdue,
                    ..Default::default()
                },
            },
        ];
        settings.active_view = Some("demo-view-all".into());
    }

    pub fn extend_demo_tracking(&mut self) {
        let mut hours = Column::new("Effort (h)", ColumnKind::Number);
        hours.id = "demo-hours".into();
        let mut approved = Column::new("Approved", ColumnKind::Checkbox);
        approved.id = "demo-approved".into();
        let mut owner = Column::new("Owner", ColumnKind::Text);
        owner.id = "demo-owner".into();
        if let Some(base) = self.pages.iter_mut().find(|p| p.id == "demo-tracker")
            && let Some(collection) = &mut base.collection
        {
            collection.columns = vec![hours, approved, owner];
        }
        let mut late = Page::new(Some("demo-tracker".into()));
        late.id = "demo-record-5".into();
        late.title = "Review the project scope".into();
        late.properties.priority = Priority::High;
        late.properties.due = Some("2026-11-08".into());
        late.properties.tags = vec!["Product".into()];
        late.blocks[0].id = "demo-record-5-0".into();
        self.pages.push(late);
        for (i, hours) in [8, 16, 5, 20, 3, 4].into_iter().enumerate() {
            if let Some(page) = self
                .pages
                .iter_mut()
                .find(|p| p.id == format!("demo-record-{i}"))
            {
                page.properties
                    .custom
                    .insert("demo-hours".into(), CellValue::Number(hours.to_string()));
                page.properties.custom.insert(
                    "demo-approved".into(),
                    CellValue::Checkbox(page.properties.status == Status::Done),
                );
                page.properties
                    .custom
                    .insert("demo-owner".into(), CellValue::Text("Sample user".into()));
            }
        }
        let mut points = Column::new("Estimated points", ColumnKind::Number);
        points.id = "demo-points".into();
        let mut launch = Page::new(None);
        launch.id = "demo-launch".into();
        launch.title = "Sample launch".into();
        launch.emoji = "🚀".into();
        launch.wide = true;
        launch.collection = Some(Collection {
            view: View::Table,
            columns: vec![points],
            ..Default::default()
        });
        launch.blocks[0].id = "demo-launch-0".into();
        self.pages.push(launch);
        self.collapsed.insert("demo-launch".into());
        for (i, (title, status, due, points)) in [
            ("Prepare the presentation", Status::Todo, "2026-11-10", 3),
            ("Test the workflows", Status::InProgress, "2026-11-13", 5),
            ("Share the first version", Status::Done, "2026-11-15", 8),
        ]
        .into_iter()
        .enumerate()
        {
            let mut page = Page::new(Some("demo-launch".into()));
            page.id = format!("demo-launch-record-{i}");
            page.title = title.into();
            page.properties.status = status;
            page.properties.due = Some(due.into());
            page.properties
                .custom
                .insert("demo-points".into(), CellValue::Number(points.to_string()));
            page.blocks[0].id = format!("{}-0", page.id);
            self.pages.push(page);
        }
        let mut dashboard = Page::new(None);
        dashboard.id = "demo-dashboard".into();
        dashboard.title = "Project overview".into();
        dashboard.emoji = "📊".into();
        dashboard.wide = true;
        dashboard.dashboard = Some(DashboardConfig::default());
        dashboard.blocks[0].id = "demo-dashboard-0".into();
        self.pages.push(dashboard);
    }
}
