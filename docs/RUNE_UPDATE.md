# Dependency upgrade — November 24, 2026

Folio 0.16.1 pins Rune **`3f208ba8cce7693298c5b0e2618623e4a1740434`**, the official
`dev` head verified for this update. This advances **306 commits** from
`4c4abf5d6d2b47892b3b01c519341b943f5cebcf`: **273** added changelog fragments
and one removed obsolete fragment. The main changelog did not change, so the
review uses the exact revision's fragments and dependency manifests. Some upstream
fragment filenames carry November 25 dates; the fetched revision is the pin,
rather than dates in filenames. The neighboring Rune working tree is unchanged.

## Resolved direct dependencies

| Dependency | Previous | Updated |
|---|---|---|
| Rune / Rune Kit | `4c4abf5d` | `3f208ba8` |
| Turso | 0.8.3 | 0.8.4 |
| Reqwest | 0.12.28 | 0.13.5 |
| Argon2 | 0.5.3 | 0.6.0 |
| ChaCha20Poly1305 | 0.10.1 | 0.11.0 |
| OS randomness | Rand Core 0.6.4 / OsRng | Getrandom 0.4.3 |
| Base64 | 0.22.1 | 0.23.1 |
| Anyhow | 1.0.104 | 1.0.105 |
| UUID | 1.29.2 | 1.29.3 |
| Zeroize | 1.9.1 | 1.9.2 |

Rust **1.100.0** remains the verified stable compiler. Ring, Libc, Semver, Chrono,
Serde, Serde JSON, Tokio and Tempfile were already at the latest stable versions
reported by their official registries. `cargo update` refreshes compatible
transitive versions too, including wgpu 30.0.2; Rune's patched Naga/wgpu-hal 30.0.1
remain authoritative through Cargo patches. The fixture producer is not updated.

## Integration changes

- **Rune patches:** add `naga` for WGSL override folding and
  `accesskit_atspi_common` for flat-tree set sizes alongside the six existing
  vendor patches. All eight patches and Rune crates resolve from one full Git SHA.
  Packaging explicitly rejects missing patches and mixed/overridden sources.
- **Rune behavior:** receive modal focus/entrance fixes, scoped virtual-row
  reveals, scroll routing, secure-buffer wiping, image ICC conversion, font/layout
  fixes, and Metal loss/recovery improvements. Keep existing cancellation handlers,
  secure editors, stable row keys and `orderly_exit`. New optional APIs, including
  system-tray capture, do not add features to Folio by themselves.
- **Production CLI:** Rune release builds without `tooling` now leave command-line
  flags to the application. Folio handles its installer, release-info and backup
  verification flags before launching Rune, so their behavior is retained.
- **HTTP:** replace Reqwest's removed `rustls-tls` feature with `rustls`; 0.13 uses
  the platform certificate verifier and AWS-LC provider. HTTPS-only requests,
  connection/overall timeouts, response size limits, archive hashes and independent
  Ed25519 release signatures remain in place.
- **Crypto:** replace removed `rand_core::OsRng` APIs with fallible
  `getrandom::fill`, propagate salt/nonce errors, and construct typed nonces from
  fixed arrays instead of the deprecated slice API. Keep every stored crypto
  parameter, authenticated-data string and envelope field unchanged.
- **Storage:** Turso 0.8.4 fixes UPSERT before-images, autocommit rollback and
  statement savepoints. Set the exact engine pin and update `release.json` to
  `turso-0.8.4-schema4-notebook7-vault1`. Schema **4**, documents **7**, notebook
  formats **1/2**, vault **1** and module API **1** stay unchanged. The old updater
  intentionally refuses the new engine contract: use the
  [manual upgrade/recovery procedure](DATA_SAFETY.md#turso-084-and-crypto-dependency-upgrade).

## Validation

Required gates:

```sh
cargo fmt --package folio -- --check
cargo check --locked --all-targets
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s scripts/tests -v
python3 scripts/build-app.py --development --verify-storage
```

All gates passed at the pinned revision:

- **198 Rust tests** pass across all targets; two historical fixture-producer
  helpers remain intentionally ignored.
- **10 Python tests** pass, including rejection of missing vendor patches and
  mismatched Rune sources.
- Formatting, compilation and Clippy with warnings denied pass.
- The development app bundle passes **26 production-profile storage and module
  tests**, reports Folio 0.16.1 / Rune `3f208ba8` / Turso 0.8.4, and passes strict
  code-signature verification.
- Eight Metal-validated captures cover Library and Settings in light/dark themes
  at 1680 × 1000 and 920 × 620, without validation errors. Runtime regression
  tests also resize both pages from two columns to one and back, checking available
  width, reachable controls and the fixed Settings header after scrolling.
- Five additional Metal-validated captures cover Aurum on Settings, Library and
  completed task groups, the small-window appearance controls, and restoration
  of the original Sage palette. A worker-backed test verifies that palette
  selection survives restarts and light/dark switches; the notebook roundtrip
  also preserves the selected palette.
- The final Settings design has four distinct pages with a fixed navigation bar.
  Twelve further Metal-validated captures cover all pages, all six dark palettes,
  light mode and the small-window gallery before/after scrolling, without errors.
  Runtime tests cover page isolation, full pane width, fixed navigation across
  resizes, Enter/arrow/Home navigation and all six saved palette choices. Contrast
  tests check primary/secondary text and accents at 4.5:1 on all dark surfaces.
- Five captures validate Vanille on Appearance at two window sizes, Library and
  notes, plus restoration of the original light palette. Vanille's text and
  accents meet the same contrast checks. Worker and notebook roundtrip tests
  preserve light palettes across restarts, dark/light switches and backups.

All profiles are temporary and synthetic; no personal notebook or historical
fixture is modified. Library fills the available pane with responsive columns and
cards; Settings uses separate pages and a responsive palette gallery. These changes do not alter
the document format or module permissions. The optional `dark_palette` preference
defaults to Aurum when absent and is omitted when saving that default; other
palettes are stored explicitly when selected. Palette research and official sources
are documented in [Settings](SETTINGS.md).
The optional `light_palette` preference similarly defaults to Original and is
omitted when saving that default; selecting Vanille persists it independently of
the dark palette. Existing notebook fixtures remain unchanged.

## Official sources

- [Rune comparison at the exact pins](https://github.com/anomalyco/rune/compare/4c4abf5d6d2b47892b3b01c519341b943f5cebcf...3f208ba8cce7693298c5b0e2618623e4a1740434)
- [Rune changelog fragments](https://github.com/anomalyco/rune/tree/3f208ba8cce7693298c5b0e2618623e4a1740434/changelog/unreleased)
- [Rune vendor definitions](https://github.com/anomalyco/rune/blob/3f208ba8cce7693298c5b0e2618623e4a1740434/Cargo.toml)
- [Turso 0.8.4](https://github.com/tursodatabase/turso/releases/tag/v0.8.4)
- [Reqwest 0.13.5 and 0.13 migration](https://github.com/seanmonstar/reqwest/blob/v0.13.5/CHANGELOG.md)
- [Argon2 0.6 changelog](https://github.com/RustCrypto/password-hashes/blob/argon2-v0.6.0/argon2/CHANGELOG.md)
- [ChaCha20Poly1305 0.11 changelog](https://github.com/RustCrypto/AEADs/blob/chacha20poly1305-v0.11.0/chacha20poly1305/CHANGELOG.md)
- [Base64 0.23 changelog](https://github.com/marshallpierce/rust-base64/blob/v0.23.1/RELEASE-NOTES.md)
- [Zeroize 1.9.2 changelog](https://github.com/RustCrypto/utils/blob/zeroize-v1.9.2/zeroize/CHANGELOG.md)

---

# Previous Rune upgrade — November 23, 2026

Folio pins Rune **`4c4abf5d6d2b47892b3b01c519341b943f5cebcf`**, the official
`dev` head verified on November 23. Rune has no numbered releases; applications
pin a reviewed Git revision. This advances **309 commits** from
`f54cbcfa84d995d339c243a579218f5062c940b5` and adds **235 changelog fragments**.
The main `CHANGELOG.md` is unchanged over this interval; the analysis uses the
fragments and manifests at the exact new revision.

## Changes affecting Folio

| Area | Upstream change | Folio integration |
|---|---|---|
| Dependency sources | Rune's patched `tiny-skia-path` package is renamed `rune-tiny-skia-path` to avoid vendor collisions | Update both direct Rune dependencies and all six application-level patches to the same full revision; regenerate `Cargo.lock` and retain packaging source verification, which also checks the renamed package |
| macOS bindings | Rune consolidates its Objective-C bindings on `objc2` 0.6 and AppKit/Foundation 0.3 | Resolve the new graph through Cargo; Folio has no direct Objective-C dependency to migrate |
| Dragging | Held drags receive a cancelled release when their source disappears or a modal opens; completed drags suppress clicks | Keep Folio's cancellation handling and click guards; validate block moves, nesting, selection, undo and sidebar/database drags |
| Inline links | Clicking `Mark::Link` inside an editor invokes `El::on_link`; full field hit testing is retained | Folio already routes this callback through `follow_reference`; verify navigation and ordinary editing together |
| Focus and modals | Disabled controls hand focus to a valid neighbour; modals make following siblings inert and reject disabled return targets | Exercise menus, keyboard landmarks and security/Library dialogs with the existing focus keys |
| Protected inputs | Secure fields omit text-run children, character geometry and selections from accessibility and tooling output | Retain both secure `TextEdit` state and `.secure()` fields; verify PIN/password entry and absence of secret text in the accessibility tree |
| Rich text and images | Span/index overflow fixes, linear decoration lookups, RTL caret fixes, selection autoscroll and bounded SVG/image decoding | Retain Folio's supported editor APIs; run rich-text, Unicode, attachment and historical document checks |
| Rendering | Metal device recovery, KVO hook fixes, synchronous Retina scale changes and surface configuration guards | Keep native Metal with the existing macOS 13 minimum; build and inspect synthetic demo captures |
| Virtual tables | Scoped reveals stay inside their viewport; virtual-row remounting, anchors and reused paths are corrected | Keep stable row keys and mounted-only row construction; test offscreen selection, focus and resized tables |
| Restart tooling | `--window-script` is a new real-window replay flag that must be stripped when forwarding launch arguments | Folio's installer already launches without forwarding any command-line arguments; no restart change is required |

No public API used by Folio requires a source migration. Existing handlers already
implement the relevant cancellation, secure-input, link and orderly-shutdown
contracts. New optional math, animated-icon and preview APIs are not enabled by
this dependency update. Windows, Linux and web changes are upstream improvements,
not newly qualified Folio platforms.

Rust stays **1.100.0** and Turso stays **0.8.3**. No existing registry dependency
is upgraded by this lockfile refresh; obsolete Rune-only dependencies disappear.
SQL schema **4**, documents **7**, notebook formats **1/2**, vault format **1** and
the signed-release storage contract stay unchanged. Validation uses synthetic
profiles and the frozen historical/encrypted corpus; it never opens a personal
notebook or regenerates fixtures.

## Validation

The required gates are:

```sh
cargo fmt --package folio -- --check
cargo check --locked --all-targets
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s scripts/tests -v
python3 scripts/build-app.py --development --verify-storage
```

Local results on November 23, 2026:

- **181 Rust tests passed** across all targets. Two helper tests remain
  intentionally ignored; their crash writer is exercised by its parent test.
- **9 Python tests passed**, including exact Rune/vendor provenance checks.
- Locked all-target checking, project formatting and Clippy with `-D warnings`
  passed.
- **26 production-profile tests passed** (18 historical/data-safety and 8 module
  lifecycle tests), followed by successful `dist/Folio.app` packaging and ad-hoc
  signature verification.
- Four offscreen demo captures completed with `MTL_DEBUG_LAYER=1` and no Metal
  validation errors: dark table, light table after window/column resizing, active
  nested block drag, and Overview with search at the 920 × 620 minimum size.
  The light table, drag and Overview images were visually inspected. Captures
  and tree dumps are local ignored artifacts under `artifacts/rune-update-*`.

These captures use the debug build's tooling; the bundle uses the production
release profile. They do not exercise moving a real window between displays,
forcing GPU loss, actual VoiceOver, or other operating systems. No runtime
performance improvement is claimed from upstream benchmark results.

## Sources

- [Exact Rune comparison](https://github.com/anomalyco/rune/compare/f54cbcfa84d995d339c243a579218f5062c940b5...4c4abf5d6d2b47892b3b01c519341b943f5cebcf)
- [Changelog fragments at the new pin](https://github.com/anomalyco/rune/tree/4c4abf5d6d2b47892b3b01c519341b943f5cebcf/changelog/unreleased)
- [Dependency and vendor definitions](https://github.com/anomalyco/rune/blob/4c4abf5d6d2b47892b3b01c519341b943f5cebcf/Cargo.toml)

---

# Previous Rust and Rune upgrade — November 19, 2026

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
