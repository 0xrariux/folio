use super::{Palette, label};
use crate::{
    overview::{Item, Tasks},
    state::State,
};
use folio::{dashboard::Summary, document::Kind, icons::controls as g, outline};
use rune::{El, ScrollSpec, TextStyle, col, row, text_input};

fn metric(key: &str, title: &str, count: usize, p: Palette) -> El {
    col()
        .key(key)
        .grow(1.0)
        .min_w(0.0)
        .padding(12.0)
        .gap(8.0)
        .radius(7.0)
        .border(1.0, p.line)
        .child(label(title, 11.0, p.muted))
        .child(label(count.to_string(), 26.0, p.ink))
}

fn entry(item: &Item<'_>, task: bool, p: Palette) -> El {
    let page = item.page.id.clone();
    let block = item.block.id.clone();
    let title = item.block.plain_text();
    let title = if title.trim().is_empty() {
        if task {
            "Untitled task"
        } else {
            "Untitled item"
        }
        .into()
    } else {
        title
    };
    row()
        .key(format!("overview-item-{block}"))
        .role("button")
        .label(format!("{title} — {}", item.page.display_title()))
        .focusable(true)
        .w_full()
        .min_w(0.0)
        .py(9.0)
        .px(8.0)
        .gap(10.0)
        .radius(5.0)
        .hover(p.tint)
        .center_y()
        .on_press_tx(move |s: &mut State, tx| s.open_overview_item(&page, Some(&block), tx))
        .child(
            rune::icon(
                if task {
                    if item.block.checked {
                        &g::CHECK
                    } else {
                        &g::CLOCK
                    }
                } else {
                    &g::BULLET_LIST
                },
                16.0,
            )
            .color(p.accent)
            .a11y_hidden(),
        )
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(3.0)
                .child(
                    label(
                        title,
                        14.0,
                        if task && item.block.checked {
                            p.faint
                        } else {
                            p.ink
                        },
                    )
                    .w_full()
                    .truncate(),
                )
                .child(
                    label(item.page.display_title(), 10.0, p.muted)
                        .w_full()
                        .truncate(),
                ),
        )
        .child(
            rune::icon(&g::ARROW_UP_RIGHT, 13.0)
                .color(p.faint)
                .a11y_hidden(),
        )
}

fn section(title: &str, p: Palette) -> El {
    col()
        .w_full()
        .min_w(0.0)
        .gap(6.0)
        .padding(12.0)
        .radius(7.0)
        .border(1.0, p.line)
        .child(label(title, 15.0, p.ink).role("heading").level(2))
}

pub(super) fn view(s: &State, p: Palette) -> El {
    let summary = s.notebook_overview();
    let mut body = col()
        .w_full()
        .gap(16.0)
        .child(label("Overview", 32.0, p.ink).role("heading").level(1))
        .child(
            row()
                .gap(8.0)
                .child(metric("overview-notes", "Notes", summary.notes.len(), p))
                .child(metric("overview-tasks", "Tasks", summary.tasks.len(), p))
                .child(metric("overview-done", "Completed", summary.done, p))
                .child(metric(
                    "overview-lists",
                    "List items",
                    summary.lists.len(),
                    p,
                )),
        )
        .child(
            text_input(&s.overview.search, TextStyle::new(13.0, 400.0))
                .key("overview-search")
                .label("Search notebook overview")
                .placeholder("Search notes and tasks…", p.faint)
                .w_full()
                .h(34.0)
                .padding(8.0)
                .radius(5.0)
                .border(1.0, p.line)
                .color(p.ink)
                .caret(p.accent, p.tint)
                .on_edit(|s: &mut State| &mut s.overview.search)
                .on_input_change(|s: &mut State, _| s.overview.limit = 40),
        );

    let mut tasks = section("Tasks", p);
    let mut filters = row().gap(5.0);
    for (key, title, filter) in [
        ("all", "All", Tasks::All),
        ("open", "Open", Tasks::Open),
        ("done", "Done", Tasks::Done),
    ] {
        filters = filters.child(
            super::action(format!("overview-filter-{key}"), title, p, move |s, _| {
                s.overview.tasks = filter;
                s.overview.limit = 40;
            })
            .fill(if s.overview.tasks == filter {
                p.tint
            } else {
                p.bg
            }),
        );
    }
    tasks = tasks.child(filters);
    let visible: Vec<_> = summary
        .tasks
        .iter()
        .filter(|item| match s.overview.tasks {
            Tasks::All => true,
            Tasks::Open => !item.block.checked,
            Tasks::Done => item.block.checked,
        })
        .collect();
    for item in visible.iter().take(s.overview.limit) {
        tasks = tasks.child(entry(item, true, p));
    }
    if visible.is_empty() {
        tasks = tasks.child(label(
            if summary.tasks.is_empty() {
                "No tasks here yet."
            } else {
                "No tasks match this filter."
            },
            12.0,
            p.faint,
        ));
    }
    body = body.child(tasks);

    if !summary.lists.is_empty() {
        let mut lists = section("List items", p);
        for item in summary.lists.iter().take(s.overview.limit) {
            lists = lists.child(entry(item, false, p));
        }
        body = body.child(lists);
    }

    let mut notes = section("Notes", p);
    for page in summary.notes.iter().take(s.overview.limit) {
        let id = page.id.clone();
        let (done, total) = outline::task_totals(&page.blocks);
        let lists = page
            .blocks
            .iter()
            .filter(|b| matches!(b.kind, Kind::Bullet | Kind::Numbered))
            .count();
        notes = notes.child(
            row()
                .key(format!("overview-note-{id}"))
                .role("button")
                .label(page.display_title())
                .focusable(true)
                .w_full()
                .min_w(0.0)
                .py(9.0)
                .px(8.0)
                .gap(10.0)
                .center_y()
                .radius(5.0)
                .hover(p.tint)
                .on_press_tx(move |s: &mut State, tx| s.open_overview_item(&id, None, tx))
                .child(super::icons::page_icon(page, 19.0, p.accent, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(3.0)
                        .child(label(page.display_title(), 14.0, p.ink).w_full().truncate())
                        .child(label(
                            format!("{done}/{total} tasks · {lists} list items"),
                            10.0,
                            p.muted,
                        )),
                )
                .child(
                    rune::icon(&g::ARROW_UP_RIGHT, 13.0)
                        .color(p.faint)
                        .a11y_hidden(),
                ),
        );
    }
    if summary.notes.is_empty() {
        notes = notes.child(label("No notes here yet.", 12.0, p.faint));
    }
    body = body.child(notes);

    if !summary.databases.is_empty() {
        let mut databases = section("Databases", p);
        for page in summary.databases.iter().take(s.overview.limit) {
            let id = page.id.clone();
            let records: Vec<_> = s
                .pages
                .iter()
                .filter(|p| {
                    p.parent_id.as_deref() == Some(&id)
                        && !p.archived
                        && p.protected.is_none()
                        && !s.settings.unlocked.contains(&p.id)
                })
                .collect();
            let stats = Summary::from_pages(&records, &s.today);
            databases = databases.child(
                row()
                    .key(format!("overview-database-{id}"))
                    .role("button")
                    .label(page.display_title())
                    .focusable(true)
                    .w_full()
                    .min_w(0.0)
                    .py(9.0)
                    .px(8.0)
                    .gap(10.0)
                    .center_y()
                    .radius(5.0)
                    .hover(p.tint)
                    .on_press_tx(move |s: &mut State, tx| s.open_overview_item(&id, None, tx))
                    .child(super::icons::page_icon(page, 19.0, p.accent, p))
                    .child(
                        col()
                            .grow(1.0)
                            .min_w(0.0)
                            .gap(3.0)
                            .child(label(page.display_title(), 14.0, p.ink).w_full().truncate())
                            .child(label(
                                format!("{}/{} records completed", stats.done, stats.total),
                                10.0,
                                p.muted,
                            )),
                    )
                    .child(
                        rune::icon(&g::ARROW_UP_RIGHT, 13.0)
                            .color(p.faint)
                            .a11y_hidden(),
                    ),
            );
        }
        body = body.child(databases);
    }
    if [
        visible.len(),
        summary.lists.len(),
        summary.notes.len(),
        summary.databases.len(),
    ]
    .iter()
    .any(|&count| count > s.overview.limit)
    {
        body = body.child(super::action("overview-more", "Show more", p, |s, _| {
            s.overview.limit += 40
        }));
    }
    col()
        .key("notebook-overview")
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .child(
            row()
                .h(51.0)
                .px(18.0)
                .gap(6.0)
                .center_y()
                .border_sides([0.0, 0.0, 1.0, 0.0], p.line)
                .child(super::icons::control(
                    "overview-back",
                    "Back to notes",
                    &g::CHEVRON_LEFT,
                    p,
                    |s, _| s.close_overview(),
                ))
                .child(label("Workspace / Overview", 12.0, p.muted)),
        )
        .child(
            col()
                .key("document-scroll")
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(col().w_full().px(30.0).py(25.0).child(body)),
        )
}
