use folio::{
    collection::{Collection, Status, View},
    dashboard::{DashboardConfig, Deadline, Scope, Summary, TrackingFilter, field_summaries},
    fields::{CellValue, Column, ColumnKind, parse_cell},
    model::Page,
    storage::Store,
};

fn record(parent: &str, status: Status, due: Option<&str>) -> Page {
    let mut page = Page::new(Some(parent.into()));
    page.properties.status = status;
    page.properties.due = due.map(str::to_string);
    page
}

#[test]
fn deadlines_follow_calendar_boundaries_and_exclude_completed_work() {
    let yesterday = record("base", Status::Todo, Some("2028-02-28"));
    let today = record("base", Status::InProgress, Some("2028-02-29"));
    let week = record("base", Status::Todo, Some("2028-03-07"));
    let later = record("base", Status::Todo, Some("2028-03-08"));
    let done = record("base", Status::Done, Some("2028-02-20"));
    let no_date = record("base", Status::Todo, None);
    let invalid = record("base", Status::Todo, Some("2028-02-30"));
    let summary = Summary::from_pages(
        &[&yesterday, &today, &week, &later, &done, &no_date, &invalid],
        "2028-02-29",
    );
    assert_eq!(
        (summary.total, summary.done, summary.overdue, summary.soon),
        (7, 1, 1, 2)
    );
    assert_eq!(Summary::from_pages(&[], "2028-02-29").progress(), 0);
    let filter = TrackingFilter {
        deadline: Deadline::Overdue,
        ..Default::default()
    };
    assert!(filter.matches(&yesterday, "2028-02-29"));
    assert!(!filter.matches(&done, "2028-02-29"));
    assert!(!filter.matches(&invalid, "2028-02-29"));
    assert_eq!(Summary::from_pages(&[&today], "2028-03-01").overdue, 1);
}

#[test]
fn numeric_totals_stay_separate_for_columns_with_identical_names_in_different_bases() {
    let mut a = Page::new(None);
    let mut b = Page::new(None);
    let first = Column::new("Charge", ColumnKind::Number);
    let second = Column::new("Charge", ColumnKind::Number);
    let checkbox = Column::new("Validé", ColumnKind::Checkbox);
    a.collection = Some(Collection {
        columns: vec![first.clone(), checkbox.clone()],
        ..Default::default()
    });
    b.collection = Some(Collection {
        columns: vec![second.clone()],
        ..Default::default()
    });
    let mut r1 = record(&a.id, Status::Todo, None);
    let mut r2 = record(&a.id, Status::Done, None);
    let mut r3 = record(&b.id, Status::Todo, None);
    r1.properties
        .custom
        .insert(first.id.clone(), CellValue::Number("12.5".into()));
    r2.properties
        .custom
        .insert(first.id.clone(), CellValue::Number("-2".into()));
    r3.properties
        .custom
        .insert(second.id.clone(), CellValue::Number("7".into()));
    r2.properties
        .custom
        .insert(checkbox.id.clone(), CellValue::Checkbox(true));
    let fields = field_summaries(&[&a, &b], &[&r1, &r2, &r3]);
    assert_eq!(
        fields.iter().find(|f| f.column_id == first.id).unwrap().sum,
        Some(10.5)
    );
    assert_eq!(
        fields
            .iter()
            .find(|f| f.column_id == second.id)
            .unwrap()
            .sum,
        Some(7.0)
    );
    let checks = fields.iter().find(|f| f.column_id == checkbox.id).unwrap();
    assert_eq!((checks.checked, checks.records), (1, 2));
    a.collection.as_mut().unwrap().columns[0].visible = false;
    assert_eq!(field_summaries(&[&a], &[&r1, &r2]).len(), 1);
}

#[test]
fn invalid_values_do_not_enter_totals_and_overflow_is_explicit() {
    assert!(folio::fields::format_number(1e308).len() < 24);
    assert_eq!(
        parse_cell(ColumnKind::Number, "12,5").unwrap(),
        Some(CellValue::Number("12.5".into()))
    );
    assert_eq!(parse_cell(ColumnKind::Number, " ").unwrap(), None);
    for value in ["NaN", "inf", "1e999", "abc"] {
        assert!(parse_cell(ColumnKind::Number, value).is_err());
    }
    assert!(parse_cell(ColumnKind::Date, "2026-02-29").is_err());
    assert!(parse_cell(ColumnKind::Date, "2028-02-29").is_ok());
    let mut base = Page::new(None);
    let column = Column::new("Grand nombre", ColumnKind::Number);
    base.collection = Some(Collection {
        columns: vec![column.clone()],
        ..Default::default()
    });
    let mut a = record(&base.id, Status::Todo, None);
    let mut b = record(&base.id, Status::Todo, None);
    a.properties
        .custom
        .insert(column.id.clone(), CellValue::Number("1e308".into()));
    b.properties
        .custom
        .insert(column.id.clone(), CellValue::Number("1e308".into()));
    assert_eq!(field_summaries(&[&base], &[&a, &b])[0].sum, None);
}

#[test]
fn dashboard_configuration_schema_and_values_survive_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("dashboard.db");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let store = Store::open(&path).await.unwrap();
            let column = Column::new("Charge", ColumnKind::Number);
            let mut base = Page::new(None);
            base.collection = Some(Collection {
                view: View::Dashboard,
                columns: vec![column.clone()],
                deadline: Deadline::NextWeek,
                ..Default::default()
            });
            let mut item = record(&base.id, Status::Todo, Some("2026-11-14"));
            item.properties
                .custom
                .insert(column.id.clone(), CellValue::Number("42.5".into()));
            let mut dashboard = Page::new(None);
            dashboard.dashboard = Some(DashboardConfig {
                scope: Scope::Selected(vec![base.id.clone()]),
                filter: TrackingFilter {
                    status: Some(Status::Todo),
                    ..Default::default()
                },
            });
            for page in [&base, &item, &dashboard] {
                store.save(page).await.unwrap();
            }
            drop(store);
            let store = Store::open(&path).await.unwrap();
            let loaded = store.load().await.unwrap();
            for expected in [base, item, dashboard] {
                assert_eq!(loaded.iter().find(|p| p.id == expected.id), Some(&expected));
            }
        });
}

#[test]
fn version_two_documents_load_with_empty_custom_schema_and_no_dashboard() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("old.db");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let store = Store::open(&path).await.unwrap();
            let mut page = Page::new(None);
            page.collection = Some(Collection::default());
            store.save(&page).await.unwrap();
            let database = turso::Builder::new_local(path.to_str().unwrap())
                .build()
                .await
                .unwrap();
            let connection = database.connect().unwrap();
            let mut json = serde_json::to_value(page.content()).unwrap();
            json["version"] = 2.into();
            json.as_object_mut().unwrap().remove("dashboard");
            json["properties"].as_object_mut().unwrap().remove("custom");
            for key in ["columns", "deadline", "priority", "hidden"] {
                json["collection"].as_object_mut().unwrap().remove(key);
            }
            connection
                .execute(
                    "UPDATE page_content SET content_json=?",
                    (serde_json::to_string(&json).unwrap(),),
                )
                .await
                .unwrap();
            drop(connection);
            drop(database);
            drop(store);
            let store = Store::open(&path).await.unwrap();
            assert_eq!(store.load().await.unwrap(), vec![page]);
        });
}
