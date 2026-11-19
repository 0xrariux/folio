# Rust and Rune upgrade — November 19, 2026

Folio 0.13.1 pins **Rust 1.100.0** and Rune
`f54cbcfa84d995d339c243a579218f5062c940b5` (the verified `dev` head on November 19).
Rune advanced **89 commits** from `7a92d24d63d9be8480e3e475489d2a4d0ed48c8d`,
adding **84** unreleased changelog fragments; its main changelog did not change.
The sources and impact analysis were read from those exact commits.

## Changes affecting Folio

| Area | Upstream change | Folio action |
|---|---|---|
| Rust | Stable 1.100.0, including stabilized allocator APIs and a new internal Cargo build-directory layout | Pin the exact compiler; retain Cargo metadata-based target-directory lookup and verify the packaged executable's provenance |
| Rune internals | Core, editing, GPU, image, media, spellchecking and text moved to separate crates; runtime and component internals became type-erased | Keep using supported `rune`/`rune-kit` public re-exports; lock all Rune crates/vendor patches to one Git revision and validate every target |
| Vendor patches | New `sctk-adwaita` patch alongside the existing platform patches | Mirror the application-level patch so the pinned vendor dependency set is consistent; this does not add Linux distribution support |
| Menus | `El::press_on_down` opens a mouse-triggered menu before release without running the action twice | Apply it to page actions, More and shortcut customization; keep destructive actions and drag grips on their existing gesture paths |
| Rendering | Native Metal resources survive command-buffer completion; CPU/GPU first-frame and drag-alpha fixes | Keep the default Metal backend and inspect native demo rendering and drag/drop behavior |
| Editing/accessibility | Large text fields retain text ranges and visible-line/caret geometry; field alignment and scale handling fixed | Retain secure `TextEdit` inputs and named landmarks; exercise editor, PIN and accessibility regressions |
| Scrolling/popovers | Focused scroll keys, wheel chaining, overlay occlusion, exit lookups and scaled-row reveal fixed | Validate existing virtual tables, Library pagination, menus and notebook dragging |
| Application provenance | Isolated worktrees have a `.git` file and non-main branches | Resolve HEAD/index/current-branch paths with Git rather than assuming `.git/refs/heads/main` |
| Turso compatibility | Rust's new inherent `Vec::new_in` shadows Turso 0.8.3's optional FTS allocator shim | Disable unused FTS, retain `mimalloc`, encryption and exact engine version; do not patch the shared registry or migrate the notebook |

Folio starts storage on a dedicated background worker from its setup hook,
explicitly schedules polling work with `Tx::wake_after`, uses public APIs and
opts into `rune::orderly_exit` to drain writes. These contracts remain intact. New opt-in live-list/prepare-text APIs are
not enabled without a measured need. Windows/Linux/web-only changes are recorded
as upstream changes, not claimed as validated Folio platforms.

## Storage boundary

Turso remains **0.8.3**. SQL schema **4**, documents **7**, notebook backup formats
**1/2**, vault format **1** and the signed-release storage contract are unchanged.
Folio does not create FTS indexes and continues using its document search.
The frozen fixture producer remains pinned to historical Turso 0.7.2.
Do not regenerate its fixtures or use a real notebook to test this update.
See [storage safety](DATA_SAFETY.md).

## Validation

Validation uses an isolated Folio worktree and exact Cargo Git sources,
synthetic profiles and the unchanged historical/encrypted corpus. Required gates:

```sh
cargo fmt --package folio -- --check
cargo check --locked --all-targets
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s scripts/tests -v
python3 scripts/build-app.py --development --verify-storage
```

Folio now resolves Rune through Cargo Git dependencies, so the main checkout and
CI can build without moving a shared Rune repository. Packaging verifies that
every Rune/vendor source matches the full reviewed revision.

The build script must still locate `release/folio` through Cargo metadata after
Rust 1.100's internal cache-layout change. Release-profile storage and module tests
run with the same compiler/deployment flags as the application. Native captures
use `FOLIO_DEMO=1`; no personal notebook is opened.

Local results on November 19, 2026:

- **163 Rust tests passed** with the final Cargo Git dependency graph. Two helper
  tests remain intentionally ignored; the crash writer is exercised by its parent.
- **9 Python tests passed**, including manifest-form handling and rejection of
  mixed, unpinned or overridden Rune sources during packaging.
- All-target checking, project-only formatting and Clippy with `-D warnings` passed.
- **26 production-profile storage/module tests passed** (18 historical/data-safety
  and 8 module lifecycle tests), followed by successful `.app` packaging/signature verification.
- Native Metal captures of Library at minimum window size, a resized table and
  an active block drag completed with Metal API validation enabled and no validation errors.
- The real upstream monitor reports Rust 1.100.0, Rune `f54cbcfa` and Turso 0.8.3
  current. No dependency pins were changed by the monitor.

## Sources for this upgrade

- [Rust 1.100 announcement](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/)
- [Pinned Rune comparison](https://github.com/anomalyco/rune/compare/7a92d24d63d9be8480e3e475489d2a4d0ed48c8d...f54cbcfa84d995d339c243a579218f5062c940b5)
- [Rune changelog fragments at the pin](https://github.com/anomalyco/rune/tree/f54cbcfa84d995d339c243a579218f5062c940b5/changelog/unreleased)
- [Rune dependency and vendor definitions](https://github.com/anomalyco/rune/blob/f54cbcfa84d995d339c243a579218f5062c940b5/Cargo.toml)

---

## Previous integration — November 15, 2026

This historical report predates Settings, encryption and document format 7.
See the current [settings guide](SETTINGS.md), [release guide](RELEASES.md) and
[changelog](../CHANGELOG.md) for later changes.

Rune advanced with `git pull --ff-only` from
`70eff81165e9854834a75f17753edb6c96ab2ab7` to
`7a92d24d63d9be8480e3e475489d2a4d0ed48c8d`: 627 commits including merges.
Its main `CHANGELOG.md` did not change in that interval; details came from
343 added fragments and one modified fragment in `changelog/unreleased/`.

## Applied adaptations

| Area | Impact | Adaptation |
|---|---|---|
| Rune vendor dependencies | Initial locked/offline check failed because published `parley_engine` lacks `line-dictionaries`; dependency patches do not propagate to an application | Application patches to sibling Rune vendors for `parley_engine`, `winit`, `wgpu-hal`, `accesskit_windows` and `accesskit_macos` |
| Reproducibility | Guides still selected an older Rune | Updated the pinned revision and lockfile without a general upgrade of published dependencies |
| Shutdown | Rune's new fast exit bypasses `Worker::drop`, FIFO write draining and final backup | Call `rune::orderly_exit()` before launch |
| Release captures | `--snap`, `--script`, `--tree` and related tools now need `rune/tooling` in release | Add application `tooling` feature; debug captures remain available |

The patched `tiny-skia-path` is already Rune's direct dependency, requiring no
extra application patch. Orderly exit retains teardown cost; fast exit would need
an explicit storage shutdown protocol before terminating the process.

Changes to `TextEdit::preedit`, combobox state and component zoom options required
no adaptation in Folio's current uses. All macOS project targets compiled.

## Upstream effects

- Text metrics at 1/64 pixel, origin/icon fixes, Metal colors and macOS smoothing
  can slightly change title, cell, menu and icon placement. Historical screenshots
  remain evidence of the previous renderer.
- Chromium-like input undo, selection, bidirectional text, focus fallback and
  nested-scroll improvements affect editing, toggles and menus.
- AccessKit patches expose native headings and fix process-abort risks in some
  macOS queries. Folio now supplies named landmarks and headings.
- Windows gains CPU-first rendering before GPU handoff and startup/rendering
  improvements. No measured Folio-specific Windows gain is claimed.

## Follow-up work in Folio

1. **Virtualized table rows.** `src/ui/table.rs` uses `virtual_rows`, stable
   `table-row-{id}` keys, 44 px cells and 1 px separators. Headers stay fixed and
   column widths persist. Filtering, sorting and position resolution happen once
   per view build; only mounted row cells are built. Actions capture record IDs.
   Select-all includes unmounted filtered records. Tests cover offscreen reveal,
   keyboard focus, sorting and filtered selection across 1,000 records.
2. **Shared Unicode search.** Pages, palette commands/mentions/blocks/collections/
   views/relations, link pickers and record/dashboard filters use
   `rune_kit::search::Fold::for_language("fr")`. `reunion` finds `Réunion`, while
   `réunion` excludes `Reunion`; Unicode case and width folding also apply.
   Palette highlights preserve original UTF-8 title ranges. Name uniqueness and
   title sorting remain separate from search matching.
3. **Accessibility structure.** Named navigation/main landmarks, level-1 page
   titles and level-2–4 block headings include editable collapsible headings.
   Dialogs have named titles. F6/Shift-F6 and continued editing are runtime-tested.
   `Tx::announce` reports saves, exports and asynchronous failures. Errors take
   priority; stale revision acknowledgements cannot announce a false saved state.
4. **Repeatable measurements before caching.** `measure_large_views` measures
   initial render and theme changes without adding `memo`. Isolated data, a
   1200 × 820 window and a collapsed tree were used in a separate debug run:

   | View | First render | Theme update | Mounted table rows |
   |---|---:|---:|---:|
   | 1,000 records | 244 ms | 13 ms | 19 |
   | 10,000 records | 405 ms | 195 ms | 19 |
   | 500 editable blocks | 211 ms | 44 ms | — |

   These include the runtime, sidebar, search and sorting, not just table cells
   or GPU time. Build profile and font caches affect timings; no before/after
   performance claim is made. Sidebar ancestor traversal and editable-block
   virtualization remain future work with explicit offscreen selection/drag targets.

   ```sh
   cargo test --bin folio --locked --offline measure_large_views -- --nocapture --test-threads=1
   ```

5. **macOS rendering inspected.** Metal captures covered a custom dark table,
   a light table after window/column resizing, Unicode search and block dragging.
   `rune-table-resize.json` and `rune-unicode-search.json` reproduce the scenarios.
   Existing tests also cover editing, selection, folding and grouped moves with
   scrolling. Actual VoiceOver usage remains to be qualified.

Optional `pretty_wrap` does not change all paragraphs automatically. `Cx::zoom`
is available for future accessibility settings; it is separate from window or
column resizing.

## Historical validation

At this integration step, formatting, all-target checks, Clippy and release/tooling
checks passed, with 101 passing tests and none ignored. Four Metal captures used
fictional demo content only. Metal required running outside the restricted
sandbox. Rune and vendor warnings remained visible, without local suppression.

This Rune update itself changed neither Turso migrations nor document format;
later Folio iterations did. Windows/Linux execution, actual VoiceOver and native
shutdown on real notes were not qualified by these temporary-profile tests.

## Pinned Rune sources

- [Cargo patches](https://github.com/anomalyco/rune/blob/7a92d24d63d9be8480e3e475489d2a4d0ed48c8d/Cargo.toml)
- [Fast exit](https://github.com/anomalyco/rune/blob/7a92d24d63d9be8480e3e475489d2a4d0ed48c8d/changelog/unreleased/2026-11-13-fast-exit-default.md)
- [Release tooling](https://github.com/anomalyco/rune/blob/7a92d24d63d9be8480e3e475489d2a4d0ed48c8d/changelog/unreleased/2026-11-13-tooling-feature.md)
- [Unicode search](https://github.com/anomalyco/rune/blob/7a92d24d63d9be8480e3e475489d2a4d0ed48c8d/changelog/unreleased/2026-11-12-search-fold.md)
- [Focus/F6](https://github.com/anomalyco/rune/blob/7a92d24d63d9be8480e3e475489d2a4d0ed48c8d/changelog/unreleased/2026-11-12-a11y-focus-fallback-f6.md)
- [macOS headings](https://github.com/anomalyco/rune/blob/7a92d24d63d9be8480e3e475489d2a4d0ed48c8d/changelog/unreleased/2026-11-15-macos-headings.md)
- [Virtual rows](https://github.com/anomalyco/rune/blob/7a92d24d63d9be8480e3e475489d2a4d0ed48c8d/changelog/unreleased/2026-11-15-scroll-key-virtual.md)
