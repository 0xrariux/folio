# Modules and Library

Folio 0.13 prepares a versioned module host. **No functional modules are bundled.**
The only supported runtime, `manifest`, loads definitions and manages their
lifecycle and private state. It does not execute code or add module views.
Native libraries, Wasm, scripts, shell commands, network access, repository
automation and AI integrations are not implemented in this first version.

## Using Library

Open **Library** in the sidebar:

- **Discover** shows the built-in catalog and imported catalogs. The built-in
  catalog is deliberately empty. **Manage catalogs** imports a local JSON catalog
  or removes one; removing a catalog never uninstalls its modules.
- **Add module…** imports a local JSON definition. Review its author, version and
  requested permissions before installing. Installation starts disabled.
- **Installed** enables, disables and uninstalls definitions. An update requires
  another explicit review and starts disabled again.
- **Retained data** reinstalls an uninstalled definition without needing the
  original file, or deletes its saved definition and data after confirmation.

Uninstalling keeps private state. Deleting retained data first creates a complete,
verified recovery backup in the profile's `backups/recovery/` directory. Historical
and recovery backups keep their previous contents; deletion does not erase those
copies. Reinstalling or updating preserves data, and cannot silently downgrade a
module or guess a data migration. Library mutations are unavailable in demo mode
and while the notebook is locked.

Catalogs are imported snapshots, not remote feeds. This version has no online
marketplace, automatic module download or publisher authentication. The saved
SHA256 fingerprint identifies the exact reviewed JSON definition; it is not a
publisher signature. Module names and authors are self-declared metadata.

## Definition format, API 1

This is a format illustration only, not an included module implementation:

```json
{
  "format": 1,
  "id": "org.example.extension",
  "name": "Example extension",
  "version": "1.0.0",
  "author": "Example author",
  "description": "A module definition, without executable code.",
  "folio": ">=0.13.0, <0.14.0",
  "api": 1,
  "runtime": "manifest",
  "data_version": 1,
  "permissions": ["storage"]
}
```

IDs use lowercase reverse-domain names. Versions and Folio requirements use
SemVer. `description` and `permissions` can be omitted. Other fields are required;
unknown definition fields are rejected during installation. Only `storage` is
supported: access to that module's own opaque JSON state, never direct SQL or
other modules' state. A definition with no storage permission can be installed
but cannot obtain a storage session. Definitions are limited to 32 KiB.

A catalog has the following envelope; each `modules` item is a complete definition:

```json
{
  "format": 1,
  "id": "org.example.catalog",
  "name": "Example catalog",
  "modules": []
}
```

Catalogs are limited to 1 MiB and 512 distinct module IDs. The built-in catalog ID,
`org.folio.library`, is reserved. Importing the same catalog ID replaces its
discovery list without changing installed modules. Unsupported definitions may
be displayed in a catalog, but cannot be installed or activated.

## Lifecycle and host storage

`src/modules.rs` implements the lifecycle independently of the UI. Each action
validates a candidate registry before replacing the previous one. SQL changes
commit through the storage worker in one transaction.

| Action | Installed | Enabled | Private data |
|---|---|---|---|
| Install | Yes | No | Created or preserved |
| Enable | Yes | Yes, when compatible | Preserved |
| Disable | Yes | No | Preserved |
| Review and install a newer definition | Yes | No | Preserved |
| Uninstall | No | No | Preserved |
| Reinstall | Yes | No | Preserved |
| Delete retained data | Record removed | No | Removed from current profile after backup |

The host checks API/runtime support, Folio version, grants and data version
before activation. Reinstalling the same installed definition is idempotent.
Different bytes under an existing version, older versions and a changed
`data_version` are rejected. A future data migration must have a separately
implemented and tested handler before this restriction can be relaxed.

The core `Store::module_session`, `read_module_data` and `write_module_data`
methods are the storage boundary prepared for future host adapters. Sessions are
opaque, non-serializable capabilities bound to a profile instance, installation
identity, definition, permission grant and activation generation. Disable, uninstall, replacement,
restore, vault lock and reopening invalidate previous sessions. The registry
isolates state by module ID, limits data to 8 MiB per module and limits the full
registry to 64 MiB. No external module receives these Rust objects in this phase.
Adapters must keep future state writes on the serialized storage worker and mark
successful writes dirty so automatic backups include them.

The registry, grants, imported catalogs and module data live in the notebook's
Turso preferences, not the application bundle. They therefore use the existing
master-password encryption, verified backups, restore and pre-update protection.
Module-only lifecycle changes also trigger automatic backups on lock and exit.
Entering Library relocks an open PIN-protected page. Locking the notebook clears
Library state from the UI until a successful unlock reloads it.

## Compatibility and future work

SQL schema **4** registers module state as a supported preference. Existing
schema-3 profiles migrate transactionally after the normal raw recovery snapshot
and preflight. The document format stays at **7**. An older client refuses schema
4 rather than dropping module state. The release storage contract changes to
`turso-0.8.3-schema4-notebook7-vault1`; schema-3 clients need a reviewed manual
upgrade rather than bypassing the updater's contract check.

Empty registries keep the existing portable notebook format **1**. Notebooks
containing module state use format **2**, carrying the full registry. Restore,
encryption conversion and backup rehearsal compare that state too. An unavailable
or future module's manifest and data remain opaque and preserved, with activation
denied until supported; this does not prevent reading other notebook content.
Malformed registry envelopes still fail closed.

Future work can add view contributions, a sandboxed runtime, signed distribution,
permission-specific host APIs and transactional module-data migrations. Those
features must be designed before importing any executable package. A development
project module and Codex integration are deliberately outside this foundation.

Validation:

```sh
cargo test --locked --test modules
cargo test --locked --bin folio module_library_tests
cargo test --locked --all-targets
python3 scripts/build-app.py --development --verify-storage
```

The release build gate runs both historical storage compatibility and module
lifecycle tests in the production release profile.
