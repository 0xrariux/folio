use super::{
    Palette,
    collections::{status_button, status_color, title_button},
    label,
};
use crate::state::{Panel, State};
use folio::icons::controls as g;
use folio::{
    collection::{Priority, Status},
    dashboard::{Deadline, Summary, calculated_summaries, due_soon, overdue},
    fields::{ColumnKind, format_number},
    model::Page,
};
use rune::{Color, El, TextStyle, col, el, row, text};

#[derive(Clone, Copy)]
enum Metric {
    All,
    Done,
    InProgress,
    Overdue,
    Soon,
}
fn metric(
    key: &str,
    title: &str,
    value: impl Into<String>,
    detail: &str,
    selected: bool,
    metric: Option<Metric>,
    p: Palette,
) -> El {
    let mut card = col()
        .key(key)
        .grow(1.0)
        .min_w(0.0)
        .min_h(96.0)
        .padding(14.0)
        .gap(8.0)
        .radius(8.0)
        .border(1.0, if selected { p.accent } else { p.line })
        .fill(if selected { p.tint } else { p.bg })
        .child(label(title, 11.0, p.muted))
        .child(text(value).font(TextStyle::new(29.0, 600.0)).color(p.ink))
        .child(label(detail, 10.0, p.faint));
    if let Some(metric) = metric {
        card = card
            .role("button")
            .label(title)
            .focusable(true)
            .hover(p.tint)
            .on_press(move |s: &mut State| {
                let filter = s.tracking_filter();
                match metric {
                    Metric::All => s.clear_tracking(),
                    Metric::Done => s.tracking_status(if filter.status == Some(Status::Done) {
                        None
                    } else {
                        Some(Status::Done)
                    }),
                    Metric::InProgress => {
                        s.tracking_status(if filter.status == Some(Status::InProgress) {
                            None
                        } else {
                            Some(Status::InProgress)
                        })
                    }
                    Metric::Overdue => {
                        s.tracking_deadline(if filter.deadline == Deadline::Overdue {
                            Deadline::All
                        } else {
                            Deadline::Overdue
                        })
                    }
                    Metric::Soon => s.tracking_deadline(if filter.deadline == Deadline::NextWeek {
                        Deadline::All
                    } else {
                        Deadline::NextWeek
                    }),
                }
            });
    }
    card
}
fn bar(count: usize, total: usize, color: Color, p: Palette) -> El {
    row()
        .h(7.0)
        .grow(1.0)
        .min_w(0.0)
        .radius(4.0)
        .fill(p.line)
        .child(el().h_full().grow(count as f32).radius(4.0).fill(color))
        .child(
            el().h_full()
                .grow(total.saturating_sub(count).max(usize::from(total == 0)) as f32),
        )
}
fn chart_box(title: &str, p: Palette) -> El {
    col()
        .grow(1.0)
        .min_w(0.0)
        .padding(16.0)
        .gap(11.0)
        .radius(8.0)
        .border(1.0, p.line)
        .child(label(title, 13.0, p.ink))
}
fn charts(s: &State, items: &[&Page], p: Palette) -> El {
    let mut statuses = chart_box("By status", p);
    let mut priorities = chart_box("By priority", p);
    let filter = s.tracking_filter();
    for status in Status::ALL {
        let count = items
            .iter()
            .filter(|p| p.properties.status == status)
            .count();
        statuses = statuses.child(
            row()
                .key(format!("chart-status-{status:?}"))
                .role("button")
                .label(format!("{} : {count}", status.name()))
                .focusable(true)
                .h(30.0)
                .gap(9.0)
                .center_y()
                .radius(4.0)
                .hover(p.tint)
                .on_press(move |s: &mut State| {
                    s.tracking_status(if filter.status == Some(status) {
                        None
                    } else {
                        Some(status)
                    })
                })
                .child(label(status.name(), 11.0, p.muted).w(72.0))
                .child(bar(count, items.len(), status_color(status, p), p))
                .child(label(count.to_string(), 11.0, p.ink).w(22.0)),
        );
    }
    for priority in Priority::ALL {
        let count = items
            .iter()
            .filter(|p| p.properties.priority == priority)
            .count();
        let selected = filter.priority;
        priorities = priorities.child(
            row()
                .key(format!("chart-priority-{priority:?}"))
                .role("button")
                .label(format!("{} : {count}", priority.name()))
                .focusable(true)
                .h(30.0)
                .gap(9.0)
                .center_y()
                .radius(4.0)
                .hover(p.tint)
                .on_press(move |s: &mut State| {
                    s.tracking_priority(if selected == Some(priority) {
                        None
                    } else {
                        Some(priority)
                    })
                })
                .child(label(priority.name(), 11.0, p.muted).w(72.0))
                .child(bar(count, items.len(), p.accent, p))
                .child(label(count.to_string(), 11.0, p.ink).w(22.0)),
        );
    }
    row().gap(12.0).child(statuses).child(priorities)
}
fn deadlines(s: &State, items: &[&Page], p: Palette) -> El {
    let mut due: Vec<_> = items
        .iter()
        .copied()
        .filter(|page| overdue(page, &s.today) || due_soon(page, &s.today))
        .collect();
    due.sort_by(|a, b| {
        a.properties
            .due
            .cmp(&b.properties.due)
            .then_with(|| a.title.cmp(&b.title))
    });
    let mut box_ui = col()
        .key("deadline-watch")
        .w_full()
        .padding(15.0)
        .gap(8.0)
        .radius(8.0)
        .border(1.0, p.line)
        .child(label("Upcoming and overdue dates", 13.0, p.ink));
    if due.is_empty() {
        box_ui = box_ui.child(label("No open due dates in this scope.", 12.0, p.faint));
    }
    for page in due.into_iter().take(6) {
        let late = overdue(page, &s.today);
        let day = page.properties.due.as_deref().unwrap_or("");
        box_ui = box_ui.child(
            row()
                .h(33.0)
                .gap(9.0)
                .center_y()
                .child(
                    rune::icon(if late { &g::ALERT } else { &g::CLOCK }, 15.0)
                        .color(p.accent)
                        .a11y_hidden(),
                )
                .child(title_button(page, p).min_w(0.0))
                .child(label(
                    if late {
                        format!("Overdue · {day}")
                    } else {
                        day.into()
                    },
                    11.0,
                    if late { p.accent } else { p.muted },
                )),
        );
    }
    box_ui
}
fn details(s: &State, items: &[&Page], p: Palette) -> El {
    let mut rows = col()
        .key("dashboard-details-content")
        .w_full()
        .min_w(700.0)
        .child(
            super::table::header_row("dashboard-details-header", 36.0, p)
                .child(
                    super::table::column_heading("Record", p)
                        .grow(1.0)
                        .min_w(170.0),
                )
                .child(super::table::column_heading("Database", p).w(150.0))
                .child(super::table::column_heading("Status", p).w(112.0))
                .child(super::table::column_heading("Due date", p).w(100.0)),
        );
    for (index, page) in items.iter().enumerate() {
        if index > 0 {
            rows = rows.child(
                el().key(format!("dashboard-separator-{}", page.id))
                    .w_full()
                    .h(1.0)
                    .fill(p.line),
            );
        }
        let base = page
            .parent_id
            .as_ref()
            .and_then(|id| s.pages.iter().find(|p| p.id == *id));
        rows = rows.child(
            row()
                .key(format!("dashboard-row-{}", page.id))
                .role("row")
                .w_full()
                .h(42.0)
                .px(10.0)
                .gap(8.0)
                .center_y()
                .child(title_button(page, p).min_w(170.0))
                .child(
                    label(base.map_or("—", |p| p.display_title()), 11.0, p.faint)
                        .w(150.0)
                        .truncate(),
                )
                .child(el().w(112.0).child(status_button(page, p)))
                .child(
                    label(page.properties.due.as_deref().unwrap_or("—"), 11.0, p.muted).w(100.0),
                ),
        );
    }
    if items.is_empty() {
        rows = rows.child(label("No records match these filters.", 12.0, p.faint).padding(16.0));
    }
    let height = 36.0
        + if items.is_empty() {
            54.0
        } else {
            items.len() as f32 * 42.0 + items.len().saturating_sub(1) as f32
        };
    col()
        .gap(8.0)
        .w_full()
        .child(label("The records behind the numbers", 13.0, p.ink))
        .child(super::table::frame(
            "dashboard-details-frame",
            "dashboard-details",
            "Records behind the numbers",
            rows,
            height,
            252.0,
            p,
        ))
}
fn sources(s: &State, items: &[&Page], p: Palette) -> El {
    let mut list = col()
        .w_full()
        .padding(15.0)
        .gap(9.0)
        .radius(8.0)
        .border(1.0, p.line)
        .child(label("Progress by database", 13.0, p.ink));
    for source in s.tracking_sources() {
        let records: Vec<_> = items
            .iter()
            .copied()
            .filter(|p| p.parent_id.as_deref() == Some(&source.id))
            .collect();
        let stats = Summary::from_pages(&records, &s.today);
        let id = source.id.clone();
        list = list.child(
            row()
                .h(33.0)
                .gap(9.0)
                .center_y()
                .child(
                    super::button(
                        format!("dashboard-base-{id}"),
                        source.display_title(),
                        p,
                        move |s, tx| {
                            s.select(&id);
                            tx.scroll_to_y("document-scroll", 0.0);
                        },
                    )
                    .w(210.0)
                    .justify_start()
                    .px(0.0)
                    .child(
                        label(source.display_title(), 12.0, p.muted)
                            .min_w(0.0)
                            .truncate(),
                    ),
                )
                .child(bar(stats.done, stats.total, p.accent, p))
                .child(
                    label(
                        format!("{} % · {}/{}", stats.progress(), stats.done, stats.total),
                        11.0,
                        p.muted,
                    )
                    .w(110.0),
                ),
        );
    }
    list
}

pub(super) fn view(s: &State, p: Palette) -> El {
    let items = s.tracking_items();
    let summary = Summary::from_pages(&items, &s.today);
    let filter = s.tracking_filter();
    let mut view = col()
        .key("tracking-dashboard")
        .w_full()
        .gap(13.0)
        .instant_changes()
        .child(
            row()
                .center_y()
                .child(label(format!("As of {}", s.today), 10.0, p.faint))
                .child(el().grow(1.0))
                .child(label(
                    if filter.active() || !s.collection_search.text.is_empty() {
                        "Filtered metrics"
                    } else {
                        "All active records"
                    },
                    10.0,
                    p.accent,
                )),
        )
        .child(
            row()
                .gap(10.0)
                .child(metric(
                    "metric-total",
                    "Records",
                    summary.total.to_string(),
                    "in this scope",
                    false,
                    Some(Metric::All),
                    p,
                ))
                .child(metric(
                    "metric-done",
                    "Completed",
                    summary.done.to_string(),
                    "work completed",
                    filter.status == Some(Status::Done),
                    Some(Metric::Done),
                    p,
                ))
                .child(metric(
                    "metric-progress",
                    "Progress",
                    format!("{} %", summary.progress()),
                    "completed records / total",
                    false,
                    None,
                    p,
                )),
        )
        .child(
            row()
                .gap(10.0)
                .child(metric(
                    "metric-active",
                    "In progress",
                    summary.in_progress.to_string(),
                    "work in progress",
                    filter.status == Some(Status::InProgress),
                    Some(Metric::InProgress),
                    p,
                ))
                .child(metric(
                    "metric-late",
                    "Overdue",
                    summary.overdue.to_string(),
                    "open records past their due date",
                    filter.deadline == Deadline::Overdue,
                    Some(Metric::Overdue),
                    p,
                ))
                .child(metric(
                    "metric-soon",
                    "Next 7 days",
                    summary.soon.to_string(),
                    "upcoming due dates for open records",
                    filter.deadline == Deadline::NextWeek,
                    Some(Metric::Soon),
                    p,
                )),
        )
        .child(charts(s, &items, p));
    let aggregate = calculated_summaries(&s.tracking_sources(), &items, &s.pages);
    if !aggregate.is_empty() {
        let mut fields = col()
            .gap(10.0)
            .w_full()
            .child(label("Column totals", 13.0, p.ink));
        for chunk in aggregate.chunks(2) {
            let mut cards = row().gap(10.0);
            for field in chunk {
                let (value, detail) = if field.kind == ColumnKind::Checkbox {
                    (
                        format!("{} / {}", field.checked, field.records),
                        "checked records".into(),
                    )
                } else {
                    (
                        field
                            .sum
                            .map(format_number)
                            .unwrap_or_else(|| "Out of range".into()),
                        format!("sum · {} values", field.filled),
                    )
                };
                cards = cards.child(
                    col()
                        .key(format!(
                            "aggregate-{}-{}",
                            field.collection_id, field.column_id
                        ))
                        .grow(1.0)
                        .min_w(0.0)
                        .padding(15.0)
                        .gap(7.0)
                        .radius(7.0)
                        .fill(p.tint)
                        .child(label(&field.label, 12.0, p.ink))
                        .child(
                            text(value)
                                .font(TextStyle::new(25.0, 600.0))
                                .color(p.accent),
                        )
                        .child(
                            label(format!("{} · {detail}", field.base), 10.0, p.muted)
                                .w_full()
                                .truncate(),
                        ),
                );
            }
            if chunk.len() == 1 {
                cards = cards.child(el().grow(1.0));
            }
            fields = fields.child(cards);
        }
        view = view.child(fields);
    }
    if s.current().is_some_and(|p| p.dashboard.is_some()) {
        view = view.child(sources(s, &items, p));
    }
    view.child(deadlines(s, &items, p))
        .child(details(s, &items, p))
}

pub(super) fn global(s: &State, p: Palette) -> El {
    let mut body = col().w_full().gap(12.0);
    body = body.child(
        row()
            .gap(8.0)
            .center_y()
            .child(
                super::icons::action(
                    "dashboard-sources",
                    &format!("{} tracked databases", s.tracking_sources().len()),
                    &folio::icons::DATABASE,
                    p,
                    |s, _| s.panel = Panel::DashboardSources,
                )
                .fill(p.tint),
            )
            .child(el().grow(1.0))
            .child(super::icons::action(
                "collection-filter",
                "Filters",
                &g::FILTER,
                p,
                |s, _| s.panel = Panel::CollectionFilter,
            ))
            .child(super::icons::action(
                "reset-tracking",
                "Reset",
                &g::CLOSE,
                p,
                |s, _| s.clear_tracking(),
            )),
    );
    body.child(row().w_full().child(super::collections::search(s, p)))
        .child(view(s, p))
}
