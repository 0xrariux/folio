use folio::{
    calculations,
    collection::{Collection, Status},
    fields::{Aggregate, CellValue, Column, ColumnKind},
    model::Page,
};
fn data() -> (Vec<Page>, String, String, String) {
    let mut projects = Page::new(None);
    projects.id = "projects".into();
    let mut tasks = Page::new(None);
    tasks.id = "tasks".into();
    let hours = Column::new("Charge", ColumnKind::Number);
    let hours_id = hours.id.clone();
    tasks.collection = Some(Collection {
        columns: vec![hours],
        ..Default::default()
    });
    let mut relation = Column::new("Tâches", ColumnKind::Relation);
    relation.config.target_base = Some(tasks.id.clone());
    let relation_id = relation.id.clone();
    let mut progress = Column::new("Progression", ColumnKind::Rollup);
    progress.config.relation = Some(relation_id.clone());
    progress.config.aggregate = Aggregate::Progress;
    let progress_id = progress.id.clone();
    let mut sum = Column::new("Charge", ColumnKind::Rollup);
    sum.config.relation = Some(relation_id.clone());
    sum.config.aggregate = Aggregate::Sum;
    sum.config.property = Some(hours_id.clone());
    let mut formula = Column::new("Double", ColumnKind::Formula);
    formula.config.expression = "prop(\"Charge\") * 2 + status_done".into();
    let formula_id = formula.id.clone();
    projects.collection = Some(Collection {
        columns: vec![relation, progress, sum, formula],
        ..Default::default()
    });
    let mut project = Page::new(Some("projects".into()));
    project.id = "project".into();
    project.properties.custom.insert(
        relation_id,
        CellValue::Relation(vec![
            "a".into(),
            "b".into(),
            "a".into(),
            "missing".into(),
            "archived".into(),
        ]),
    );
    let mut a = Page::new(Some("tasks".into()));
    a.id = "a".into();
    a.properties.status = Status::Done;
    a.properties
        .custom
        .insert(hours_id.clone(), CellValue::Number("2.5".into()));
    let mut b = Page::new(Some("tasks".into()));
    b.id = "b".into();
    b.properties
        .custom
        .insert(hours_id.clone(), CellValue::Number("3.5".into()));
    let mut archived = Page::new(Some("tasks".into()));
    archived.id = "archived".into();
    archived.archived = true;
    archived
        .properties
        .custom
        .insert(hours_id, CellValue::Number("1000".into()));
    (
        vec![projects, tasks, project, a, b, archived],
        progress_id,
        formula_id,
        "project".into(),
    )
}
#[test]
fn relations_rollups_formulas_and_csv_recompute_from_current_records() {
    let (mut pages, progress, formula, project) = data();
    let column = |id: &str| {
        pages[0]
            .collection
            .as_ref()
            .unwrap()
            .columns
            .iter()
            .find(|c| c.id == id)
            .unwrap()
    };
    assert_eq!(
        calculations::value(&pages[2], column(&progress), &pages).unwrap(),
        50.0
    );
    assert_eq!(
        calculations::value(&pages[2], column(&formula), &pages).unwrap(),
        12.0
    );
    let csv = folio::export::csv_with_context("Projet", &[&pages[0]], &[&pages[2]], &pages);
    assert!(csv.contents.contains("\"6\",\"12\""));
    pages[4].properties.status = Status::Done;
    let column = pages[0]
        .collection
        .as_ref()
        .unwrap()
        .columns
        .iter()
        .find(|c| c.id == progress)
        .unwrap();
    assert_eq!(
        calculations::value(
            pages.iter().find(|p| p.id == project).unwrap(),
            column,
            &pages
        )
        .unwrap(),
        100.0
    );
}
#[test]
fn formulas_reject_cycles_invalid_syntax_and_non_finite_results() {
    let (mut pages, _, formula, _) = data();
    let column = pages[0]
        .collection
        .as_mut()
        .unwrap()
        .columns
        .iter_mut()
        .find(|c| c.id == formula)
        .unwrap();
    column.config.expression = "prop(\"Double\")".into();
    let column = pages[0]
        .collection
        .as_ref()
        .unwrap()
        .columns
        .iter()
        .find(|c| c.id == formula)
        .unwrap();
    assert!(
        calculations::value(&pages[2], column, &pages)
            .unwrap_err()
            .contains("circulaire")
    );
    for bad in ["1/0", "NaN", "1+", "((1)", "1e309", "std::process(1)"] {
        assert!(calculations::evaluate(bad).is_err(), "{bad}");
    }
    assert_eq!(
        calculations::evaluate("(2.5 + 3.5) * 2 - -1").unwrap(),
        13.0
    );
    assert!(calculations::evaluate(&format!("{}1{}", "(".repeat(100), ")".repeat(100))).is_err());
}
