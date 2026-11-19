# Settings, persistence and protection

Open **Settings** with the gear at the bottom of the sidebar.

## Keep notes across restarts

Normal mode saves edits to a local Turso notebook on a dedicated worker. Orderly
exit waits for queued writes and backs up a changed session. Resolve save errors
before relying on an edit being durable.

On macOS the default folder is `~/Library/Application Support/Folio/`, independent
of the executable and Git checkout. `FOLIO_DB_PATH` selects another profile.
`FOLIO_DEMO=1` uses fictional in-memory examples that reset at launch. To run the
normal development notebook explicitly:

```sh
env -u FOLIO_DEMO cargo run --locked
```

Templates in **More → Templates** create ordinary pages only when requested.
Archiving and deleting them is persistent. Demo content never populates a real
notebook; the welcome page is created only during initialization. An intentionally
empty notebook stays empty.

To delete a page, use its sidebar or page **⋯ → Delete page… → Delete permanently**.
Confirmation also deletes local page history; subpages move to its parent and
remain intact. Previous backups remain restorable. Protected pages need unlocking
before deletion. Archiving keeps the page and offers restoration later.

## Customize navigation

Drag a shortcut above or below another; the insertion line shows its new position.
Escape cancels. **Shortcuts → ⋯** also provides move up/down, hide, restore and
reset. Hidden shortcuts appear in the menu even if every shortcut is hidden.
Order and visibility persist in notebook preferences and backups, without deleting
pages. Notebook page dragging also highlights the dragged page and destination.

## Updates

Settings shows the compiled version and checks the latest published stable
GitHub release with signed metadata. It does not compare against `main`.
Local development builds are identified as such.

**Update** downloads the Apple Silicon application. **Install and restart** waits
for writes, creates and rehearses a complete backup, then delegates bundle
replacement to a separate installer. Encrypted notebooks get encrypted backups.
The previous bundle is retained and restored if the new app does not acknowledge
startup. Notes stay in their data folder; encrypted notebooks restart locked.

Installed users do not need Git, Rust or GitHub CLI. A raw Cargo executable cannot
receive bundle updates; use `Folio.app`. A repository push alone does not publish
an update. Without a published release the check reports none available. An
incompatible storage contract blocks automatic installation and requires a
reviewed manual migration; an old binary cannot reverse a database migration.

See [builds, signatures, publication and recovery](RELEASES.md). No Apple Developer
account is used; downloads are ad-hoc signed and not notarized.

## Encrypt the notebook

Close older Folio instances first. In **Settings → Security → Enable encryption…**,
set and confirm a master password of at least 12 characters. Use a long unique
password and keep it in a password manager. Encryption starts only after this action.

The worker copies the complete notebook, including modules, into an encrypted
database, reopens and compares it, and retains a verified encrypted recovery
backup before removing the plaintext database and journals. Existing automatic
portable backups are converted. Incomplete cleanup is reported and retried on
unlock. A plaintext database that reappears beside a vault is preserved and
reported to avoid erasing notes written by an older version.

The vault uses `folio.encrypted.db` and `folio.security.json` beside the logical
`folio.db` profile. Configuration contains the format, salt and an encrypted
verifier, never the password or derived key. Preserve database, configuration
and any WAL together. An incomplete vault fails instead of becoming an empty
notebook. Existing raw plaintext recovery snapshots remain plaintext and are
reported; see [storage safety](DATA_SAFETY.md) before managing those copies.

Turso's native **AES-256-GCM** protects database pages and WAL, but its 0.8.3 API
is marked **experimental**. Folio has not received an independent security audit.
Keep a verified encrypted backup on separate storage.

The master password is required before pages load at startup. Locking the
notebook waits for writes, backs up changes, removes session keys and clears
page and Library state. Another current Folio instance cannot open the same
profile concurrently.

## Protect a note with a PIN

After enabling notebook encryption, open the menu of an eligible simple note
without attachments, a collection, dashboard or children, then choose PIN
protection. Set and confirm at least six digits. Longer PINs better resist an
offline brute-force attempt against copied files.

PIN protection encrypts title, content, properties and historical versions using
an authenticated **XChaCha20-Poly1305** envelope, a fresh nonce per write and a
bound page identity. Keys derive with **Argon2id**, a random 16-byte salt, 64 MiB
of memory, three passes and parallelism one. The master password uses the same
derivation with a separate salt.

A locked page exposes only a generic title and encrypted envelope in UI state.
Its text is excluded from search, mentions and exports. It relocks when leaving
the note, entering Settings or Library, restarting, or choosing lock immediately.
Five wrong attempts in a session impose a 30-second wait; this UI limit does not
replace resistance to attacks on copied files.

Rune secure inputs mask text and exclude values from accessibility output.
Input buffers are cleared; Folio-managed keys and transmitted secrets are
zeroized when released. Existing automatic backups have prior open versions of
the protected note sealed too; cleanup errors remain visible. History rewriting
is transactional, followed by compaction and WAL checkpointing.

## Scope and recovery

- Attachments remain separate plaintext files. PIN protection refuses notes with
  attachments, including in their history. System disk encryption can cover them.
- Explicit Markdown/CSV exports are plaintext. The protected-note export flow
  explains this before exporting an unlocked page.
- Manual copies, older exported backups, text copied into other notes and system
  snapshots are not retroactively erased. Deletion is not forensic erasure.
- Protection does not defend against a compromised operating system, screenshots
  or memory access while content is open.
- There is no password/PIN recovery, reset that decrypts content, or universal code.
  Keep credentials separately from the notebook files.

New full `.folio` backups are encrypted while the vault is active. They include
the salt needed for the original password. On another Mac, enable the destination
notebook vault first, then restore and enter the backup password when requested.
Protected notes retain their own PINs. Modules are included in these backups.

Document format is **7**, SQL schema **4** and portable notebook format **2** when
module state is present. Older unsupported clients must stay away from these files.
The tests use only temporary profiles and frozen synthetic fixtures, including
wrong passwords, tampering, transplanted envelopes, history, backup/restore,
restart persistence and worker-driven UI locking. See [data safety](DATA_SAFETY.md)
and [module lifecycle](MODULES.md) for validation commands and remaining limits.
