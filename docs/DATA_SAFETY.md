# Storage safety and compatibility

Folio keeps one local Turso profile, independently of the application bundle.
A failed compatibility check stops opening/updating/restoring the notebook;
Folio does not silently replace damaged data with an empty notebook or rebuild
missing block documents from their HTML projections. All automated checks use
synthetic fixtures and temporary profiles, never the user's notebook.

## Checks before touching an existing database

Opening a database can itself change a SQLite journal header, even before an
application query writes anything. Folio therefore preserves the database and
its WAL **before handing either file to Turso**, then validates a private copy.
Only a successful copy check permits opening the live profile.

Checks cover:

- SQLite/Turso page and index integrity with `PRAGMA integrity_check`.
- Folio application ID `FOLI` and supported schema versions (current: `4`).
- A storage contract bound to the resolved Cargo.lock Turso version, SQL schema,
  document version and encryption format; a stale contract fails the build checks.
- Exact table/column shapes and complete page/block relationships.
- Every current document, history entry, parent hierarchy and saved preference.
- Supported document versions `1..=7`, known fields/enums, valid block identities.
- Rejection of missing documents, orphaned history, unknown schemas or truncated files.

Unknown fields and unknown future shortcut names fail explicitly. Ignoring them
would let an older client drop information at its next save. Existing old fields
with documented defaults remain readable. Module manifests and private JSON state
are an explicit opaque extension boundary: unavailable/future definitions remain
preserved and inactive instead of blocking the notebook. The registry envelope
still validates strictly. See [module compatibility](MODULES.md).
Missing database files accompanied by
backups, assets or a WAL also fail; they do not create a replacement welcome page.

Schema creation, legacy HTML-to-block conversion and version markers commit in
one transaction. Existing profiles get a raw recovery snapshot first. Valid page
shape is checked before a save or checkpoint; page/content/history writes and
notebook restoration remain transactional. `synchronous=FULL` is explicitly set
and verified, and orderly worker shutdown checkpoints the WAL. A backend panic
reports a storage failure rather than leaving the app loading indefinitely.

These protections are exercised against the pinned Turso implementation, not just
assumed from SQLite behavior. Primary reference: [Turso PRAGMA documentation](https://github.com/tursodatabase/turso/blob/main/docs/sql-reference/pragmas.mdx).
An engine integrity result alone cannot prove that all expected logical records
are present, which is why the separate document/history and fixture checks exist.

## A pre-update backup must actually restore

Before an update, Folio saves pending edits/preferences and checks the live store.
It captures the complete notebook, including history and referenced attachment
bytes, module definitions, grants, catalogs and private data, writes the backup,
reads it back and compares the **whole** notebook.
Encrypted notebooks use their existing master-key encryption for this backup.

It then restores that backup into a private temporary Turso database, verifies
integrity, compares all contents, closes it, reopens it and compares everything
again. The rehearsal database is encrypted when the notebook is encrypted.
A backup changing during this verification is rejected.

Only success creates a `*.verification.json` receipt tied to the exact backup
size/SHA256, source build, schema and storage contract. The updater checks the
receipt and file before handoff and again immediately before bundle replacement.
A missing, stale or corrupted receipt/backup cancels installation. No plaintext
password or encryption key is written into the receipt or installer plan.

Manual restore rehearses the incoming notebook first, then creates and verifies
a pinned recovery backup of the notebook being replaced before the SQL transaction.
Enabling database encryption likewise compares the complete migrated notebook,
including history/preferences/files, after reopening the encrypted database, and
retains a verified encrypted recovery backup before removing the plain database.

## What is retained

| Location beside the profile | Contents and retention |
|---|---|
| `folio.backups/` | Last 14 successful automatic portable backups; each is read back before old backups rotate. |
| `folio.backups/updates/` | Verified pre-update portable backups and receipts; retained until manually removed. |
| `folio.backups/recovery/` | Verified pre-restore, encrypted conversion and module-data deletion backups; retained until manually removed. |
| `folio.recovery/before-open-*/` | Raw database/WAL/config snapshots taken before opening. Identical snapshots are reused. Three successfully checked routine snapshots rotate; migration, WAL recovery and failed-check snapshots stay retained. |
| `folio.encrypted.recovery/` | Corresponding raw encrypted-database snapshots. Ciphertext stays encrypted; vault configuration is copied with it. |

Snapshot directories are private (`0700`). Database, WAL and the associated vault
configuration must be recovered together. Do not copy only a live `.db` file and
assume it includes the last committed writes: those can still be in `-wal`.
Do not modify the sole copy of a damaged profile while experimenting with recovery.
A corrupt retained snapshot stops a reuse attempt rather than overwriting it.

**Older raw snapshots of a previously plaintext database stay plaintext.** Enabling
encryption preserves those recovery copies and reports their presence; it does
not silently delete them. Move them to secure encrypted storage before removing
them after verifying a usable encrypted recovery backup. Nested portable backups
in `updates/` and `recovery/` are encrypted during vault conversion and have their
page histories sealed when a page receives PIN protection. A receipt for an older
backup rewritten during protection must be reverified before any update use.

Do not delete `backups/` or `recovery/` as ordinary build output. A Turso upgrade,
document-format change or a different storage contract requires an explicit
migration plan; replacing an executable cannot roll back a changed database.

## Frozen compatibility corpus

`tests/fixtures/manifest.json` fixes the byte fingerprints of the synthetic corpus.
It includes:

- A SQLite-created v1 database with nested/rich/Unicode/archived content.
- A real file written by **Turso 0.7.2**, compared with a complete fixed v7 notebook.
- A real **Turso 0.8.3 encrypted database**, its vault configuration, a pre-existing
  encrypted portable backup and PIN-protected page, using public test credentials.
- All 20 block kinds, two binary/image attachments, collections, every custom
  column type, saved views/layouts, relations, formulas, dashboards, hierarchy,
  favorites/archives, history and preferences.

Normal tests never regenerate the corpus. They must keep reading the old bytes,
not merely round-trip data written by the new implementation. See
[`tests/fixtures/README.md`](../tests/fixtures/README.md) for producer provenance.

The suite additionally tests document versions 1 through 7, unknown future fields,
corrupt/truncated files, lost block rows, actual SQL constraint failures after
the first and second document writes, backup corruption, snapshot retention and a real process kill with committed writes still in the WAL. The
parent test kills only its own temporary writer, then checks every committed note,
history entry and preference after recovery.

```sh
cargo test --locked --test data_safety --test modules
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
python3 scripts/build-app.py --development --verify-storage
```

`--verify-storage` runs the corpus in the production **release** profile using the
same deployment target and compiler flags as the bundle. `build-app.py --release`
requires these checks automatically before preparing a signed publication.
The separate **Storage compatibility** GitHub workflow runs on PRs and main pushes;
make its status a required branch-protection check when configuring GitHub.

For a copied **plaintext** portable backup, an offline check is available:

```sh
/path/to/Folio.app/Contents/MacOS/folio --verify-backup /path/to/copied-backup.folio
```

This creates a verification receipt beside the supplied copy and opens only
isolated rehearsal databases. It never opens the normal notebook. It fails on an
encrypted backup without its key; use the app's password-aware restore path on an
intact/new profile for encrypted backups. For a damaged profile, preserve its raw
files and open a fresh separate profile before using Restore. Do not overwrite
the damaged original as a first recovery attempt.

## Rust 1.100 compatibility

Folio 0.13.1 retains Turso **0.8.3**, SQL schema **4**, document format **7**,
notebook formats **1/2** and vault format **1**. It explicitly enables Turso's
`mimalloc` feature instead of the default feature set. The optional `fts` extension
is unused by Folio and its 0.8.3 allocator shim collides with Rust 1.100's stabilized
`Vec::new_in`. Disabling it avoids patching or upgrading the storage engine.

Folio searches its in-memory documents, not SQL FTS indexes. Its existing strict
schema validation already rejects additional virtual/index tables outside the
supported notebook schema. Existing supported notebooks therefore have no FTS
index to migrate. Encryption features remain enabled through the Turso SDK.
The production historical/encrypted corpus, full restoration rehearsals and
module-state tests still gate the build under the new compiler.

## Limits and release discipline

This is a tested recovery strategy, not a guarantee against every hardware fault,
engine bug or power failure. All local copies share the Mac's failure risks: keep
an independent, verified backup on another storage device. A binary rollback is
not a substitute for a database restore. Never publish when historical, encrypted
or crash-recovery tests fail, and never remove a failing old fixture to unblock a
release. Nexus must preserve these gates and show their evidence for approval.


The native installer smoke test also has a refusal case:

```sh
python3 scripts/test-update-install.py --app dist/Folio.app --release /path/to/newer/assets --damage-backup
```

It corrupts only its temporary pre-update backup, checks that installation is
rejected before replacement and that the original bundle remains installed,
then closes the exact test process recorded by the installer. Both this case and
the successful signed-update case were exercised locally without publication.
