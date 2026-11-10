use crate::state::State;
use folio::{
    collection::{Collection, View},
    document::{Kind, NoteBlock},
    fields::{Aggregate, CellValue, Column, ColumnKind},
    model::Page,
    workspace::EmbeddedView,
};
impl State {
    pub fn demo_workspace(&mut self) {
        if !self.demo {
            return;
        }
        let mut relation = Column::new("Tâches du projet", ColumnKind::Relation);
        relation.id = "demo-project-tasks".into();
        relation.config.target_base = Some("demo-tracker".into());
        relation.config.reverse_of = Some("demo-task-project".into());
        let mut progress = Column::new("Progression (%)", ColumnKind::Rollup);
        progress.id = "demo-project-progress".into();
        progress.config.relation = Some(relation.id.clone());
        progress.config.aggregate = Aggregate::Progress;
        let mut hours = Column::new("Charge totale (h)", ColumnKind::Rollup);
        hours.id = "demo-project-hours".into();
        hours.config.relation = Some(relation.id.clone());
        hours.config.aggregate = Aggregate::Sum;
        hours.config.property = Some("demo-hours".into());
        let mut budget = Column::new("Budget indicatif", ColumnKind::Formula);
        budget.id = "demo-project-budget".into();
        budget.config.expression = "prop(\"Charge totale (h)\") * 80".into();
        let mut projects = Page::new(None);
        projects.id = "demo-projects-base".into();
        projects.title = "Les projets de Folio".into();
        projects.emoji = "🗂️".into();
        projects.wide = true;
        projects.collection = Some(Collection {
            columns: vec![relation, progress, hours, budget],
            ..Default::default()
        });
        self.pages.push(projects);
        let mut project = Page::new(Some("demo-projects-base".into()));
        project.id = "demo-workspace-project".into();
        project.title = "Construire Folio".into();
        project.emoji = "🚀".into();
        project.wide = true;
        project.properties.custom.insert(
            "demo-project-tasks".into(),
            CellValue::Relation(vec![
                "demo-record-0".into(),
                "demo-record-1".into(),
                "demo-record-3".into(),
            ]),
        );
        let mut owner = Column::new("Projet", ColumnKind::Relation);
        owner.id = "demo-task-project".into();
        owner.config.target_base = Some("demo-projects-base".into());
        if let Some(base) = self
            .pages
            .iter_mut()
            .find(|p| p.id == "demo-tracker")
            .and_then(|p| p.collection.as_mut())
        {
            base.columns.push(owner);
            base.view = View::Table;
        }
        for page in self.pages.iter_mut().filter(|p| {
            ["demo-record-0", "demo-record-1", "demo-record-3"].contains(&p.id.as_str())
        }) {
            page.properties.custom.insert(
                "demo-task-project".into(),
                CellValue::Relation(vec![project.id.clone()]),
            );
        }
        let mut embed = NoteBlock::new(Kind::DatabaseView);
        embed.embedded = Some(EmbeddedView {
            base_id: "demo-tracker".into(),
            view_id: None,
            relation_filter: Some("demo-task-project".into()),
        });
        project.blocks = vec![
            NoteBlock::new(Kind::TableOfContents),
            NoteBlock::text(Kind::Heading2, "Le projet, au même endroit"),
            NoteBlock::text(
                Kind::Text,
                "Les notes, les tâches et leurs indicateurs partagent les mêmes données. Les propriétés ci-dessus se recalculent depuis les fiches liées.",
            ),
            embed,
            NoteBlock::text(Kind::Heading2, "Décisions et prochaines étapes"),
            NoteBlock::text(Kind::Toggle, "Préparer la prochaine livraison"),
            NoteBlock::text(Kind::Todo, "Valider le parcours d’écriture"),
            NoteBlock::text(Kind::Todo, "Vérifier les sauvegardes du carnet"),
        ];
        project.blocks[6].indent = 1;
        project.blocks[7].indent = 1;
        project.blocks[6].checked = true;
        for (i, b) in project.blocks.iter_mut().enumerate() {
            b.id = format!("demo-workspace-{i}");
        }
        project.sync_document();
        self.pages.push(project);
        self.select("demo-workspace-project");
    }
    pub fn demo_calendar(&mut self) {
        if self.demo {
            self.select("demo-tracker");
            self.collection_view(View::Calendar);
        }
    }
}
