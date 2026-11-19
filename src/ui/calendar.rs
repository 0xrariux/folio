use super::{Palette, action, icons, label, panels::surface};
use crate::{
    calendar::{self, DatePicker},
    entity_drag,
    state::{Panel, State},
};
use chrono::{Datelike, Months};
use rune::{El, TextStyle, col, el, row, text_input};

pub(super) fn view(s: &State, p: Palette) -> El {
    let month = s.calendar_month();
    let records = s.collection_items();
    let mut body = col().key("collection-calendar").w_full().gap(8.0).child(
        row()
            .gap(8.0)
            .center_y()
            .child(icons::control(
                "calendar-prev",
                "Previous month",
                &folio::icons::controls::CHEVRON_LEFT,
                p,
                |s, _| s.step_calendar(false),
            ))
            .child(label(month.format("%m / %Y").to_string(), 16.0, p.ink))
            .child(icons::control(
                "calendar-next",
                "Next month",
                &folio::icons::controls::CHEVRON_RIGHT,
                p,
                |s, _| s.step_calendar(true),
            ))
            .child(action("calendar-today", "Today", p, |s, _| {
                s.workspace.calendar_month = s.today[..7].into()
            })),
    );
    let mut weekdays = row().w_full().gap(4.0);
    for day in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
        weekdays = weekdays.child(label(day, 10.0, p.faint).grow(1.0).min_w(0.0));
    }
    body = body.child(weekdays);
    for week in calendar::days(month).chunks(7) {
        let mut line = row().w_full().gap(4.0);
        for day in week {
            let mut cell = col()
                .grow(1.0)
                .min_w(0.0)
                .h(106.0)
                .padding(6.0)
                .gap(3.0)
                .radius(5.0)
                .border(1.0, p.line);
            if let Some(day) = day {
                let date = day.format("%Y-%m-%d").to_string();
                let day_date = date.clone();
                cell = cell
                    .fill(
                        if date == s.today
                            || super::entity_target(s, &format!("calendar-day-{date}"))
                        {
                            p.tint
                        } else {
                            p.bg
                        },
                    )
                    .child(
                        action(
                            format!("calendar-date-{date}"),
                            &day.day().to_string(),
                            p,
                            move |s, _| s.panel = Panel::CalendarDay(day_date.clone()),
                        )
                        .h(18.0)
                        .px(0.0)
                        .justify_start(),
                    );
                let matching: Vec<_> = records
                    .iter()
                    .filter(|p| p.properties.due.as_deref() == Some(&date))
                    .collect();
                for record in matching.iter().take(2) {
                    let id = record.id.clone();
                    let drag_id = id.clone();
                    cell = cell.child(
                        super::button(
                            format!("calendar-record-{id}"),
                            record.display_title(),
                            p,
                            move |s, _| {
                                if !s.entity_click(&id) {
                                    s.select(&id);
                                }
                            },
                        )
                        .w_full()
                        .min_w(0.0)
                        .px(3.0)
                        .h(24.0)
                        .justify_start()
                        .fill(p.sidebar)
                        .on_drag(move |s: &mut State, d, tx| {
                            s.drag_entity(&drag_id, entity_drag::Kind::Calendar, d, tx)
                        })
                        .child(
                            label(record.display_title(), 10.0, p.ink)
                                .min_w(0.0)
                                .truncate(),
                        ),
                    );
                }
                if matching.len() > 2 {
                    let day_date = date.clone();
                    cell = cell.child(
                        action(
                            format!("calendar-more-{date}"),
                            &format!("+{} records", matching.len() - 2),
                            p,
                            move |s, _| s.panel = Panel::CalendarDay(day_date.clone()),
                        )
                        .h(18.0)
                        .px(0.0),
                    );
                }
                cell = cell.child(
                    el().key(format!("calendar-day-{date}"))
                        .absolute(0.0, 0.0)
                        .w_full()
                        .h_full(),
                );
            }
            line = line.child(cell);
        }
        body = body.child(line);
    }
    let undated: Vec<_> = records
        .iter()
        .filter(|p| p.properties.due.is_none())
        .collect();
    if !undated.is_empty() {
        body = body.child(label("NO DUE DATE — drag to a day", 10.0, p.faint));
        for record in undated.iter().take(20) {
            let id = record.id.clone();
            let drag_id = id.clone();
            body = body.child(
                action(
                    format!("calendar-record-{id}"),
                    record.display_title(),
                    p,
                    move |s, _| {
                        if !s.entity_click(&id) {
                            s.select(&id);
                        }
                    },
                )
                .justify_start()
                .on_drag(move |s: &mut State, d, tx| {
                    s.drag_entity(&drag_id, entity_drag::Kind::Calendar, d, tx)
                }),
            );
        }
    }
    body
}
pub(super) fn picker(s: &State, form: &DatePicker, p: Palette) -> El {
    let mut body = col().gap(7.0).child(
        row()
            .gap(6.0)
            .center_y()
            .child(icons::control(
                "date-prev",
                "Previous month",
                &folio::icons::controls::CHEVRON_LEFT,
                p,
                |s, _| {
                    if let Panel::Date(f) = &mut s.panel
                        && let Some(m) = f.month.checked_sub_months(Months::new(1))
                    {
                        f.month = m;
                    }
                },
            ))
            .child(label(form.month.format("%m / %Y").to_string(), 13.0, p.ink).grow(1.0))
            .child(icons::control(
                "date-next",
                "Next month",
                &folio::icons::controls::CHEVRON_RIGHT,
                p,
                |s, _| {
                    if let Panel::Date(f) = &mut s.panel
                        && let Some(m) = f.month.checked_add_months(Months::new(1))
                    {
                        f.month = m;
                    }
                },
            )),
    );
    let mut names = row().gap(2.0);
    for name in ["M", "T", "W", "T", "F", "S", "S"] {
        names = names.child(label(name, 10.0, p.faint).w(33.0));
    }
    body = body.child(names);
    for week in calendar::days(form.month).chunks(7) {
        let mut line = row().gap(2.0);
        for day in week {
            line = line.child(if let Some(day) = day {
                let date = day.format("%Y-%m-%d").to_string();
                let key = format!("pick-date-{date}");
                action(key, &day.day().to_string(), p, move |s, _| {
                    s.pick_date(Some(&date))
                })
                .w(33.0)
                .px(0.0)
            } else {
                el().w(33.0)
            });
        }
        body = body.child(line);
    }
    body = body.child(
        text_input(&form.value, TextStyle::new(12.0, 400.0))
            .key("date-value")
            .label("Date YYYY-MM-DD")
            .single_line()
            .w_full()
            .h(32.0)
            .px(7.0)
            .border(1.0, p.line)
            .radius(4.0)
            .color(p.ink)
            .caret(p.accent, p.tint)
            .on_edit(|s: &mut State| {
                let Panel::Date(f) = &mut s.panel else {
                    unreachable!()
                };
                &mut f.value
            })
            .on_submit(|s: &mut State, _| {
                if let Panel::Date(f) = &mut s.panel {
                    let date = f.value.text.trim().to_string();
                    if date.is_empty() {
                        s.pick_date(None);
                    } else if folio::collection::valid_date(&date) {
                        s.pick_date(Some(&date));
                    } else {
                        f.error = Some("Invalid date: YYYY-MM-DD".into());
                    }
                }
            }),
    );
    if let Some(error) = &form.error {
        body = body.child(label(error, 11.0, p.ink));
    }
    body = body.child(
        row()
            .gap(5.0)
            .child(action("date-today", "Today", p, |s, _| {
                let today = s.today.clone();
                s.pick_date(Some(&today));
            }))
            .child(action("date-clear", "Clear", p, |s, _| s.pick_date(None))),
    );
    let _ = s;
    surface("date-picker", &form.anchor, 275.0, p)
        .padding(12.0)
        .child(body)
}

pub(super) fn day(s: &State, date: &str, p: Palette) -> El {
    let mut rows = col().gap(5.0);
    for record in s
        .collection_items()
        .into_iter()
        .filter(|p| p.properties.due.as_deref() == Some(date))
    {
        rows = rows.child(super::collections::title_button(record, p));
    }
    let date = date.to_string();
    let new_date = date.clone();
    let title = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_or(date, |d| d.format("%Y-%m-%d").to_string());
    super::workspace::dialog(
        &title,
        p,
        col()
            .gap(10.0)
            .child(
                col()
                    .h(280.0)
                    .scroll(rune::ScrollSpec {
                        stick_bottom: false,
                    })
                    .child(rows),
            )
            .child(icons::action(
                "calendar-new-on-day",
                "New record on this date",
                &folio::icons::controls::PLUS,
                p,
                move |s, tx| {
                    s.create_record(None, tx);
                    if let Some(id) = s.selected.clone() {
                        s.set_due(&id, None, Some(&new_date));
                    }
                },
            )),
    )
}
