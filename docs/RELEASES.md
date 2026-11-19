# Folio releases and updates

Folio targets **Apple Silicon, macOS 13 or later**. A release is a complete
`Folio.app`, not a source checkout. End users need no Rust, Git, GitHub CLI or
Nexus to install an update. Data stays outside the application bundle, in
`~/Library/Application Support/Folio/` (or the `FOLIO_DB_PATH` profile).

## Open the app

For this local working copy, build with:

```sh
python3 scripts/build-app.py --development
open dist/Folio.app
```

Move `dist/Folio.app` to a writable Applications folder for everyday use, such
as `~/Applications/Folio.app`. Updates replace **the bundle currently running**.
A raw Cargo executable cannot install bundle updates. A development bundle can
update to a strictly newer published version, but is never a publishable artifact.
The builder preserves the prior local bundle as `dist/.Folio.previous.app`.

No Apple Developer account is used. The bundle has an **ad-hoc code signature**;
it is neither Developer ID signed nor notarized. On a first downloaded install,
macOS may require an explicit approval in Privacy & Security, or block launch
under its policy. Folio does not remove quarantine or disable Gatekeeper. This
limitation also applies to downloaded updates: a launch blocked by macOS causes
installation failure and rollback, rather than silently bypassing the policy.
See [Apple's guidance](https://support.apple.com/en-us/102445).

## What is versioned

| Item | Source of truth |
|---|---|
| Folio version | `Cargo.toml`, `Cargo.lock`, top entry of `CHANGELOG.md` |
| Rust toolchain | `rust-toolchain.toml` |
| Rune source and vendored patches | full SHA in `folio-update.json` and Cargo.toml; locked Git sources |
| Turso | exact dependency in `Cargo.toml` and resolved `Cargo.lock` |
| Update protocol and trust anchor | `release.json` |
| Published stable build | immutable Git tag `vVERSION`, signed GitHub Release assets |
| Running executable | `folio --release-info`, also embedded in `Contents/Resources/build-info.json` |

The current source pins Rust **1.100.0** and Rune
`f54cbcfa84d995d339c243a579218f5062c940b5`. Turso stays at **0.8.3**, with
its unused optional FTS extension disabled for compiler compatibility; the
notebook storage contract is unchanged. See [upgrade notes](RUNE_UPDATE.md).

Rune's crate version alone is insufficient: its Git revision and vendor patches
are part of each build. The build script checks all manifest pins and resolved
Cargo sources, Cargo's Rune checkout, the source revision and whether Folio has
uncommitted changes. It uses `--locked`,
remaps local source paths out of compiler diagnostics and packages the existing
brand icon. The public key is embedded in the executable; the private key is not.

Rune is a Git dependency fetched by Cargo, including the vendor overrides.
A Folio checkout or worktree can build independently of other projects and their
Rune checkouts. Keep every Rune/vendor `rev` aligned with `folio-update.json`;
packaging rejects missing, mixed, registry or path replacements.

## Weekly intake, with approval

`.github/workflows/dependency-watch.yml` runs on Mondays at 08:00 UTC and can be
started manually. It checks Rust stable, Turso's stable crate and Rune's upstream
default-branch commit, then retains a dated JSON/Markdown report and GitHub job summary.
A failed source check is an error, never “up to date”. The report links upstream
release notes/diffs and identifies the Folio areas that need review.

This workflow is **read-only intake**, not an AI assessment or an upgrade bot.
The report also retains bounded upstream release-note and commit excerpts.
Detailed per-change code impact analysis, candidate edits and human review belong to the future Nexus workflow described in `NEXUS_PROMPT.md`.
It must prepare an isolated branch, run checks and expose a test bundle before
approval. GitHub scheduling works without the local Mac being on. Local builds
and Codex sessions need a running Mac; CI builds are an alternative.

Nothing is published weekly without approval. A Git push to `main` is not a
release and does not notify installed applications.

## Signing key — one-time setup already performed locally

A new Ed25519 key was generated in `.folio-release-key/private.pk8`. This entire
directory is ignored by Git, with directory permissions `0700` and key `0600`.
**Back it up securely before the first publication.** Losing the key prevents
existing installations from trusting future updates. Never send it to a coding
agent, put it in a prompt/log, attach it to a release, or include it in an app.
Only the public key in `release.json` belongs in Git. It is not an Apple identity.

For a new installation of the release infrastructure, the offline utility is:

```sh
cargo run --locked --bin folio-release -- keygen .folio-release-key release.json
```

It refuses to overwrite an existing key or configured trust anchor. Rotation
requires a deliberate signed transition; this version does not implement rotation.

## Prepare a release

1. Review and commit the intended source changes. Update the Folio version,
   changelog and pins together, including `Cargo.lock`. Verify old profiles,
   encryption, PIN pages, backup/restore and migrations using copied test data.
2. Set up the GitHub `release` environment with required reviewers where the
   repository's plan supports them. Add `FOLIO_RELEASE_KEY_BASE64` as its secret
   through a secure local workflow. It is the Base64 encoding of the private
   PKCS#8 file. Do not paste it into a conversation. No GitHub secret or environment
   has been configured by this local implementation.
3. Start **Prepare reviewed Apple Silicon release** with the exact reviewed SHA
   and version. It checks the identity, fetches the pinned Rune source, runs
   format/tests/Clippy, builds on `macos-15` arm64, signs the manifest and creates
   a **draft** release. The temporary private key is removed even on failure.
4. Review the actual draft, version/tag/commit, notes, assets and test evidence.
   Ensure no existing tag is moved or reused; then publish the draft from GitHub
   or a future explicitly approved Nexus action. Never replace published assets.

A local equivalent for a clean, reviewed checkout is:

```sh
python3 scripts/build-app.py --release --key .folio-release-key/private.pk8
```

This creates assets only; it does not push a commit, create a tag or publish.
The workflow likewise does not publish its draft. Its signing secret is unavailable
until configured. A private repository has no anonymous public release feed:
Folio reports that a release is unavailable, and does not request/store credentials.

## Release contract for Nexus

`GET https://api.github.com/repos/0xrariux/folio/releases/latest` discovers a stable,
non-draft release with tag `vVERSION`. The app retrieves these exact assets:

- `folio-release.json` — UTF-8 metadata, at most 64 KiB.
- `folio-release.sig` — Base64 Ed25519 signature over the **exact JSON bytes**.
- `Folio-VERSION-macos-arm64.zip` — zipped, ad-hoc signed `Folio.app`.

The metadata contains `schema`, `version`, full `revision`, `target`, `bundle_id`,
`minimum_macos`, `storage_contract`, `archive`, `size`, and `sha256`. Its canonical
example is generated by `scripts/build-app.py`, not hand-edited. URLs are derived
from the embedded repository, exact tag and constrained asset name, not accepted
from remote metadata. Authentication verifies the signing key before trusting
metadata. ZIP size (maximum 512 MiB), SHA256, paths, bundle identity/version,
embedded provenance, arm64 executable and macOS signature must all pass.

A current process's version must be strictly older. Prereleases, incompatible
storage contracts and different compatibility levels are refused. Do not relax
these checks to force an update through. Increase the protocol schema only with
a planned client migration. The current contract is
`turso-0.8.3-schema4-notebook7-vault1`: a Turso/encryption/document format change requires
explicit compatibility work. Changing this value **blocks** automatic installation;
it is not itself a database migration. Old clients then need a reviewed manual
migration/reinstall path. Binary rollback cannot reverse arbitrary database writes.

Folio 0.13 adds the module registry in SQL schema 4. Upgrading a schema-3 profile
requires that reviewed manual path; old clients must not bypass contract checks.
The module compatibility gate runs alongside the frozen database corpus. Module
state is included in backup rehearsals and must survive unavailable definitions.

## Installation transaction and recovery

Folio checks for a release on startup and daily while running; demo mode never
auto-checks. Settings shows a short status and offers **Update** only when a signed
newer release is available. **Install and restart** performs these steps:

1. Wait for saves, reject existing save errors and close the current edit.
2. Save preferences and produce a full notebook backup through the existing
   storage worker, including history and attachments. An encrypted notebook gets
   an encrypted backup using its master-password-derived key. Failure cancels
   installation. The backup must also restore into an isolated database, reopen
   successfully and match the complete notebook; its verification receipt binds
   the exact bytes and source build. These backups are in `folio.backups/updates/` and are outside
   the automatic 14-backup rotation.
3. Drain and close the storage worker, then start a copied installer helper and
   quit the UI. The helper waits for the original process to exit. It rechecks
   signatures, download integrity and compatibility and takes an installation lock.
4. Stage the whole bundle beside the target, preserve the previous app as
   `.Folio.previous-UUID.app`, and replace it using filesystem renames.
5. Start the new app with the same profile. Require an acknowledgement after
   successful data/preferences loading (or a valid encrypted-vault lock screen).
   Early exit, invalid acknowledgement or a 90-second startup timeout kills the
   new process, restores the old bundle and tries to relaunch it with an error.

The helper never copies, moves or replaces the live database. An encrypted vault
is locked after restart. The health check covers startup, not all future behavior.
Do not use this updater for an unreviewed storage migration. A power loss during
the two-rename swap can require manual recovery; this is not a general crash-proof
filesystem transaction. The sibling previous bundle and the recovery journal are
kept for that purpose. Folio has no privileged helper: a read-only or translocated
installation must first be moved to a writable Applications folder.

Diagnostics are private under `folio.updates/release-*/install.log`,
`recovery.json`, `installed.json` and `failed.json`. If recovery is required, quit Folio, restore
the recorded previous bundle to its original path, and launch it. Preserve the
profile and the pre-update backup; restore data only through a reviewed backup
restore operation. Logs can include local paths, and do not belong in public Git.
Old rollback apps and pre-update backups are retained until manually removed.

## Validation

```sh
cargo fmt --package folio -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
python3 -m unittest discover -s scripts/tests
python3 scripts/build-app.py --development
/usr/bin/codesign --verify --deep --strict dist/Folio.app
```

For a native installation smoke test, provide an older built bundle and signed
assets for a newer version using the same public key:

```sh
python3 scripts/test-update-install.py --app dist/Folio.app --release /path/to/newer/release-artifacts
```

The harness copies the app, creates a fresh temporary profile, installs the release,
checks startup acknowledgement and unchanged notebook pages, and closes its own
test processes. It never updates the supplied bundle or opens the personal profile.
During implementation, an unpublished temporary `0.12.1` fixture successfully
exercised packaging/signing and installation from the `0.12.0` bundle. This fixture
is test data, not a published Folio release. Only the current local macOS version
was exercised; the declared macOS 13 deployment target still needs older-OS testing.

Use `--package folio` when formatting: `cargo fmt --all` also traverses the local
Rune dependency checkout. Before enabling public distribution, separately review
Git history for old private data/author metadata; making the current tree generic
does not rewrite historical commits.


Storage changes must also satisfy the [data safety and recovery policy](DATA_SAFETY.md).
`build-app.py --release` now runs the frozen corpus in the production release profile
before signing assets. Keep the separate Storage compatibility workflow required;
never approve a release by replacing failing historical/encrypted fixtures.
