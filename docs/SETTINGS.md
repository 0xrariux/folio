# Settings, persistence and protection

Open **Settings** with the gear at the bottom of the sidebar.

Settings fills the available content pane with four distinct pages:
**Notebook**, **Security**, **Appearance** and **Updates**. The top bar identifies
the current page, provides direct navigation between them and returns to notes.
It stays visible while the page scrolls. Only the selected page is shown; changing
pages resets the scroll position. The last visited page is remembered for the
current session. Security requirements and installer errors open the relevant page.
Navigation also supports keyboard focus, Enter/Space activation, Left/Right to
change pages, and Home/End to jump to the first/last page.

The dark mode defaults to **Aurum**, with near-black brown surfaces, terracotta
selections, copper accents and ivory text. In **Appearance**, choose **Light** or
**Dark**, then apply a palette using its workspace preview. Light mode offers
**Original** (the previous linen/sage colors) and **Vanille** (pastel yellow).
The two modes remember their palettes independently.

Vanille uses a cream canvas (`#FFFBEE`), butter-yellow sidebar (`#F8F0D3`),
pastel-yellow selection (`#F3E2A9`), honey accent (`#855A1F`) and warm brown text
(`#403526`). Secondary text uses `#756044` and borders use `#E7D9B1`. Text and
accent contrast is checked against the three main surfaces, including selection.
Choose **Original** in Light mode to return to the previous light colors.

Dark mode offers six palettes:

| Palette | Direction | Source |
|---|---|---|
| Aurum | Burnt orange, terracotta and ivory | Folio signature palette |
| Sage | Soft greens and charcoal | Original Folio dark palette |
| Nord | Arctic slate and frost blue | [Nord palette and UI guidance](https://www.nordtheme.com/docs/colors-and-palettes/) |
| Catppuccin Mocha | Deep navy and pastel lavender | [Official palette](https://github.com/catppuccin/palette/blob/main/palette.json) |
| Rosé Pine | Midnight plum and dusty rose | [Official palette](https://rosepinetheme.com/palette/) |
| Gruvbox Material | Earthy charcoal, amber and olive | [Official color definitions](https://github.com/sainnhe/gruvbox-material/blob/master/autoload/gruvbox_material.vim) |

These are Folio UI adaptations, with text and surface roles adjusted for the app.
They offer distinct warm, cool and pastel choices rather than an objective ranking.
Automated contrast checks require at least 4.5:1 for primary/secondary text and
accent labels against each dark background, sidebar and selection surface, and
3:1 for faint labels. This check covers palette tokens, not a full accessibility audit.

Choose **Sage** (marked Original) to restore the previous dark colors exactly.
Switching display modes preserves both chosen palettes for later. The choices are included in
notebook preferences and backups. Older preferences use Aurum for dark mode and
Original for light mode when the palette fields are absent.

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

Close older Folio instances first. In **Settings → Privacy & security → Enable encryption…**,
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

Turso's native **AES-256-GCM** protects database pages and WAL, but its 0.8.4 API
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
