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
        page.title = "Les signes de l’interface".into();
        page.emoji = "📝".into();
        page.wide = true;
        page.replace_html("<p>Des repères communs pour naviguer, organiser et écrire.</p>");
        self.pages.push(page);
        self.select("demo-control-catalog");
    }

    pub fn demo_icon_catalog(&mut self) {
        if !self.demo {
            return;
        }
        let mut page = Page::new(None);
        page.id = "demo-icon-catalog".into();
        page.title = "La collection Folio".into();
        page.emoji = "🌿".into();
        page.wide = true;
        page.replace_html("<p>Seize signes pour les idées, les notes et les projets.</p>");
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
        page.title = "Chaque chose à sa place".into();
        page.emoji = "✅".into();
        page.blocks = vec![
            NoteBlock::text(
                Kind::Callout,
                "Attrapez la poignée ⠿ à gauche d’un bloc, déplacez-le, puis relâchez sur le repère vert. Échap annule le geste.",
            ),
            NoteBlock::text(Kind::Heading2, "Préparer la prochaine version"),
            NoteBlock::text(Kind::Numbered, "Rassembler les idées et les retours"),
            NoteBlock::text(Kind::Numbered, "Choisir les prochaines priorités"),
            NoteBlock::text(Kind::Numbered, "Partager une première version"),
            NoteBlock::text(Kind::Todo, "Poser les fondations de Folio"),
            NoteBlock::text(Kind::Todo, "Réorganiser cette liste à la souris"),
            NoteBlock::text(
                Kind::Bullet,
                "Le contenu, les styles et les cases cochées suivent le bloc.",
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
                name: "Tous les projets".into(),
                view: View::Table,
                sort: Sort::Title,
                filter: TrackingFilter::default(),
            },
            SavedView {
                layout: None,
                id: "demo-view-active".into(),
                name: "En cours".into(),
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
                name: "À débloquer".into(),
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
        let mut hours = Column::new("Charge (h)", ColumnKind::Number);
        hours.id = "demo-hours".into();
        let mut approved = Column::new("Livrable validé", ColumnKind::Checkbox);
        approved.id = "demo-approved".into();
        let mut owner = Column::new("Référent", ColumnKind::Text);
        owner.id = "demo-owner".into();
        if let Some(base) = self.pages.iter_mut().find(|p| p.id == "demo-tracker")
            && let Some(collection) = &mut base.collection
        {
            collection.columns = vec![hours, approved, owner];
        }
        let mut late = Page::new(Some("demo-tracker".into()));
        late.id = "demo-record-5".into();
        late.title = "Valider le cadrage".into();
        late.properties.priority = Priority::High;
        late.properties.due = Some("2026-11-08".into());
        late.properties.tags = vec!["Produit".into()];
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
                    .insert("demo-owner".into(), CellValue::Text("Moi".into()));
            }
        }
        let mut points = Column::new("Points estimés", ColumnKind::Number);
        points.id = "demo-points".into();
        let mut launch = Page::new(None);
        launch.id = "demo-launch".into();
        launch.title = "Lancement de Folio".into();
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
            ("Préparer la présentation", Status::Todo, "2026-11-10", 3),
            ("Tester les parcours", Status::InProgress, "2026-11-13", 5),
            (
                "Partager la première version",
                Status::Done,
                "2026-11-15",
                8,
            ),
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
        dashboard.title = "Le cap, en un regard".into();
        dashboard.emoji = "📊".into();
        dashboard.wide = true;
        dashboard.dashboard = Some(DashboardConfig::default());
        dashboard.blocks[0].id = "demo-dashboard-0".into();
        self.pages.push(dashboard);
    }
}
