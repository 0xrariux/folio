use super::{Palette, action, icons, label};
use crate::{command_palette::Mode, state::State};
use folio::{
    collection::{Sort, Status, View},
    document::{Kind, NoteBlock},
    fields::ColumnKind,
    model::Page,
};
use rune::{El, ScrollSpec, col, el, row};

fn items<'a>(s: &'a State, block: &NoteBlock) -> Vec<&'a Page> {
    let Some(embed) = &block.embedded else {
        return Vec::new();
    };
    let Some(base) = s
        .pages
        .iter()
        .find(|p| p.id == embed.base_id && !p.archived)
        .and_then(|p| p.collection.as_ref())
    else {
        return Vec::new();
    };
    let saved = embed
        .view_id
        .as_ref()
        .and_then(|id| base.views.iter().find(|v| v.id == *id));
    let filter = saved.map_or_else(|| base.filter(), |v| v.filter.clone());
    let mut records: Vec<_> = s
        .pages
        .iter()
        .filter(|p| {
            !p.archived
                && p.parent_id.as_deref() == Some(&embed.base_id)
                && filter.matches(p, &s.today)
                && embed.relation_filter.as_ref().is_none_or(|column| {
                    base.columns
                        .iter()
                        .find(|c| c.id == *column)
                        .is_some_and(|c| {
                            folio::calculations::related(&s.pages, p, c)
                                .iter()
                                .any(|p| Some(&p.id) == s.selected.as_ref())
                        })
                })
        })
        .collect();
    let sort = saved.map_or(base.sort, |v| v.sort);
    records.sort_by(|a, b| {
        match sort {
            Sort::Title => a.title.cmp(&b.title),
            Sort::Updated => b.updated_at.cmp(&a.updated_at),
            Sort::Due => a
                .properties
                .due
                .as_deref()
                .unwrap_or("9999")
                .cmp(b.properties.due.as_deref().unwrap_or("9999")),
            Sort::Priority => b.properties.priority.cmp(&a.properties.priority),
        }
        .then_with(|| a.id.cmp(&b.id))
    });
    records
}
pub(super) fn database(s: &State, block: &NoteBlock, p: Palette) -> El {
    let block_id = block.id.clone();
    let Some(embed) = &block.embedded else {
        return icons::action(
            format!("choose-embed-{block_id}"),
            "Choose an existing database",
            &folio::icons::DATABASE,
            p,
            move |s, tx| {
                s.open_palette(Mode::Embed {
                    block: block_id.clone(),
                    base: None,
                });
                tx.focus("palette-search", false);
            },
        );
    };
    let Some(source) = s
        .pages
        .iter()
        .find(|p| p.id == embed.base_id && !p.archived)
    else {
        return label("The linked database is no longer available.", 12.0, p.faint);
    };
    let Some(base) = &source.collection else {
        return label("This page is not a database.", 12.0, p.faint);
    };
    let records = items(s, block);
    let mut settings = base.clone();
    if let Some(view_id) = &embed.view_id {
        settings.select_view(view_id);
    }
    let view = settings.view;
    let source_id = source.id.clone();
    let config_id = block.id.clone();
    let new_id = block.id.clone();
    let mut box_ = col()
        .key(format!("embedded-{}", block.id))
        .grow(1.0)
        .min_w(0.0)
        .padding(10.0)
        .gap(8.0)
        .radius(7.0)
        .border(1.0, p.line)
        .child(
            row()
                .gap(4.0)
                .center_y()
                .child(
                    icons::action(
                        format!("embed-open-{}", block.id),
                        source.display_title(),
                        &folio::icons::DATABASE,
                        p,
                        move |s, _| s.select(&source_id),
                    )
                    .grow(1.0)
                    .min_w(0.0)
                    .justify_start(),
                )
                .child(action(
                    format!("embed-config-{}", block.id),
                    "View…",
                    p,
                    move |s, tx| {
                        s.open_palette(Mode::Embed {
                            block: config_id.clone(),
                            base: None,
                        });
                        tx.focus("palette-search", false);
                    },
                ))
                .child(icons::control(
                    format!("embed-new-{}", block.id),
                    "New record",
                    &folio::icons::controls::PLUS,
                    p,
                    move |s, tx| s.new_embedded_record(&new_id, tx),
                )),
        );
    let count = records.len();
    let done = records
        .iter()
        .filter(|p| p.properties.status == Status::Done)
        .count();
    box_ = box_.child(label(
        format!(
            "{count} records · {done} completed · {}% progress",
            (done * 100).checked_div(count).unwrap_or(0)
        ),
        11.0,
        p.muted,
    ));
    let relations: Vec<_> = base
        .columns
        .iter()
        .filter(|c| {
            c.kind == ColumnKind::Relation
                && c.config.target_base == s.current().and_then(|p| p.parent_id.clone())
        })
        .map(|c| (c.id.clone(), c.name.clone()))
        .collect();
    if !relations.is_empty() {
        let id = block.id.clone();
        let title = embed
            .relation_filter
            .as_ref()
            .and_then(|id| relations.iter().find(|(key, _)| key == id))
            .map_or("All records".to_string(), |(_, name)| {
                format!("Linked to this page via {name}")
            });
        box_ = box_.child(
            action(
                format!("embed-relation-{}", block.id),
                &title,
                p,
                move |s, _| {
                    if let Some(b) = s.blocks.iter_mut().find(|b| b.data.id == id)
                        && let Some(e) = &mut b.data.embedded
                    {
                        let next = e
                            .relation_filter
                            .as_ref()
                            .and_then(|id| relations.iter().position(|(key, _)| key == id))
                            .map_or(0, |i| i + 1);
                        e.relation_filter = relations.get(next).map(|(id, _)| id.clone());
                    }
                    s.commit_blocks(true);
                },
            )
            .justify_start(),
        );
    }
    if view == View::Dashboard {
        for column in base.columns.iter().filter(|c| {
            matches!(
                c.kind,
                ColumnKind::Number | ColumnKind::Rollup | ColumnKind::Formula
            ) && c.visible
        }) {
            let total = records
                .iter()
                .filter_map(|r| folio::calculations::value(r, column, &s.pages).ok())
                .sum::<f64>();
            box_ = box_.child(label(
                format!("{} : {}", column.name, folio::fields::format_number(total)),
                14.0,
                p.accent,
            ));
        }
    }
    if view == View::Board {
        let mut lanes = row().w_full().gap(7.0);
        for status in Status::ALL {
            let mut lane =
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .gap(5.0)
                    .child(label(status.name(), 11.0, p.muted));
            for page in records
                .iter()
                .filter(|p| p.properties.status == status)
                .take(12)
            {
                lane = lane.child(record(s, block, page, p));
            }
            lanes = lanes.child(lane);
        }
        return box_.child(
            col()
                .h(260.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(lanes),
        );
    }
    if view == View::Calendar {
        return box_.child(mini_calendar(s, block, &records, p));
    }
    box_.child(data_grid(s, block, &settings, &records, p))
}
fn record(_s: &State, block: &NoteBlock, page: &Page, p: Palette) -> El {
    let id = page.id.clone();
    let status_id = id.clone();
    let status = page.properties.status;
    row()
        .w_full()
        .gap(5.0)
        .center_y()
        .h(35.0)
        .radius(4.0)
        .fill(p.sidebar)
        .child(
            action(
                format!("embed-record-{}-{id}", block.id),
                page.display_title(),
                p,
                move |s, _| s.select(&id),
            )
            .grow(1.0)
            .min_w(0.0)
            .justify_start(),
        )
        .child(action(
            format!("embed-status-{}-{}", block.id, page.id),
            status.name(),
            p,
            move |s, _| {
                s.set_status(
                    &status_id,
                    match status {
                        Status::Todo => Status::InProgress,
                        Status::InProgress => Status::Done,
                        Status::Done => Status::Todo,
                    },
                )
            },
        ))
}
pub(super) fn media(s: &State, block: &NoteBlock, p: Palette) -> El {
    let Some(asset) = &block.asset else {
        let id = block.id.clone();
        return action(
            format!("attach-{id}"),
            "Choose an image or file…",
            p,
            |s, tx| s.attachment_dialog(tx),
        );
    };
    let open_id = asset.id.clone();
    let export = asset.clone();
    let mut body = col()
        .grow(1.0)
        .min_w(0.0)
        .gap(6.0)
        .padding(8.0)
        .radius(6.0)
        .border(1.0, p.line);
    if block.kind == Kind::Image
        && let Some(cache) = &s.workspace.images
        && let Ok(path) = folio::workspace::asset_path(&s.asset_folder(), &asset.id)
    {
        let image = cache.file(path, 800.0, 2.0);
        let width = 650.0 * asset.width_percent.clamp(20, 100) as f32 / 100.0;
        let height = image.size().map_or(180.0, |[w, h]| {
            (width * h as f32 / w.max(1) as f32).clamp(50.0, 550.0)
        });
        body = body.child(
            el().key(format!("image-{}", block.id))
                .w_full()
                .max_w(width)
                .h(height)
                .pixels_fit(image.pixels(), rune::SurfaceFit::Fit),
        );
        let id = block.id.clone();
        body = body.child(
            row().justify_end().child(
                el().key(format!("image-resize-{id}"))
                    .size(16.0, 10.0)
                    .fill(p.line)
                    .radius(3.0)
                    .help("Drag to resize the image")
                    .on_drag(move |s: &mut State, d, _| {
                        if let Some(b) = s.blocks.iter_mut().find(|b| b.data.id == id)
                            && let Some(asset) = &mut b.data.asset
                        {
                            if d.phase == rune::Phase::Down {
                                d.data = asset.width_percent as f32;
                            }
                            asset.width_percent = if d.cancelled() {
                                d.data as u16
                            } else {
                                (d.data + d.dx / 6.5).clamp(20.0, 100.0) as u16
                            };
                        }
                        if d.phase == rune::Phase::Up {
                            s.commit_blocks(true);
                        }
                    }),
            ),
        );
        if let Some(error) = image.error() {
            body = body.child(label(
                format!("Preview unavailable: {error}"),
                11.0,
                p.faint,
            ));
        }
    }
    body.child(
        row()
            .gap(5.0)
            .center_y()
            .child(
                label(
                    format!("{} · {} KiB", asset.name, asset.size / 1024),
                    11.0,
                    p.muted,
                )
                .grow(1.0)
                .min_w(0.0)
                .truncate(),
            )
            .child(action(
                format!("open-asset-{}", block.id),
                "Open",
                p,
                move |s, _| {
                    s.workspace_request(crate::workspace_worker::Request::OpenAsset(
                        open_id.clone(),
                    ))
                },
            ))
            .child(action(
                format!("export-asset-{}", block.id),
                "Export",
                p,
                move |s, tx| s.export_asset(&export, tx),
            )),
    )
}
pub(super) fn toc(s: &State, p: Palette) -> El {
    let mut body = col()
        .grow(1.0)
        .min_w(0.0)
        .gap(3.0)
        .padding(10.0)
        .radius(5.0)
        .fill(p.sidebar)
        .child(label("ON THIS PAGE", 10.0, p.faint));
    for b in s.blocks.iter().filter(|b| {
        matches!(
            b.data.kind,
            Kind::Heading1
                | Kind::Heading2
                | Kind::Heading3
                | Kind::ToggleHeading1
                | Kind::ToggleHeading2
                | Kind::ToggleHeading3
        )
    }) {
        let id = b.data.id.clone();
        let level = match b.data.kind {
            Kind::Heading2 | Kind::ToggleHeading2 => 1.0,
            Kind::Heading3 | Kind::ToggleHeading3 => 2.0,
            _ => 0.0,
        };
        body = body.child(
            action(format!("toc-{id}"), &b.edit.text, p, move |s, tx| {
                s.goto_block(&id, tx)
            })
            .w_full()
            .justify_start()
            .pl(9.0 + level * 12.0),
        );
    }
    body
}

fn data_grid(
    s: &State,
    block: &NoteBlock,
    settings: &folio::collection::Collection,
    records: &[&Page],
    p: Palette,
) -> El {
    let columns: Vec<_> = settings.columns.iter().filter(|c| c.visible).collect();
    let mut header = super::table::header_row(format!("embed-header-{}", block.id), 36.0, p)
        .child(
            super::table::column_heading("Name", p)
                .px(6.0)
                .w(200.0)
                .min_w(200.0)
                .grow(1.0),
        )
        .child(super::table::column_heading("Status", p).px(6.0).w(100.0))
        .child(super::table::column_heading("Due date", p).px(6.0).w(105.0));
    for c in &columns {
        header = header.child(
            super::table::column_heading(&c.name, p)
                .px(6.0)
                .w(settings.width(&c.id, 140.0)),
        );
    }
    let min_width = 441.0
        + columns
            .iter()
            .map(|c| settings.width(&c.id, 140.0) + 8.0)
            .sum::<f32>();
    let mut rows = col()
        .key(format!("embed-content-{}", block.id))
        .w_full()
        .min_w(min_width)
        .child(header);
    for (index, page) in records.iter().take(100).enumerate() {
        if index > 0 {
            rows = rows.child(
                el().key(format!("embed-separator-{}-{}", block.id, page.id))
                    .w_full()
                    .h(1.0)
                    .fill(p.line),
            );
        }
        let id = page.id.clone();
        let status_id = id.clone();
        let status = page.properties.status;
        let due_id = id.clone();
        let anchor = format!("embed-due-{}-{id}", block.id);
        let due_key = anchor.clone();
        let mut line = row()
            .key(format!("embed-row-{}-{id}", block.id))
            .role("row")
            .w_full()
            .h(40.0)
            .px(10.0)
            .gap(8.0)
            .center_y()
            .child(
                super::button(
                    format!("embed-record-{}-{id}", block.id),
                    page.display_title(),
                    p,
                    move |s, _| s.select(&id),
                )
                .w(200.0)
                .min_w(200.0)
                .grow(1.0)
                .px(6.0)
                .justify_start()
                .child(
                    label(page.display_title(), 12.0, p.muted)
                        .min_w(0.0)
                        .truncate(),
                ),
            )
            .child(
                action(
                    format!("embed-status-{}-{}", block.id, page.id),
                    status.name(),
                    p,
                    move |s, _| {
                        s.set_status(
                            &status_id,
                            match status {
                                Status::Todo => Status::InProgress,
                                Status::InProgress => Status::Done,
                                Status::Done => Status::Todo,
                            },
                        )
                    },
                )
                .w(100.0)
                .px(6.0)
                .justify_start(),
            )
            .child(
                action(
                    due_key,
                    page.properties.due.as_deref().unwrap_or("—"),
                    p,
                    move |s, _| s.open_date(&due_id, None, &anchor),
                )
                .w(105.0)
                .px(6.0)
                .justify_start(),
            );
        for c in &columns {
            let cell = super::custom_fields::cell_with_key(
                s,
                page,
                c,
                p,
                format!("embed-cell-{}-{}-{}", block.id, page.id, c.id),
            );
            line = line.child(el().w(settings.width(&c.id, 140.0)).child(
                if c.kind == ColumnKind::Checkbox {
                    cell.ml(6.0)
                } else {
                    cell
                },
            ));
        }
        rows = rows.child(line);
    }
    if records.is_empty() {
        rows = rows.child(label("No records in this view.", 12.0, p.faint).padding(12.0));
    }
    let count = records.len().min(100);
    let height = 36.0
        + if count == 0 {
            54.0
        } else {
            count as f32 * 40.0 + count.saturating_sub(1) as f32
        };
    super::table::frame(
        format!("embed-frame-{}", block.id),
        format!("embed-table-{}", block.id),
        "Embedded view records",
        rows,
        height,
        320.0,
        p,
    )
}
fn mini_calendar(s: &State, block: &NoteBlock, records: &[&Page], p: Palette) -> El {
    use chrono::Datelike;
    let month = s.calendar_month();
    let mut body = col().w_full().gap(4.0).child(
        row()
            .gap(6.0)
            .center_y()
            .child(icons::control(
                format!("embed-month-prev-{}", block.id),
                "Previous month",
                &folio::icons::controls::CHEVRON_LEFT,
                p,
                |s, _| s.step_calendar(false),
            ))
            .child(label(month.format("%m / %Y").to_string(), 13.0, p.ink))
            .child(icons::control(
                format!("embed-month-next-{}", block.id),
                "Next month",
                &folio::icons::controls::CHEVRON_RIGHT,
                p,
                |s, _| s.step_calendar(true),
            )),
    );
    for week in crate::calendar::days(month).chunks(7) {
        let mut line = row().w_full().gap(3.0);
        for day in week {
            let mut cell = col()
                .grow(1.0)
                .min_w(0.0)
                .h(88.0)
                .padding(4.0)
                .gap(3.0)
                .border(1.0, p.line)
                .radius(4.0);
            if let Some(day) = day {
                let date = day.format("%Y-%m-%d").to_string();
                cell = cell.child(label(day.day().to_string(), 10.0, p.faint));
                for page in records
                    .iter()
                    .filter(|p| p.properties.due.as_deref() == Some(&date))
                    .take(2)
                {
                    let id = page.id.clone();
                    cell = cell.child(
                        action(
                            format!("embed-calendar-{}-{id}", block.id),
                            page.display_title(),
                            p,
                            move |s, _| s.select(&id),
                        )
                        .h(23.0)
                        .px(2.0)
                        .w_full()
                        .justify_start(),
                    );
                }
            }
            line = line.child(cell);
        }
        body = body.child(line);
    }
    body
}
