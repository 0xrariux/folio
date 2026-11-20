# Changelog

## 0.13.2

- Remove favorite stars from notebook sidebar rows. Favorite actions stay on the
  page, and the Favorites shortcut remains available.
- Expand the page picker to 65 symbols with 30 original icons for work, creativity,
  nature and travel. All symbols use the existing eight theme-aware colors.
- Add shared SVG/Blender stroke sources, an editable Blender icon sheet and preview.
  Existing page icons and notebook data keep their stored identities.

## 0.13.1

- Rust 1.100.0 and Rune pinned to `f54cbcfa84d995d339c243a579218f5062c940b5`
  (89 commits since the previous pin), with the new internal crate split and
  aligned vendor patches. Cargo fetches the exact Git sources without relying on
  a shared neighboring checkout. Published dependencies remain locked.
- Use Rune’s mouse-down activation for page, More and shortcut menus, while
  retaining keyboard activation and existing drag/drop behavior.
- Resolve Git metadata paths for application build provenance in worktrees;
  validate Rune/vendor Git sources during packaging and accept both Cargo
  dependency forms in the weekly version report.
- Keep Turso at 0.8.3, with its unused optional FTS extension disabled because
  its allocator shim conflicts with Rust 1.100. Encryption and the storage/document
  contracts stay unchanged; historical storage and module checks remain required.
- Receive upstream fixes for Metal resource lifetime, long-field accessibility,
  scroll targeting, popup interaction and CPU/GPU rendering consistency.
  [Upgrade impact and validation](docs/RUNE_UPDATE.md).

## 0.13.0

- Library with discovery, local JSON definition/catalog imports, permission review,
  installation, enable/disable, reviewed upgrades and retained-data management.
- Module API 1: versioned definitions, compatibility checks, isolated JSON storage
  and revocable sessions. Only the metadata runtime is supported; no executable
  code or functional modules are included.
- Uninstall preserves state; confirmed deletion creates a verified recovery backup.
  Module state survives notebook encryption, backup, restore and application updates,
  including unavailable future definitions. Module-only changes are backed up on exit
  and vault lock; entering Library relocks protected notes.
- SQL schema 4 and portable notebook format 2 when module state is present, with
  historical corpus and module lifecycle checks required in production builds.
  The new storage contract requires a reviewed manual upgrade from schema-3 clients.

## 0.12.0 — development changes included in 0.13.0

- Fail-closed Turso preflight on a private copy, raw database/WAL recovery snapshots,
  transactional schema markers and strict document/history readers.
- Pre-update backups now require complete restoration, reopening and comparison,
  with a file fingerprint checked again before installation. Manual restore keeps
  a verified recovery backup; vault conversion verifies the complete notebook.
- Frozen SQLite/Turso 0.7.2 and encrypted 0.8.3 compatibility fixtures, all document
  versions, actual process-kill WAL recovery, and mandatory release-profile checks.

- Notebook page dragging now highlights the dragged page and valid destinations,
  with insertion lines between rows and an outline for nesting inside a page.

- Customizable sidebar shortcuts: drag to reorder, hide, restore, or reset Overview,
  All pages, Favorites, and Archives. Layout is saved with notebook preferences.

- Sidebar hover now covers the entire rounded page row. Each page has a nearby
  actions menu, with confirmed deletion available directly for active pages.

- Page icon picker expanded to 35 symbols, with eight theme-aware colors and
  a reset option. Page colors persist in history, backups, and protected notes.

- English interface, templates, welcome page, and fictional demo content.
  Existing notebooks retain their original content and stored identifiers.
- Public-facing README in English, portable documentation paths, sanitized
  Blender source metadata, and additional local database/credential exclusions.

- Native Apple Silicon `Folio.app` packaging with icon, build provenance and
  ad-hoc macOS signature. Source versions and dependency pins stay in the repository.
- Published stable releases replace source compilation in the app: Ed25519-signed
  metadata, archive size/SHA256, bundle verification, full pre-update backup and
  separate installer with startup acknowledgement and bundle rollback.
- Read-only weekly dependency intake, manually dispatched draft-release workflow,
  and a separate implementation prompt for the future Nexus project manager.
- Minimal settings include daily release checks, an available-update indicator
  and release notes; no developer tools are required by the installed app.
- Optional local vault: master password, native AES-256-GCM database and WAL
  encryption, encrypted automatic backups, verified migration and explicit locking.
  Turso still marks the native encryption API experimental.
- PIN-protected simple notes: title, content and history encrypted with Argon2id
  and XChaCha20-Poly1305; relock on leaving the note and after restart. Attachments
  and container pages are outside the scope of per-page PIN protection.
- Confirmed permanent deletion, retained children and no reinitialization of an
  intentionally emptied notebook.
- Document format 7, with readers for versions 1–6.
  [Settings, updates and encryption scope](docs/SETTINGS.md).
- Rune updated from `70eff811` to `7a92d24d` (627 commits): application-level vendor
  patches, aligned lockfile/guides and orderly exit preserving queued writes and
  the final notebook backup. The `tooling` feature enables release captures.
  [Integration details](docs/RUNE_UPDATE.md).
- Database tables built with `virtual_rows`: stable keys, offscreen focus and
  selection of all filtered records remain supported.
- Unicode search across pages, commands, links and properties, including accent
  folding and highlighted original title ranges in the palette.
- Named navigation/content landmarks, heading levels, save/export/error
  announcements and tested F6/Shift-F6 navigation.
- Large-view measurements and repeatable table-resize and Unicode-search captures.

## 0.10.0

- Embedded tables fill available width, including after resizing. Tables and
  dashboards share a frame/header with one border and separators between records.
- New Blender-built Folio logo: solid F with folded-leaf ends, revised badge,
  editable `.blend`, PNG renders and SVG exports, integrated in the sidebar/catalog.
- Secondary sidebar commands grouped under More: page/database creation,
  templates and notebook tools, with common actions still visible.
- Left-aligned collection-progress labels, navigation commands and property grids.
- Durable history of 50 versions per page, checkpoints and version restoration
  that also keeps the current state.
- Full `.folio` backup/restore: pages, views, files, history and preferences;
  preview, transactional replacement and a backup of the previous notebook.
  The last 14 automatic backups are retained.
- Multiple block selection, copy/paste, duplication, transformation, deletion and
  subtree moves in one undo step.
- Mouse dragging for Kanban cards, notebook pages and calendar dates, with saved
  order, cycle prevention, Escape cancellation and automatic scrolling.
- Database views embedded in notes/records, saved-view selection, project-relation
  filters and direct creation/editing of source records.
- Automatic reverse relations, rollups for count/completed/progress/sum/remaining,
  and numeric formulas.
- Monthly calendar, day detail, creation on a date and native Rune date pickers.
- Managed local images/files, image sizing, system opening, original-file recovery
  and media included in Markdown exports.
- Global Command-K palette, `@` mentions, block links, dynamic contents and backlinks.
- Per-view column widths/order/visibility and bulk record status/priority/date edits.
- Persistent theme, sidebar width and last page, with dark as the initial theme.
- Document format 6, reading formats 1–5; workspace/calendar demos and tested
  mouse/keyboard flows, files and restoration.

## 0.9.0

- Toggles and collapsible H1/H2/H3 headings, nested up to six indentation levels.
  Open state is persistent, undoable and keyboard accessible.
- Tab/Shift-Tab, toolbar and menu organization; child creation and expand/collapse all.
- Mouse/keyboard subtree moves, duplication and deletion include hidden children
  in one undo step.
- Per-level numbering continues after sublists, with restart/continue commands
  and depth-specific bullets.
- Markdown input shortcuts, rich multiline list paste, exiting empty lists and
  Backspace outdenting.
- Toggle task progress, whole-list check/uncheck, Outline notebook template and
  the `lists` scene.
- Markdown export preserves numbering, six levels and HTML `<details>` trees;
  hidden content remains searchable and exported.
- Document format 5 reads formats 1–4; older clients refuse it to preserve state
  and numbering restart points.
- Dark theme by default, with a persistent light-theme choice.

## 0.8.0

- Expanded the Folio family to 59 SVGs: 43 command/navigation/block/property/view signs.
- Shared icons in the sidebar, menus, tables, Kanban, dashboards and editor grips.
- Accessible labels/tooltips for icon-only buttons; consistent selections, stars
  and checkboxes.
- Standalone light/dark sheets and native `icon-catalog`/`icon-controls` scenes
  inspect drawings at three sizes.
- Existing notes and identifiers preserved; document format remains 4.

## 0.7.0

- Original Folio SVG family: 15 page symbols and a mark, with shared strokes and
  theme-aware colors.
- Catalog symbols replace emojis in navigation, titles, records, links, selectors
  and callouts.
- Grid picker with accessible names, tooltips and active selection; previous
  identifiers preserved without rewriting existing notes.
- Editable sources, standalone light/dark catalog and three-size `icon-catalog` scene.

## 0.6.1

- Tab separator sits beneath buttons with dedicated spacing, clear of selected
  backgrounds and hover states.
- Table outline independent of scrolling; layout reserves border space and clips
  content to rounded corners.
- Removed the double bottom line; row-only separators, aligned headers/cells and
  a header that remains visible while scrolling vertically.
- `launch-table` inspection scene and light/dark comparison captures.

## 0.6.0

- Grip-based mouse block dragging in both directions and to the end, with floating
  preview and insertion marker.
- Automatic scrolling near document edges; Escape, focus loss and dropping outside
  cancel. Click and keyboard block menus remain available.
- One undo step and save on drop, without writes during hover; rich text, IDs,
  indentation and checked states preserved, with numbering following block order.
- Numbering redraws after reordering; Rune input-focus synchronization preserves
  the active block.
- `drag-list` scene, demo scripts and tests for dragging, cancellation, scrolling,
  rich content, history and Turso persistence.
- Rune advanced from `8a6f2f42` to `70eff811` with `git pull --ff-only` (56 commits).
  Document format remains 4.

### Rune changes received November 9–10, 2026

- macOS reloads precompiled Metal shaders; stabilized animations, dragging and
  line sizing with inline code.
- Fixed empty-input undo, Tab navigation, macOS menus and release of Secure Event
  Input when changing windows.
- Handler-side `Tx::focused` and `Tx::by_keys` distinguish keyboard activation
  from clicking, used by Folio grips.
- Additional text-rendering modes, CJK segmentation, paragraph direction,
  persistent text highlights and multiline tooltips.
- Mermaid rendering and ToastStack improvements.

Dragging events already existed before this update. Folio uses `El::on_drag`,
`Drag::cancelled`, target queries and gesture repetition.

## 0.5.0

- Named database views with independent layout, combined filters and sorting;
  active selection and settings persist automatically.
- Create, rename and remove views without deleting records; validated names,
  at most 20 views and at least one retained.
- Markdown export for notes/records, including blocks, styles, code and properties.
- CSV export for a current view or full scope, preserving hidden-column handling
  and numeric precision.
- Native destination picker, cancellation, background atomic writes and visible
  success/errors; protects the active database from export overwrite.
- Reads document formats 1–3 and writes format 4; view persistence, migration and
  UI export-flow tests.
- Saved-view and overdue-tracking demo scenes.

## 0.4.0

- Custom text, number, date and checkbox columns: create, rename, show/hide and
  edit with validation.
- Combined status/priority/due-date filters shared by collection views.
- Collection Dashboard view and global dashboards spanning selected collections.
- Dynamic progress/status/deadline indicators, clickable charts, numeric totals
  and checkbox counts.
- Collection progress, upcoming deadlines and links to record detail.
- Dashboard template, fictional examples and preview scenes.
- Reads older documents and writes format 3, preserving IDs and hidden values in Turso.

## 0.3.0

- Full-screen record collections: each record is a note page with built-in status,
  priority, date and tag properties.
- Table and Kanban over the same records, creation by status, separate record
  search, status filtering and four persistent sort orders.
- Date/tag form with validation and cancellation.
- `/link` and `/lien` page links, target search, navigation and automatic backlinks;
  references follow renames.
- Project tracking template and fictional database in demo mode.
- Reads document format 1 and writes format 2, with defaults for new fields and
  transactional Turso writes.
- Feature comparison documented in `NOTION-PARITY.md`.

## 0.2.0

- Stable-ID blocks: text, three heading levels, bullet/numbered lists, tasks,
  quotes, callouts, code and dividers.
- Slash commands searchable in French and English, keyboard selection and block
  transformations preserving content.
- Block menu insertion/duplication/deletion/moving and document-level session undo/redo.
- Meeting, project and task-list templates.
- Page icons, three cover colors and full width.
- Page duplication, cycle-safe hierarchy moves, back/forward navigation and subpage links.
- Initial-document conversion preserves rich text and list indentation, with
  atomic Turso writes.

## 0.1.0

- First native Rust/Rune notebook: pages/subpages, search, favorites, archives,
  rich text, light/dark themes and local Turso storage.
