# Folio feature status

Current scope: Folio 0.15. This describes Folio's implemented behavior, rather
than claiming complete feature or performance parity with Notion. Earlier
iterations are recorded in the [changelog](CHANGELOG.md).

| Area | Available | Remaining scope |
|---|---|---|
| Editor | 20 block kinds, rich text, collapsible nested lists, drag nesting, task completion rollups, multiple selection and subtree operations | Block columns and advanced typography |
| Organization | Pages/subpages, favorites, archives, confirmed deletion, drag order/nesting | Shared spaces and collaboration permissions |
| Notebook overview | Built-in shortcut, note/task/list summaries, search, task filters and links to source blocks | Customizable overview widgets |
| Collections | Table, Kanban, Calendar, Dashboard, named views, combined filters and sorting | Gallery, timeline and complex nested filters |
| Tables | Per-view widths/order/visibility, bulk record edits, virtualized rows | Spreadsheet-style keyboard cell editing |
| Relations/calculations | Reverse relations, count/completion/sum/remaining rollups, numeric formulas | Full formula language and automations |
| Project pages | Embedded source views, shared records and project-relation filters | Free widget arrangement |
| Navigation | Command-K palette, Unicode search, mentions, page/block links, contents and backlinks | Indexed whole-notebook search and published external links |
| Media | Managed local images/files, image sizing and original-file export | Remote media, embedded PDF reader and shared storage |
| History/portability | 50 versions/page, checkpoints, full `.folio` backup/restore, Markdown/CSV export | Notion/Markdown/CSV import and sync |
| Appearance | Dark/light themes, persistent sidebar preferences, 105 symbols/eight colors and customizable shortcuts | Cross-device preferences |
| Security | Optional encrypted Turso vault and PIN-protected eligible simple notes | Attachment encryption and independent security audit |
| Application delivery | Apple Silicon `.app`, signed release protocol and guarded in-app bundle replacement | Developer ID/notarization; a reviewed published release is still required |
| Modules | Library, local definitions/catalogs, reviewed lifecycle, private state and recovery | Functional modules, executable runtime, remote marketplace and data-migration handlers |

The calendar, Kanban and notebook tree support cancellable dragging. Embedded
views edit original records, and relation-filtered creation links new tasks to
its project. Formulas support arithmetic and `prop("Column name")`; they do not
provide a general programming language. Widgets have a fixed layout.

Built-in statuses are To do, In progress and Done; priorities are Low, Medium and
High. Dates use `YYYY-MM-DD`. Records accept up to 12 tags of 40 characters each.
Archived records are excluded from collection views. Record search is separate
from the global page filter. Named views retain their filters and sorting.

Links use stable page identities; renaming updates labels without changing their
destination. Backlinks exclude archived sources and group references from the
same page. Missing targets remain visible. Duplicating a page leaves its children
attached to the original; duplicating a collection copies configuration, while
records stay in the source collection. Active children of an archived parent
remain accessible at the root.

Current document format is 7, with readers for versions 1–6; module storage uses
SQL schema 4 and portable notebook format 2 when nonempty. Unsupported old clients
refuse newer data rather than rewriting it with information loss. Page/document
writes and notebook restoration are transactional. See [storage safety](docs/DATA_SAFETY.md).

The notebook loads into memory. Virtualized table rows preserve offscreen focus
and selection, but do not virtualize the whole application. No evidence establishes
performance parity with Notion. Local encryption is not a security certification;
Turso's native encryption remains experimental and attachments remain plaintext.

## Future directions

- Import with correct links/media mapping.
- Indexed search and scalable notebook navigation.
- Block columns, widget arrangement, gallery and timeline.
- Richer formulas and explicit local automation capabilities.
- Functional modules using the host boundaries already prepared.
- Sync/collaboration after defining conflict resolution and permissions.

Notion reference material for readers comparing product concepts:
[databases](https://www.notion.com/help/intro-to-databases),
[properties](https://www.notion.com/help/database-properties),
[views](https://www.notion.com/help/category/database-views),
[links](https://www.notion.com/help/create-links-and-backlinks),
[export](https://www.notion.com/help/export-your-content),
[sharing](https://www.notion.com/help/sharing-and-permissions) and
[backup/history](https://www.notion.com/help/back-up-your-data).
