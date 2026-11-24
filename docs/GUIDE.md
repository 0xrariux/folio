# Folio guide

Folio 0.16 is a local native notebook built with Rust, Rune and Turso.
The interface, templates and documentation are in English.
[Back to README](../README.md)

## Start Folio

The supported application targets Apple Silicon Macs running macOS 13 or later.
Open `Folio.app` from a reviewed release. Downloaded builds are ad-hoc signed,
without Apple Developer ID or notarization; macOS may require manual approval.
See [builds and releases](RELEASES.md) for distribution details.

For development, clone Folio:

```sh
git clone https://github.com/0xrariux/folio.git
cd folio
cargo run --locked
```

`rust-toolchain.toml` selects Rust 1.100.0. `Cargo.lock` resolves published
libraries, including Turso 0.8.4. Cargo downloads Rune and its vendor patches at the full Git SHA recorded in
`Cargo.toml` and `folio-update.json`; it does not use a neighboring Rune checkout.
The first compilation can take several minutes.

To build a native application:

```sh
python3 scripts/build-app.py --development --verify-storage
open dist/Folio.app
```

Use an isolated development notebook or fictional demo content:

```sh
FOLIO_DB_PATH="$PWD/data/dev.db" cargo run --locked
FOLIO_DEMO=1 cargo run --locked -- --scene workspace
```

Demo edits stay in memory and reset at launch. Normal notebooks persist outside
the executable in `~/Library/Application Support/Folio/`. Templates create pages
only when requested. Deleting the welcome page or emptying the notebook is
persistent; templates and demo pages do not reappear in a real notebook.

## Pages and navigation

Create pages and subpages with the sidebar plus button or Command-N. Rename a
page through its title. The page menu offers duplication, moving, full width,
archiving and confirmed permanent deletion. Duplication copies content and
appearance, leaving subpages attached to the original.

The sidebar and page menus both expose **Delete page…**. Permanent deletion also
removes that page's local history; children move to its parent and remain intact.
Older backups remain restorable. Unlock a protected page before deleting it.
Archiving retains content, and active children of an archived parent remain
accessible at the root until the parent is restored.

Drag notebook pages between rows to reorder, onto the center of a page to nest,
or into the root drop area to detach. Highlighted insertion lines show the
position. Cycles are refused; Escape cancels the gesture. Breadcrumbs and back
and forward navigation keep access to the surrounding pages.

Drag the top shortcuts to reorder them. Their **Shortcuts → ⋯** menu also offers
move, hide, restore and reset actions. Hiding a shortcut never deletes pages;
the menu remains accessible even with all shortcuts hidden. The layout, sidebar
width, chosen theme and last page are saved with the notebook.

Click the page icon to choose from 105 symbols and eight colors. Reset returns to
the theme color. Existing emoji identifiers remain readable. Cover selection
cycles through three colors and then removes the cover. Dark is the initial theme.
See [icons](../assets/icons/README.md) and [logo sources](../assets/brand/README.md).

## Writing and organizing blocks

Folio supports rich text, three heading levels, bulleted and numbered lists,
tasks, quotes, callouts, code, dividers, toggles and collapsible headings, links,
embedded views, a table of contents, images and files.

Type `/` at the start of a block and filter commands. Choose with the arrow keys
and Enter; Escape closes the menu while keeping the typed text. Historical
French aliases remain supported. The type selector transforms an existing block
without discarding its text. The block plus button inserts a following block.

Click a block's grip for its move, duplicate and delete menu. Drag the grip to
reorder its subtree, including hidden children. The target determines the group's
indentation while preserving relative child indentation, up to six levels.
The insertion line and floating preview show the destination; changes commit
only on drop, as one undo step. The document scrolls near its edges. Escape,
window focus loss or dropping outside the editor cancels the gesture.
Drop on the middle of an item to nest beneath it. Its highlight and the
"Nest inside" preview confirm the target. Drop near the top or bottom to reorder.

Enter splits text with its styles. Headings continue as text; lists and tasks
continue their current type. Enter in an empty nested item outdents it, and in
an empty root list returns to text. Shift-Enter inserts a line inside a block;
Enter in code does the same. Tab and Shift-Tab indent and outdent a subtree.
Backspace at the beginning of an item outdents it or converts a root list to text.
Consecutive empty paragraphs are reduced to one when opening or editing a note.
Undo can restore the original spacing. Repeated Enter on an empty paragraph
stays on that line; empty tasks, headings, code and parents retain their content.

Items with children, toggles and collapsible headings retain their open state.
Their chevrons hide or show descendants without changing nested toggle states. Enter in a toggle title
creates content. The block menu can add a child, restart or continue numbering,
and check or uncheck a task list including hidden tasks. The page menu can expand
or collapse all groups. Numbering continues after nested lists and restarts for
each parent. A typed number such as `5.` can start a sequence at that number.

Use **Add a task** below the note, **Task** in the type picker, or the block menu's
**Turn into a task** to create a checkbox. A single task can be checked directly;
no child is required or created. **Add a subtask** adds a child only when requested.
Converting a point to a task also converts its existing nested text and list
points, including folded children. Existing checked tasks keep their state;
files, links, embedded views and code keep their type. Undo restores the group.
Choose **Task** again on an existing task to convert its remaining nested points.
An item containing tasks shows its completion and checks itself when every leaf
task is complete, including tasks inside collapsed groups. Reopening a child or
adding a new subtask reopens the parent. Clicking a parent's checkbox checks or
unchecks its whole subtree in one undo step; unrelated tasks remain unchanged.

Fully completed tasks move into **Completed** below the note, with a count of
completed entities. A nested group moves only when all its leaf tasks are done;
checked children stay with an unfinished parent. Folded descendants and any
supporting content stay with their group. Uncheck a task to return the group to
its original place, or use Undo. Adding an unfinished subtask reopens the group.
**Add a task** stays above Completed. This grouping is a view of the saved note:
the original order, identities, indentation, backups and exports are preserved.

At the beginning of an empty text block, type a prefix followed by a space:

| Prefix | Result |
|---|---|
| `-`, `*`, `+` | Bullet |
| `1.` | Numbered item |
| `[]`, `[ ]`, `[x]` | Task, unchecked or checked |
| `>` | Toggle |
| `>#`, `>##`, `>###` | Collapsible heading |
| `#`, `##`, `###` | Heading |
| `"` | Quote |
| Three backticks | Code |
| `---` | Divider |

Pasting multiple lines creates separate blocks in one undo step. HTML lists keep
styles and indentation; plain lists recognize prefixes and four spaces or a tab
per level. Pasting into code keeps its line breaks in one block. Hidden content
remains searchable and is exported; toggles become HTML `<details>` in Markdown.

Command-click grips to select blocks, or Shift-click for a range. Selection
commands copy, paste, duplicate, transform, move or delete subtrees in one undo
step. Internal block paste creates fresh identifiers. Escape clears selection.

| Shortcut | Action |
|---|---|
| Command-N | New page |
| Command-K | Global page and command palette |
| Command-S | Retry failed saves |
| Command-B / I / U | Bold / italic / underline |
| Command-D in a block | Duplicate block and descendants |
| Tab / Shift-Tab | Indent / outdent group |
| Option-Shift-Up / Down | Move group among siblings |
| Command-Enter | Check a task or toggle collapsed state |
| Command-Z / Command-Shift-Z | Undo / redo in the current document |
| F6 / Shift-F6 | Move between navigation and content |

Undo is scoped to the current page and session; durable history remains available
after restart. Enter in a page title focuses its first block.

## Collections, tables and views

A collection is a database of records; every record is also a page with blocks.
Table, Kanban, Calendar and Dashboard show the same underlying records.
Built-in properties include status, priority, due date and tags. Dates use
`YYYY-MM-DD`; records accept up to 12 tags of 40 characters each.

Create records in a status column, open a record for notes, or drag Kanban cards
between statuses. Archived records are excluded. Each named view saves its own
layout, combined filters, sort, column widths, order and visibility. Views can be
created, renamed or removed without deleting records, with a limit of 20 views
and at least one retained. Record search is separate from global sidebar search.

Custom columns support text, number, date, checkbox, relation, rollup and formula.
Names and entered values are validated. Computed cells are read-only. Table
checkboxes select records for bulk status, priority or date changes. Table rows
are virtualized with stable record keys; selecting all includes filtered records
that are currently off screen.

Relation columns select a target collection. An optional reverse relation is
created by default and reflects links automatically; either side can edit them.
Archived records are excluded from displayed relation values and calculations.
Rollups compute count, completed count, completion percentage, a numeric sum or
remaining count through a relation.

Numeric formulas support numbers, `+`, `-`, `*`, `/`, parentheses and
`prop("Column name")`. `status_done` is 1 for completed records and 0 otherwise.
For example: `prop("Total effort (h)") * 80`. Dependencies recalculate on read;
cycles and division by zero display an error. CSV exports include computed values.

## Overview, dashboards and calendar

**Overview** is a built-in shortcut view; opening it never creates a page.
It summarizes notes, leaf tasks, completion, and bullet/numbered list items,
including folded content. Search or filter tasks, then click an item to open its
source. Archived and PIN-protected content is excluded. Database summaries appear
separately when databases exist. Empty legacy auto-created Overview pages are
hidden from the regular tree, retained in storage, and available through sidebar search.

A **Project dashboard** page chooses collections and its own filters independently of source views.
Widgets show progress, statuses, deadlines, numeric totals and checkbox counts;
chart segments and records provide navigation into the underlying data.
A new database instead creates a collection that holds its own records.
Widgets currently have a fixed layout.

An embedded database block chooses an existing collection and saved view.
Changes update the originals, other views and calculations. On a project record,
a relation filter can limit the embedded view to linked tasks; newly created
records receive that relation. Embedded views show up to 100 rows; open the source
collection to reach the rest.

Calendar shows due dates Monday through Sunday. Drag a record to change its date.
Undated records remain accessible below the grid. Click a day to open its records
or create one with that date. Date cells and due dates use native Rune pickers.

## Links, search and media

Command-K searches pages and commands with keyboard navigation. Unicode matching
supports case and width folding; `reunion` finds `Réunion`, while an accented
query keeps that accent as a requirement. The palette highlights original title
ranges. Name uniqueness and sorting have separate rules.

`/link` inserts a page link after choosing its target; converting existing text
keeps that text and inserts the link after it. `@` inserts a page mention. Block
menus copy links to a specific block. Backlinks include mentions and group
references from the same source, excluding archived sources. Renaming a target
updates labels without changing its identity. Missing targets remain visible as
missing pages. A table of contents follows headings and opens toggles as needed.

Image/file commands, the notebook menu or dropping files into a note import local
files. Originals are copied into the managed asset folder, so moving the source
does not break the note. Each import accepts up to 20 files of 50 MiB each. Images
resize to 20–100% of available width. System opening and original-file export are
available. Markdown export copies media into an adjacent `folio-assets` directory.

Notes and records export to Markdown; collection and dashboard scopes export to
CSV, either the current view or all records. Export uses a native destination
dialog, background atomic writes, cancellation and visible success/errors.
Exports are plaintext; protected-note exports require an explicit unlocked page.

## Persistence, history and backups

Turso runs on a dedicated worker. Each edit sends a page snapshot; its page row
and authoritative versioned JSON document commit in the same transaction.
Derived HTML/plain text support display and search. **All changes saved** waits
for acknowledged revisions. Failed writes remain visible and can be retried.
Orderly exit waits for queued writes; a forced stop can lose unacknowledged edits.
Folio does not include a browser or JavaScript engine.

History retains up to 50 versions per page, with an automatic prior version per
minute and explicit checkpoints. Restoring a version also keeps the current one.
The notebook menu provides history, backup, restore and file import. A `.folio`
backup contains pages, views, custom properties, history, preferences, assets and
module state. Restore previews counts, verifies files, keeps a verified recovery
copy of the previous notebook, then replaces content transactionally.

Automatic backups run on opening and after a changed session, including vault
lock. The last 14 normal backups are retained beside the database in
`folio.backups/`; assets live in `folio.assets/`. Custom database paths use the
same sibling extensions. Backup files are limited to 512 MiB, including at most
256 MiB of raw attachment data. Close Folio before manually copying a profile;
keep its database, WAL, configuration and assets together.

Current document format is 7, with readers for versions 1–6. SQL schema 4 adds
the module registry. Portable backups with module state use notebook format 2;
empty registries retain format 1. Unsupported versions fail explicitly rather
than dropping data. See [storage safety](DATA_SAFETY.md) for preflight, raw
snapshots, frozen historical files, encrypted fixtures and recovery limits.

## Settings, security, updates and modules

The sidebar gear opens **Settings**. Database encryption is opt-in with a master
password; eligible simple notes can also use individual PINs. Attachments remain
separate plaintext files. Turso's native encryption is experimental. See
[security and settings](SETTINGS.md) for the exact scope and recovery requirements.

Update checks compare signed published stable GitHub releases, not `main`.
The installed app downloads and replaces a prebuilt bundle after a verified
backup; a failed launch restores the previous bundle. An incompatible storage
contract requires a reviewed manual migration. See [release operations](RELEASES.md).

**Library** manages local module definitions and catalogs, reviewed installation,
activation, deactivation, uninstall and retained data. No functional modules or
external executable runtime are included. See [module lifecycle](MODULES.md).

## Development and validation

The main boundaries are `document.rs`/`editor.rs` for blocks, `collection.rs` and
related view modules for records, `workspace.rs` for portable notebooks,
`storage/` for Turso and recovery, `worker.rs` for serialization and shutdown,
`security.rs`/`vault.rs` for protection, `modules.rs` for the module host, and
`ui/` for native views. `main.rs` handles startup, wakeups and shortcuts.

```sh
cargo fmt --package folio -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s scripts/tests -v
python3 scripts/build-app.py --development --verify-storage
```

Screenshots and scripted interactions use fictional content. Debug tooling is
available by default; release captures require the `tooling` feature:

```sh
FOLIO_DEMO=1 cargo run --release --locked --features tooling -- --snap /tmp/folio.png
```

The interface provides navigation/content landmarks, named headings and save,
export and error announcements. F6 focus navigation is tested; real VoiceOver
usage still requires qualification. See [Rune integration notes](RUNE_UPDATE.md).

## Current limits

No synchronization, collaboration, hosted sharing, Notion/Markdown/CSV import,
gallery, timeline, block columns, free widget layout, full formula language or
executable modules are implemented. The notebook is loaded in memory; table row
virtualization does not make all navigation or editing virtualized. A successful
build is not a guarantee against engine or hardware faults. Keep an independent,
verified backup outside the Mac.
