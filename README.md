# Folio

Notes, tasks, and project tracking in a native workspace inspired by Notion.
Built with **Rust**, **Rune**, and **Turso**. Local data, no account required.

## Features

- Pages, blocks, collapsible lists, tasks, and drag and drop.
- 35 page icons with eight colors, adapted to light and dark themes.
- Sidebar shortcuts you can drag to reorder, hide, and restore.
- Collections with table, Kanban, calendar, and dashboard views.
- Relations, rollups, formulas, and database views embedded in notes.
- Search, internal links, mentions, and file attachments.
- Page history, notebook backups, and Markdown/CSV exports.
- Settings, signed GitHub releases, and in-app updates.
- Optional database encryption and PIN protection for simple notes.
- Archives and permanent deletion with confirmation.
- Library with a module lifecycle and private state; no functional modules bundled yet.

Dark theme by default. Sync and collaboration are not available.
The interface, templates, demo content, guides, and changelog are in English.

## Installation and everyday use

Folio supports Apple Silicon Macs running macOS 13 or later. Open `Folio.app`
from a reviewed release and keep it in a writable Applications folder. No Rust,
Git or Nexus is needed to run the app or install a published update.

This project does not use an Apple Developer account: downloaded applications
are not notarized and macOS may require manual approval. No public release is
provided until the reviewed release workflow has been completed.

For a local application build, with the pinned Rune checkout available:

```sh
python3 scripts/build-app.py --development
open dist/Folio.app
```

Your notebook lives in `~/Library/Application Support/Folio/`, independently of
the executable. Set `FOLIO_DB_PATH` to use a different profile. Templates are
created on request; deleting existing pages is persistent.

The settings gear provides update checks, backups and security. **Update**
downloads a signed release; **Install and restart** backs up the notebook and
replaces the complete application. Encryption is enabled explicitly with a master
password. Simple eligible notes can receive a separate PIN. Attachments remain
unencrypted; Turso's native encryption API is experimental.
See the [settings guide](docs/SETTINGS.md) and [release/update guide](docs/RELEASES.md).

Open **Library** to import and review a module definition or local catalog, then
enable or disable it. Uninstall keeps its data; **Retained data** allows
reinstallation or confirmed deletion with a verified recovery backup.
This first version prepares the host and executes no external code.
See the [module format and lifecycle](docs/MODULES.md).

## Development

Rust is pinned in `rust-toolchain.toml`, Turso in Cargo.toml/Cargo.lock, and Rune
by full Git SHA in `folio-update.json`. The current source layout is:

```text
workspace/
  rune/                  # clean checkout at the pinned SHA
  folio/                 # this repository
```

Clone Rune from `https://github.com/anomalyco/rune.git`, check out the revision in
`folio-update.json`, then clone Folio into that layout. The Cargo path dependencies
and vendor patches resolve `../rune` from the Folio root.

```sh
cargo run --locked
cargo test --locked --all-targets
cargo fmt --package folio -- --check
```

A raw Cargo executable supports development and persistence; bundle installation
uses `Folio.app`. See [Rune update notes](docs/RUNE_UPDATE.md),
[release infrastructure](docs/RELEASES.md), and the separate
[Nexus implementation prompt](docs/NEXUS_PROMPT.md).

## Demo

Explore fictional sample content without opening your notebook:

```sh
FOLIO_DEMO=1 cargo run --locked -- --scene workspace
```

Demo changes stay in memory and reset on restart. In normal mode, deleting
the welcome page or a page created from a template is persistent.
Changing the bundled templates does not rewrite existing notes.

For screenshots and scripts with a release binary, enable tooling:

```sh
FOLIO_DEMO=1 cargo run --release --locked --features tooling -- --snap /tmp/folio.png
```

## Documentation

[Storage safety, compatibility and recovery](docs/DATA_SAFETY.md) ·

[User and development guide](docs/GUIDE.md) ·
[Settings, updates, and security](docs/SETTINGS.md) ·
[Changelog](CHANGELOG.md) · [Feature status](NOTION-PARITY.md) ·
[Logo and Blender sources](assets/brand/README.md)

Licensed under [MIT](LICENSE).
