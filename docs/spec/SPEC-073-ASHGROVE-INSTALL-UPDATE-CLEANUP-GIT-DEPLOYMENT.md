# SPEC-073: Ashgrove Install, Update, Cleanup, and Git Deployment

**Status:** Draft
**Date:** 2026-05-28
**Amends:** [SPEC-005](SPEC-005-CLI.md), [SPEC-009](SPEC-009-MODULES.md), [SPEC-012](SPEC-012-IMPORTS.md), [SPEC-038](SPEC-038-LANGUAGE-SERVER.md), [SPEC-070](SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
**Builds on:** [SPEC-069](SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md), [SPEC-071](SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md)
**Plan:** [PLAN-122](../plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md)
**Implementation Tasks:** [TASK-964](../plan/tasks/TASK-964-ashgrove-install-policy-packet.md) through [TASK-974](../plan/tasks/TASK-974-ashgrove-closeout-acceptance.md)

## 1. Summary

Ash alpha must be installable, updatable, removable, and deployable from git-pinned Ash projects without relying on an external package registry. This spec defines `ashgrove <command>` as the user-local toolchain and deployment manager for the first package-management slice.

`ashgrove` owns:

1. Installing Ash toolchains from either source checkouts/source archives or binary tarballs.
2. Installing a coherent versioned Ash bundle containing `ash`, `ashgrove`, the standard library, standard tooling, runtime support metadata, and install metadata.
3. Updating installed Ash by installing a new immutable toolchain and switching selectors only when requested.
4. Cleaning caches and removing unused or explicitly selected toolchains.
5. Fetching git-based Ash dependencies pinned by tag or revision and resolving them into exact commit hashes in a lockfile.
6. Using XDG-compatible user-local paths for alpha; global/system installs are deferred.

The daemon surface remains the existing `ash daemon ...` command from [SPEC-070](SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md). A separate `ashd` binary is not required by this spec; it may be added later only as an explicit compatibility shim or internal helper.

## 2. Motivation

Before the first alpha release, a user must be able to answer:

- How do I install Ash?
- Does installation include the standard library and standard tooling?
- How do I update Ash, and does that update the stdlib/tooling coherently?
- Where do installed files, caches, runtime state, and project dependency locks live?
- How do I deploy an Ash project that depends on git-hosted Ash libraries before a registry exists?

The alpha answer is intentionally thin but strict: install immutable toolchain bundles, keep the standard library coupled to the selected toolchain, and use git URLs resolved to immutable commits for third-party libraries.

## 3. Normative terms

- **Ash toolchain:** A coherent versioned bundle containing Ash executables, standard library, standard tooling, runtime support metadata, and install metadata.
- **Installed toolchain:** An immutable toolchain directory under the user-local XDG data tree.
- **Active toolchain:** The toolchain selected for `ash`, `ashgrove`, and standard tools by explicit override, project pin, or user default.
- **Toolchain selector:** User or project metadata that chooses a toolchain version without modifying installed toolchain contents.
- **Launcher shim:** A stable executable under `$HOME/.local/bin` that resolves the active toolchain and then execs the selected versioned binary. A launcher shim is not the same thing as the versioned binary it dispatches to.
- **Source install:** An install path that builds Ash from a git checkout or source archive on the target machine.
- **Binary tarball install:** An install path that verifies and unpacks a prebuilt Ash release archive.
- **Ash project manifest:** Project-local `ash.toml` metadata for package identity, toolchain compatibility, dependencies, and project configuration.
- **Ash lockfile:** Project-local `ash.lock` recording exact git commit resolution, package metadata digests, and toolchain compatibility evidence.
- **Standard library:** The Ash `std` library shipped as part of a toolchain. It is not an ordinary third-party dependency in the alpha install model.
- **Standard tooling:** Tools released with and compatibility-tested against a toolchain, including at minimum `ash` and `ashgrove`, plus any stabilized sibling tools selected by the release policy.

## 4. Scope

### 4.1 In scope

1. User-local XDG-compatible installation.
2. `ashgrove <command>` command naming and behavior.
3. Source install and binary tarball install semantics.
4. Immutable versioned toolchain directories.
5. Bundled stdlib and standard tooling policy.
6. Update, remove, and cleanup semantics.
7. Project manifest and lockfile fields needed for git-based deployment.
8. Git dependency fetch/lock behavior for tags and exact revisions.
9. Compiler/module-loader integration needed for locked git dependencies to be importable.
10. Reserved trust/signing metadata without mandatory signature enforcement.

### 4.2 Out of scope

1. System-wide/global installation roots.
2. A hosted Ash package registry.
3. Release-channel discovery or remote version indexes.
4. Dependency solving across SemVer ranges from a registry.
5. Mandatory package signing or transparency logs.
6. Editor plugin installation.
7. OS package-manager integration (`apt`, `dnf`, Homebrew, Nix, etc.).
8. Automatic user-project rewriting during toolchain update.
9. Independent stdlib updates decoupled from toolchain updates.

## 5. Command surface

The install manager command is `ashgrove`. `ash` remains the language/tool CLI. `ashgrove` manages installation and deployment substrate; it must not become a second language execution CLI.

Required alpha user commands:

```text
ashgrove install --from source --path PATH [--version VERSION] [--rev REV] [--allow-dirty-source] [--allow-unidentified-source] [--switch]
ashgrove install --from tarball (--path PATH | --url URL) [--version VERSION] [--digest sha256:...] [--switch]
ashgrove update --to VERSION --from source --path PATH [--allow-dirty-source] [--allow-unidentified-source] [--switch]
ashgrove update --to VERSION --from tarball (--path PATH | --url URL) [--digest sha256:...] [--switch]
ashgrove default <toolchain-id>
ashgrove list
ashgrove current [--project PATH]
ashgrove remove <toolchain-id> [--force]
ashgrove cleanup [--project PATH] [--dry-run] [--cache] [--orphans] [--old-toolchains]
ashgrove fetch [--project PATH]
ashgrove lock [--project PATH] [--check]
ashgrove vendor [--project PATH] [--output PATH] [--check]
```

Bare version install/update, for example `ashgrove install 0.1.0-alpha.1`, is rejected until a later release-index/channel policy defines how a version maps to an authenticated source or tarball URL.

Required release-side tooling:

- A conforming binary tarball producer must exist before binary tarball install can be accepted. The first-slice producer is the repository script `scripts/package-ash-toolchain.sh`.
- The producer must generate the archive shape, metadata, stdlib payload, and digest consumed by `ashgrove install --from tarball`.

## 6. Toolchain bundle and launcher contract

Installing Ash installs one coherent toolchain bundle.

A toolchain bundle MUST contain:

```text
toolchains/<toolchain-id>/
  bin/
    ash
    ashgrove
    # optional compatibility/internal helpers may appear here only when specified
  lib/ash/
    std/
      ash.toml              # generated/staged std package manifest
      src/
    runtime/                # optional/reserved unless TASK-965 identifies concrete payloads
    schemas/                # optional/reserved unless TASK-965 identifies concrete payloads
  share/ash/
    completions/            # optional in first slice
    man/                    # optional in first slice
    reference/              # optional generated/read-only reference snapshot
  manifest.toml
  install-record.toml
```

The release must freeze the exact public standard-tool list during TASK-965. The minimum required public tool binaries for this spec are `ash` and `ashgrove`. Existing sibling binaries such as `ash-lsp`, `ash-lint`, or `ash-mcp` may be included only if TASK-965 classifies them as release tooling for this alpha. Developer/test binaries such as doc-test or fuzz helpers must not be installed as standard user tooling unless a later release policy says so.

The daemon is invoked through `ash daemon ...` inside the selected `ash` binary. If a future `ashd` binary exists, it is either an optional compatibility shim for `ash daemon ...` or an internal helper; it is not a separate semantic daemon surface.

Launcher shims under `$HOME/.local/bin` MUST be stable dispatchers:

```text
$HOME/.local/bin/ash       -> resolves active toolchain, then execs toolchains/<id>/bin/ash
$HOME/.local/bin/ashgrove  -> resolves active/default manager, then execs toolchains/<id>/bin/ashgrove
```

The launcher must resolve in this order:

1. Explicit CLI override, if provided.
2. Project toolchain pin in `ash.toml`.
3. User default in `$XDG_CONFIG_HOME/ash/toolchains.toml`.
4. Fail with an actionable diagnostic if no suitable toolchain is installed.

The launcher contract avoids the bug where a direct symlink to the user default would prevent `ash` from seeing a project-local toolchain pin before process startup.

`ashgrove` itself is bundled in each toolchain so that install/update behavior is versioned and reproducible. The stable launcher must protect itself during removal: `ashgrove remove --force` MUST NOT delete the toolchain that currently provides the running manager binary.

The standard library is bundled in `lib/ash/std/` and selected by the active toolchain. For alpha, users MUST NOT update the bundled stdlib independently of the toolchain.

## 7. Source install vs binary tarball install

### 7.1 Source install

A source install builds from a git checkout or source archive on the target machine. A source archive is acceptable only if it carries release-source metadata that identifies the originating commit hash; otherwise `ashgrove` must reject it unless `--allow-unidentified-source` is passed and the install is marked non-reproducible.

A source install MUST:

1. Record the source URL if available.
2. Record the source revision commit hash when available.
3. Record whether `--allow-dirty-source` or `--allow-unidentified-source` was used.
4. Record the build profile and target triple.
5. Build the public tool binaries from source.
6. Copy the standard library from the source tree or generated release staging directory.
7. Generate/stage `lib/ash/std/ash.toml` or fail if stdlib package metadata cannot be produced.
8. Generate `manifest.toml` and `install-record.toml`.
9. Install into a temporary staging directory first, then atomically publish the completed toolchain directory.

A source install MUST NOT install from a dirty checkout unless the user passes `--allow-dirty-source`. Dirty-source installs must mark `install-record.toml` as non-reproducible.

### 7.2 Binary tarball install

A binary tarball install unpacks a prebuilt Ash release archive.

A binary tarball install MUST:

1. Verify the tarball shape before publishing it as installed.
2. Verify the included `manifest.toml` and `install-record.toml` schema, including first-slice `archive_schema_version = 1`.
3. Confirm the archive version/toolchain id matches the target toolchain directory name.
4. Confirm required executable files exist and have executable permissions.
5. Confirm the bundled stdlib manifest exists.
6. Record tarball path/URL, digest, and install time.
7. Install into a temporary staging directory first, then atomically publish the completed toolchain directory.

Tarball extraction MUST be safe. `ashgrove` must reject absolute paths, `..` traversal, symlink/hardlink escapes, device files, setuid/setgid bits, and any archive entry that would write outside the staging directory.

Binary tarball signing is reserved but not required for the first alpha implementation. If a signature or attestation is present, `ashgrove` may record it as observed evidence without enforcing trust by default.

## 8. XDG user-local layout

Alpha installs are user-local and XDG-compatible.

If environment variables are unset, defaults follow XDG conventions:

| Purpose | XDG variable | Default | Ash path |
| --- | --- | --- | --- |
| Executable launchers | n/a | `$HOME/.local/bin` | `$HOME/.local/bin/ash`, `$HOME/.local/bin/ashgrove` |
| Installed toolchains | `XDG_DATA_HOME` | `$HOME/.local/share` | `$XDG_DATA_HOME/ash/toolchains/<toolchain-id>/` |
| User config/selectors | `XDG_CONFIG_HOME` | `$HOME/.config` | `$XDG_CONFIG_HOME/ash/` |
| Disposable cache | `XDG_CACHE_HOME` | `$HOME/.cache` | `$XDG_CACHE_HOME/ash/` |
| Runtime state/logs/locks | `XDG_STATE_HOME` | `$HOME/.local/state` | `$XDG_STATE_HOME/ash/` |

Required user config files:

```text
$XDG_CONFIG_HOME/ash/
  toolchains.toml       # installed/default selector metadata and optional known-project roots
  config.toml           # user preferences; optional in first slice
```

Required cache/state directories:

```text
$XDG_CACHE_HOME/ash/
  downloads/
  git/repos/
  git/checkouts/
  builds/
  module-cache/

$XDG_STATE_HOME/ash/
  locks/
  logs/
  daemon/
```

Daemon state under `$XDG_STATE_HOME/ash/daemon/` remains subject to [SPEC-070](SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md) same-user ownership and not-group/world-writable validation.

Toolchain directories are immutable after successful installation. `ashgrove update` installs a new directory; it does not patch the old one in place.

## 9. Project manifest and toolchain selection

The canonical alpha project/package manifest filename is lower-case `ash.toml`.

Existing `.ash.toml` configuration references in older specs remain compatibility configuration, not the canonical package manifest. During the migration window:

1. `ash.toml` owns package, toolchain, dependency, and future entrypoint metadata.
2. `.ash.toml` may continue to hold legacy CLI/LSP configuration.
3. If both files contain package/dependency/toolchain fields, `ashgrove` must reject the project with a migration diagnostic rather than merging ambiguous metadata.
4. TASK-965 must audit and patch SPEC-005/SPEC-038 discovery wording if implementation requires a stronger compatibility rule.

A project MAY pin a toolchain:

```toml
[toolchain]
ash = "0.1.0-alpha.1"
```

A project MAY specify a compatibility range when exact pinning is too strict:

```toml
[toolchain]
ash = ">=0.1.0-alpha.1, <0.2"
```

For alpha release/deployment reproducibility, exact pins are preferred. Compatibility ranges must resolve deterministically to an installed exact toolchain before execution. The alpha default is the highest installed compatible version according to SemVer/pre-release ordering, and the resolved exact version must be recorded in `ash.lock` when a lockfile is present.

## 10. Update policy

`ashgrove update` installs a new coherent toolchain bundle and optionally switches the active default. `--from source` and `--from tarball` use the same source/tarball validation rules as `install`, and alpha local updates require `--to` to match the computed source-root identity or local tarball payload identity before publishing. Channel-based update discovery and authenticated URL download are deferred until a release-index/channel policy exists.

An update MUST update together:

1. `ash`.
2. `ashgrove`.
3. The bundled stdlib.
4. Standard tooling shipped for the release.
5. Runtime support metadata and schemas if present.
6. Toolchain metadata.

An update MUST NOT:

1. Mutate an existing installed toolchain directory.
2. Rewrite project source files.
3. Update third-party project dependencies unless a dependency command is explicitly invoked.
4. Remove older toolchains unless `cleanup` or `remove` is explicitly invoked.

`ashgrove update --switch` may update the user default after the new toolchain is installed and verified. Without `--switch`, update installs the new toolchain but preserves the current default unless no default exists and the user explicitly accepts initialization.

First install may initialize the user default if no default exists. Subsequent installs do not switch the default unless `--switch` or `ashgrove default <toolchain-id>` is used.

Existing-version installs MUST be deterministic. If the target toolchain id already exists, `ashgrove` must either no-op when the existing manifest/digest is identical or reject with an explicit `already installed` diagnostic. It must not overwrite an installed immutable toolchain in place. Source builds with different commit/profile/target but the same package version require a distinct toolchain id or explicit rejection; TASK-965 must choose the first-slice id scheme. If multiple installed toolchain ids share the same package version, commands that select or remove a toolchain must require an exact toolchain id rather than guessing.

## 11. Remove and cleanup policy

`ashgrove remove <toolchain-id>` removes one installed toolchain.

Removal MUST refuse to remove a toolchain if:

1. It is the user default and `--force` was not provided.
2. It is currently selected by the current project and `--force` was not provided.
3. A live `ash daemon` instance reports that it is using that toolchain.
4. It provides the currently running `ashgrove` manager binary.

`--force` may override user-default and project-selector protections after an explicit confirmation. `--force` MUST NOT override live daemon usage or currently running manager protection. Removing live daemon toolchains requires stopping the daemon first.

To support live daemon protection, the daemon control plane must expose or register toolchain id/root state under `$XDG_STATE_HOME/ash/daemon/`. TASK-971 owns the minimal integration needed by `ashgrove remove`; A73-6 cannot be accepted without this evidence.

`ashgrove cleanup` removes disposable or unreachable data according to flags:

- `--cache`: remove downloads, build cache, module cache, and git checkouts that can be refetched or rebuilt.
- `--orphans`: remove staged/incomplete installs and cache entries not referenced by any lockfile or installed toolchain metadata.
- `--old-toolchains`: remove non-default toolchains that are not pinned by known projects, subject to confirmation.
- `--dry-run`: print planned deletions without deleting.

For alpha, **known projects** means only the current project supplied by `--project`/current working directory plus any explicit project roots recorded in `$XDG_CONFIG_HOME/ash/toolchains.toml`. `ashgrove` MUST NOT crawl the user's filesystem looking for projects during cleanup.

Cleanup MUST NOT delete project-local `ash.toml` or `ash.lock` files.

Current implementation note: the TASK-971 cleanup planner slice implements dry-run reporting for project-pinned toolchains, conservative deletion for known Ash-owned cache children, invalid toolchain-directory orphan cleanup under the XDG toolchain root, and old-toolchain cleanup that preserves default, project-pinned, live-daemon, and running-manager toolchains. It does not yet implement broader lockfile/cache reachability analysis or interactive confirmation policy.

## 12. Project manifest and git dependency metadata

Project-local `ash.toml` is the package/project manifest.

Minimum alpha fields:

```toml
[package]
name = "example"
version = "0.1.0"
kind = "app" # app | library
license = "MIT OR Apache-2.0"

[toolchain]
ash = "0.1.0-alpha.1"

[dependencies]
foo = { git = "https://github.com/example/foo.ash.git", tag = "v0.1.2" }
bar = { git = "ssh://git@github.com/example/bar.ash.git", rev = "0123456789abcdef0123456789abcdef01234567" }
```

For alpha:

1. Dependency entries MUST use `git` plus exactly one of `tag` or `rev` for reproducible deployment.
2. Manifest `rev` values SHOULD be full commit hashes. If an abbreviation is accepted, `ashgrove lock` must expand and verify it to a full commit hash in `ash.lock`.
3. Branch dependencies are dev-only and MUST require an explicit unstable marker if supported.
4. Unpinned git dependencies are rejected outside an explicit local development override.
5. Tags are intent; resolved commits are execution truth.
6. `ashgrove lock` MUST resolve every tag to a commit hash and record that hash in `ash.lock`.

Reserved trust fields may appear in project manifests and lockfiles:

```toml
[trust]
signing = "none" # none | minisign | sigstore | gpg, future
signature = ""
attestation = ""
```

Read-modify-write operations MUST preserve unknown future-compatible trust fields unless the command explicitly rewrites the whole file from a documented schema and reports that preservation is unavailable.

## 13. Lockfile contract

`ash.lock` records exact dependency resolution.

Minimum lockfile information:

```toml
version = 1

[toolchain]
requested = "0.1.0-alpha.1"
resolved = "0.1.0-alpha.1"
stdlib = "0.1.0-alpha.1"

[[package]]
name = "foo"
version = "0.1.2"
source = "git+https://github.com/example/foo.ash.git"
requested = { tag = "v0.1.2" }
resolved = { rev = "0123456789abcdef0123456789abcdef01234567" }
manifest_digest = "sha256:..."
content_digest = "sha256:..." # optional in first slice if too expensive
```

The lockfile MUST be stable enough for review diffs. Implementations should sort packages by name/source.

`ashgrove lock --check` MUST fail if `ash.toml` dependencies are not represented exactly in `ash.lock`.

## 14. Git dependencies and module resolution

`ashgrove` git dependency work is not complete unless locked dependencies become visible to `ash check` and `ash run`.

For alpha:

1. `[dependencies]` keys in `ash.toml` define external package aliases/module roots.
2. Each locked package root is either a fetched checkout under `$XDG_CACHE_HOME/ash/git/checkouts/` or a vendored root under the project vendor directory.
3. `ash-cli`/`ash-engine` must receive a dependency-root map from `ash.toml` + `ash.lock` or from the vendor metadata.
4. The module loader must search those package roots when resolving imports for dependency aliases.
5. A source-level `dependency <alias> from "<path>";` declaration from [SPEC-009](SPEC-009-MODULES.md) remains a local/path dependency form. A duplicate alias between source declarations and `ash.toml` dependencies is rejected in alpha rather than merged.

Current implementation note: the Phase 127 follow-up slices teach `ash-engine` module resolution to discover an ancestor lower-case `ash.toml`, read `ash.lock` only when `vendor/ash/` exists, validate locked package names and full 40-character commits, and add `vendor/ash/` plus locked `vendor/ash/<package>/` directories for `ash check src/main.ash` and explicit ordinary-file `ash run src/main.ash:main` without requiring `ASH_DEP_ROOTS` or `ASH_DEPENDENCY_ROOTS`. Auto-discovered project vendor roots are searched after the selected stdlib root, so a locked vendored package shaped like a stdlib module does not override the active or explicit stdlib. The vendor gate also validates lock package names and commits when a `vendor/ash` root or `vendor/ash/<package>` root is supplied explicitly through dependency-root environment variables, so that ambient roots cannot bypass the lock boundary. It does not crawl arbitrary directories. This remains partial because fetched-cache roots are not discovered without vendoring.

TASK-972 owns the compiler/module-loader integration. Merely fetching git repositories is not enough to satisfy this spec.

## 15. Fetch, vendor, and deployable git projects

`ashgrove fetch` reads `ash.toml` and `ash.lock`, fetches missing git sources into the XDG cache, and checks out exact locked revisions.

`ashgrove vendor` materializes locked dependencies into a project-controlled directory for offline deployment. The default vendor directory is `vendor/ash/` unless `--output PATH` is supplied. The vendor directory format is intentionally tool-owned in alpha but must include enough metadata to prove which lockfile entry produced each vendored package.

`ashgrove vendor --check` validates that the vendor directory exactly matches `ash.lock` without writing or fetching anything.

Current implementation note: the Phase 127 metadata/staging slice adds typed `ashgrove` toolchain manifest and install-record carriers, XDG selector metadata that preserves reserved trust/signing fields during read-modify-write, stdlib package metadata staging with fail-closed missing-stdlib behavior, deterministic staged-publish collision helpers, and stable `ash`/`ashgrove` launcher shims that dispatch through typed metadata under temporary XDG/home roots. The launcher shims target a stable user-local `.ashgrove-dispatcher` copy instead of a transient versioned `current_exe()` path, preserve selected-tool exit status, reject selected toolchain roots that are symlinks, and harden shim temp-file writes. The tarball slice provides `scripts/package-ash-toolchain.sh` as a repository release producer, validates typed manifest/install-record identity and `archive_schema_version = 1` before publish, rejects unsafe symlink, hardlink, absolute-path, parent-traversal, and setuid archive entries, installs through staged publish, and records local tarball path, digest, and install time. The git deployment slice materializes local git dependencies into `$XDG_CACHE_HOME/ash/git/repos/<package>-<url-digest>.git` plus `$XDG_CACHE_HOME/ash/git/checkouts/<package>-<url-digest>/<commit>/`, expands accepted abbreviated manifest `rev` values to full lockfile commits, preserves existing lockfile `[trust]` metadata during `ashgrove lock` rewrites, and `vendor` copies and checks package content from those locked checkouts. Follow-up slices add `ash check src/main.ash` and explicit ordinary-file `ash run src/main.ash:main` discovery for the default `vendor/ash/` layout, with selected stdlib roots taking precedence over stdlib-shaped vendored packages. This is still not full acceptance because direct fetched-cache dependency roots, broader lower-case package/toolchain metadata coverage, packaged dispatcher lifecycle, source-archive release metadata, and authenticated tarball URL download/recording remain deferred.

Deployment from git-based Ash projects MUST be possible with explicit current CLI forms:

```text
ashgrove install --from tarball --path ash-0.1.0-alpha.1-x86_64-unknown-linux-gnu.tar.gz --version 0.1.0-alpha.1 --switch
ashgrove lock --check
ashgrove fetch
ash check src/main.ash
ash run src/main.ash:main
```

or offline with a previously generated vendor directory:

```text
ashgrove install --from tarball --path ash-0.1.0-alpha.1-x86_64-unknown-linux-gnu.tar.gz --version 0.1.0-alpha.1 --switch
ashgrove vendor --check
ash check src/main.ash
ash run src/main.ash:main
```

Manifest-aware defaults such as bare `ash check` or `ash run <entry>` require a later CLI/entrypoint spec or an explicit amendment to [SPEC-005](SPEC-005-CLI.md). They are not assumed by this spec.

The standard library comes from the selected toolchain, not from the project dependency graph.

## 16. Installed stdlib discovery

The installed `ash` binary must use the selected toolchain's bundled stdlib.

The live pre-SPEC-073 implementation has historically used workspace-relative stdlib fallbacks such as `std/src`. TASK-965 must bind the exact live files and TASK-967/TASK-968 must implement a release-safe mechanism before source or tarball install can be accepted. Acceptable mechanisms include:

1. Resolve stdlib relative to the selected toolchain root / current executable.
2. Have launcher shims pass an explicit stdlib root to the selected toolchain.
3. Add an `EngineBuilder`/CLI stdlib-root setting and route selected-toolchain paths through `ash check` and `ash run`.

Acceptance evidence must prove an installed/tarball-style `ash` uses a temporary toolchain stdlib and does not accidentally read the source workspace `std/src`.

Current implementation note: the TASK-968 follow-up slice proves the `ash-engine` selected-stdlib override seam with a temporary installed-style stdlib root. Source installs now build real local source checkouts from an isolated cache copy with an external Cargo target dir, stage immutable toolchains through the staged publish path, record source URL/revision/build profile/target triple plus dirty/unidentified override and reproducibility state, reject dirty or unidentified source roots unless the matching explicit override is present, fail closed when git-like roots cannot report `HEAD` or dirty status, reject same-id metadata conflicts, keep identical reinstalls deterministic, and refresh launcher shims that pass the selected toolchain stdlib root to `ash`. This remains partial because source archive release metadata and concrete runtime-support payload metadata are still undefined.

## 17. Rust/tooling implementation constraints

TASK-965 must freeze exact implementation choices before Rust work starts:

1. `ashgrove` implementation home. Preferred first-slice shape is a new workspace crate/binary `crates/ashgrove`, not an `ash` subcommand.
2. TOML dependency (`toml`, `toml_edit`, or equivalent). Unknown trust fields require preservation-aware parsing/writing.
3. Git integration. Shelling out to `git` is acceptable for the first slice if the `git` executable prerequisite is documented and tests use local temporary repositories.
4. Tarball dependencies and compression format.
5. XDG path implementation strategy.
6. Toolchain package producer shape.
7. Exact public standard-tool list for the alpha bundle.
8. Module-loader integration points for dependency roots and installed stdlib roots.
9. Daemon status/registry integration points for live toolchain removal protection.

## 18. Trust and signing placeholders

The first implementation reserves trust metadata but does not require signature enforcement.

Manifests and lockfiles MAY include:

```toml
[trust]
signing = "none" # none | minisign | sigstore | gpg, future
signature = ""
attestation = ""
```

A later spec may make signature verification mandatory for release channels or registries.

## 19. Diagnostics

Diagnostics MUST be actionable and distinguish:

1. Missing toolchain.
2. Installed but incompatible toolchain.
3. Missing stdlib inside a toolchain.
4. Broken or incomplete installed toolchain metadata.
5. Bare version install/update rejected because no release-index/channel policy exists.
6. Dirty source install without `--allow-dirty-source`.
7. Source archive without source commit metadata and without `--allow-unidentified-source`.
8. Tarball shape/schema/version mismatch.
9. Unsafe tarball entries.
10. Floating/unpinned git dependency.
11. Tag resolved to a different commit than the lockfile records.
12. Duplicate dependency aliases between `ash.toml` and source `dependency ... from` declarations.
13. Project manifest filename conflict between `ash.toml` and `.ash.toml`.
14. Project entrypoint/default command unsupported by the current CLI contract.
15. Attempted removal of active/default/live/running-manager toolchain.
16. Cleanup dry-run vs actual deletion.

## 20. Acceptance criteria

- **A73-1:** A user can install Ash from source into XDG user-local paths.
- **A73-2:** A user can install Ash from a binary tarball into the same layout.
- **A73-3:** Both install modes produce equivalent required toolchain contents: `ash`, `ashgrove`, stdlib, metadata, and any TASK-965-frozen standard tools.
- **A73-4:** `ashgrove update` installs a new immutable toolchain without mutating the old one.
- **A73-5:** `ashgrove default` switches active toolchain-id metadata/launchers without rewriting projects.
- **A73-6:** `ashgrove remove` refuses active/default/live/running-manager toolchain deletion according to the force and safety rules.
- **A73-7:** `ashgrove cleanup --dry-run` reports cache/orphan/old-toolchain deletions without deleting.
- **A73-8:** `ash.toml` git dependencies with tags resolve to exact commits in `ash.lock`.
- **A73-9:** `ashgrove lock --check` detects drift between manifest and lockfile.
- **A73-10:** The selected toolchain provides the stdlib; project dependency resolution does not fetch stdlib as a normal third-party dependency.
- **A73-11:** Reserved trust/signing metadata is parsed or preserved as reserved metadata without mandatory enforcement.
- **A73-12:** Locked git dependencies are visible to `ash check` and `ash run` through the module/dependency root integration.

## 21. Implementation tasks

- [TASK-964](../plan/tasks/TASK-964-ashgrove-install-policy-packet.md): Create this spec/plan/task packet and register Phase 127.
- [TASK-965](../plan/tasks/TASK-965-ashgrove-live-install-audit-gate.md): Audit existing CLI, build, release, stdlib, daemon, and XDG seams before implementation.
- [TASK-966](../plan/tasks/TASK-966-ashgrove-cli-crate-and-command-skeleton.md): Add the `ashgrove` command skeleton and shared error/reporting substrate.
- [TASK-967](../plan/tasks/TASK-967-toolchain-metadata-and-xdg-layout.md): Implement toolchain metadata, selectors, XDG path resolution, launcher dispatch, stdlib metadata staging, trust-field preservation, and atomic staging/publish helpers.
- [TASK-968](../plan/tasks/TASK-968-source-install-flow.md): Implement source install.
- [TASK-969](../plan/tasks/TASK-969-binary-tarball-install-flow.md): Implement binary tarball production/consumption and install.
- [TASK-970](../plan/tasks/TASK-970-update-default-list-current-flow.md): Implement update, default, list, and current selection flows.
- [TASK-971](../plan/tasks/TASK-971-remove-cleanup-flow.md): Implement remove and cleanup policy, including daemon/running-manager protection.
- [TASK-972](../plan/tasks/TASK-972-ash-manifest-lock-git-fetch.md): Implement `ash.toml` dependency metadata, `ash.lock`, git fetch, lock checking, trust-field preservation, and module-loader dependency-root integration.
- [TASK-973](../plan/tasks/TASK-973-vendor-and-deployable-git-project-flow.md): Implement vendor/offline deployment flow for git-based Ash projects.
- [TASK-974](../plan/tasks/TASK-974-ashgrove-closeout-acceptance.md): Close out SPEC-073 with acceptance matrix, broad gates, and independent review remediation.

## 22. Changelog

### 2026-05-28

- TASK-970 completed the alpha local update/default/list/current selector surface: source updates build from real source roots, local tarball updates consume producer-compatible payloads, `--to` must match payload identity, selectors preserve or switch defaults according to `--switch`, incomplete toolchains fail closed, and bare/network update remains rejected pending release-index/download policy.
- TASK-966 completed the public `ashgrove` command skeleton evidence, including isolated non-zero CLI smoke tests and fail-closed bare install/update version rejection before release-index policy exists.
- TASK-967 added typed toolchain metadata, selector trust preservation, stdlib metadata staging, staging publish/collision helpers, and stable launcher shim installation/dispatch coverage under temporary XDG/home roots.
- Initial draft for `ashgrove` user-local install/update/remove/cleanup policy, source vs binary tarball install semantics, XDG layout, and git-pinned deployment substrate.
