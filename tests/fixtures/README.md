# Frozen, synthetic storage fixtures

These files contain **fictional test data only**. No personal profile, token,
release signing key or user password is included. Credentials below are public
fixture values and must never be used for a real notebook.

- `legacy-v1.sqlite.fixture`: written with Python's SQLite **3.54.1**, using the
  initial Folio schema, two fixed parent/child rows and rich Unicode HTML.
- `notebook-v7.json`: independently specified, fixed expected logical notebook.
  Includes all block kinds, binary/image bytes, hierarchy, record properties,
  relations/calculations, saved views, dashboard, history and preferences.
- `turso-0.7.2-v7.sqlite.fixture`: raw database written by the standalone
  `producer/` tool using exactly **Turso 0.7.2**, with the logical v7 data above.
  The producer inserts the payload as opaque JSON; this is storage-engine
  compatibility evidence, not a claim that an older Folio release had every UI feature.
- `turso-0.8.3-encrypted.db.fixture`: real native encrypted Turso **0.8.3** database.
- `vault-v1.security.fixture`: that synthetic vault's KDF/verifier configuration.
- `encrypted-notebook-v1.folio.fixture`: frozen encrypted portable backup, including
  a separately PIN-protected page and history.
- `pin-page-clear.json`: expected plaintext of that one synthetic PIN page.
- `manifest.json`: reviewed size/SHA256 of every fixed input.

Encrypted fixture master password: `Folio fixture master password 2026`.
PIN fixture: `58391726`. They intentionally protect public test content only.

The 0.7.2 producer is an independent workspace with its own lockfile. It is **not**
a managed application or an upgrade target for Nexus. Never auto-bump its dependency.
The encrypted producer is an ignored integration-test helper requiring an explicit
output directory; normal CI does not invoke it. The other ignored test helper is
started only by its crash-recovery parent in a fresh temporary profile.

Do not regenerate/overwrite old fixtures or change their expected hashes when an
upgrade fails. Fix compatibility or write an explicit migration. If a new format
is introduced, add another corpus entry, document its writer and keep prior bytes.
Generation helpers are for adding reviewed fixtures, not for making tests pass.
