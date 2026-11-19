# Rune integration — November 15, 2026

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
