# ashgrove - Ash Toolchain Manager

`ashgrove` is Ash's user-local toolchain and deployment manager. It installs coherent Ash toolchain bundles and selects which installed toolchain should run `ash`/`ashgrove`.

It also updates by publishing new immutable toolchains instead of mutating old ones, removes or cleans unused local state conservatively, and prepares git-pinned Ash project dependencies for reproducible or offline deployment.

The current implementation is the SPEC-073 Implemented MVP. It is intentionally strict and local-first:

- installs are user-local and XDG-compatible;
- installed toolchains are immutable directories under the Ash data root;
- launcher shims dispatch to the selected installed toolchain;
- source and tarball installs must include `ash`, `ashgrove`, stdlib metadata/source, runtime-support metadata, and install metadata;
- project dependencies are git-pinned in `ash.toml`, resolved into exact commits in `ash.lock`, fetched into the Ash cache, and optionally vendored into `vendor/ash/`;
- unsupported release/registry flows fail closed instead of guessing.

See also:

- [SPEC-073](../../docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md)
- [PLAN-122](../../docs/plan/PLAN-122-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md)
- [PLAN-123](../../docs/plan/PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Prerequisites and command prefix

For repository-local development, run commands from the repository root with Cargo:

```bash
cargo run -p ashgrove -- <ashgrove-args>
```

After installing a toolchain and ensuring `$HOME/.local/bin` is on `PATH`, use the installed command directly:

```bash
ashgrove <ashgrove-args>
```

The examples below use installed-command form for readability. During first install from a checkout, substitute `cargo run -p ashgrove --` for `ashgrove`.

`ashgrove` expects the Ash repository's Rust toolchain and `git` to be available when building or locking from source. Packaging examples run repository scripts from the repository root.

## Build and inspect the CLI

From the repository root:

```bash
cargo run -p ashgrove -- --help
cargo run -p ashgrove -- install --help
cargo run -p ashgrove -- update --help
```

When running an installed `ashgrove`, drop the `cargo run -p ashgrove --` prefix:

```bash
ashgrove list
ashgrove current
```

## Where ashgrove writes files

`ashgrove` derives paths from `HOME` and the standard XDG variables:

| Purpose | Default path |
| --- | --- |
| Stable launcher shims | `$HOME/.local/bin/ash`, `$HOME/.local/bin/ashgrove` |
| Installed toolchains | `$XDG_DATA_HOME/ash/toolchains/` or `$HOME/.local/share/ash/toolchains/` |
| Selector/config metadata | `$XDG_CONFIG_HOME/ash/` or `$HOME/.config/ash/` |
| Fetched git cache | `$XDG_CACHE_HOME/ash/` or `$HOME/.cache/ash/` |
| Runtime/local state | `$XDG_STATE_HOME/ash/` or `$HOME/.local/state/ash/` |

Tests and scripted experiments should set temporary `HOME`/`XDG_*` roots so they do not touch a developer's real Ash installation state.

## Scenario: install from a source checkout

Use this when you have a local Ash source checkout and want `ashgrove` to build and publish an installed toolchain from it.

```bash
ashgrove install --from source --path /path/to/ash --switch
```

Behavior:

1. builds the source root in an isolated cache/staging area;
2. stages `ash`, `ashgrove`, stdlib, runtime-support metadata, manifest, and install record;
3. publishes the toolchain under the user-local toolchain root;
4. installs or refreshes stable launcher shims;
5. switches the user default when `--switch` is passed, or when no default exists yet.

Safety rules:

- dirty source roots are rejected unless `--allow-dirty-source` is passed;
- unidentified source roots are rejected unless `--allow-unidentified-source` is passed;
- non-reproducible installs record that fact in install metadata.

## Scenario: install from a source archive-shaped directory

A source archive-shaped directory must carry release-source metadata unless you explicitly accept a non-reproducible install.

```bash
ashgrove install --from source --path /path/to/source-archive --switch
```

Expected archive metadata includes `release-source.toml` with the origin commit and required attestation evidence. If the archive is intentionally unidentified:

```bash
ashgrove install --from source --path /path/to/source-archive --allow-unidentified-source
```

This records the archive digest and non-reproducibility boundary rather than pretending the archive is identified.

## Scenario: install from a local binary tarball

Use the repository producer script to create a schema-versioned toolchain tarball, then install that tarball.

```bash
scripts/package-ash-toolchain.sh --output-dir /tmp/ash-toolchains
# The script prints archive=/tmp/ash-toolchains/<toolchain-id>.tar.gz
# and digest=sha256:<64-hex>.
ashgrove install \
  --from tarball \
  --path /tmp/ash-toolchains/<toolchain-id>.tar.gz \
  --digest sha256:<64-hex> \
  --switch
```

Tarball installs validate the archive shape before publishing. The tarball must contain executable `ash` and `ashgrove` binaries, stdlib metadata/source, runtime-support metadata, `manifest.toml`, and `install-record.toml`. Unsafe archive entries, missing executable bits, missing metadata, schema mismatches, identity mismatches, and digest mismatches fail before publish.

If you already know the expected SHA-256 digest, require it explicitly:

```bash
ashgrove install --from tarball --path /tmp/ash-toolchains/<toolchain-id>.tar.gz --digest sha256:<64-hex> --switch
```

## Scenario: install or update from an explicit-digest local file URL

The current MVP supports explicit-digest `file://` tarball URLs. It does not implement best-effort network lookup, hosted HTTP release URLs, or a hosted release channel.

```bash
ashgrove install \
  --from tarball \
  --url file:///tmp/ash-toolchains/<toolchain-id>.tar.gz \
  --digest sha256:<64-hex> \
  --switch
```

For update:

```bash
ashgrove update \
  --to ash-<version-or-id> \
  --from tarball \
  --url file:///tmp/ash-toolchains/<toolchain-id>.tar.gz \
  --digest sha256:<64-hex> \
  --switch
```

Unsigned or unbound release-index flows remain fail-closed. Release-index signature metadata is not accepted as digest evidence until a later resolver binds toolchain id, tarball URL, and digest.

## Scenario: update to a new toolchain

`update` installs a new immutable toolchain and optionally switches selectors. It does not mutate the old installed toolchain.

From a source checkout:

```bash
ashgrove update --to ash-<new-id> --from source --path /path/to/ash --switch
```

From a local tarball:

```bash
ashgrove update --to ash-<new-id> --from tarball --path /tmp/ash-toolchains/<toolchain-id>.tar.gz --digest sha256:<64-hex>
```

Rules:

- `--to` is required and must be a valid installed toolchain id shape beginning with `ash-`;
- source/tarball payload identity must match the requested `--to` id;
- without `--switch`, the existing user default is preserved;
- bare version updates such as `ashgrove update 1.2.3` are rejected until a release-index policy exists.

## Scenario: inspect and switch the selected toolchain

List installed toolchains:

```bash
ashgrove list
```

Print the selected user-default toolchain:

```bash
ashgrove current
```

Print the selected toolchain for a specific project:

```bash
ashgrove current --project /path/to/project
```

Set the user default:

```bash
ashgrove default ash-<toolchain-id>
```

Launcher selection order is:

1. explicit `ASH_TOOLCHAIN` override;
2. project pin from project metadata;
3. user default selector.

The stable launcher shims use the current working directory for project-pin selection. The `current` inspection command checks a project pin only when `--project PATH` is supplied.

The stable launcher shims pass selected-toolchain stdlib and runtime-support metadata to the selected `ash` binary so commands such as `ash check` and `ash run` use the installed toolchain contents rather than the workspace checkout.

## Scenario: remove an installed toolchain

Remove an installed toolchain by id:

```bash
ashgrove remove ash-<toolchain-id>
```

`remove` refuses to delete protected toolchains, including the user default, the current project-pinned toolchain, live-daemon toolchains, and the running manager toolchain. `--force` is limited to explicitly allowed selector override cases; daemon/running-manager protections remain fail-closed.

```bash
ashgrove remove ash-<toolchain-id> --force
```

Use `list` and `current` before removal when you are unsure which toolchain is active.

## Scenario: plan or perform cleanup

Start with a dry run:

```bash
ashgrove cleanup --project /path/to/project --dry-run --cache --orphans --old-toolchains
```

Cleanup categories:

| Flag | Meaning |
| --- | --- |
| `--cache` | remove known Ash-owned cache children under the Ash cache root when safe |
| `--orphans` | remove invalid toolchain directories under the toolchain root |
| `--old-toolchains` | remove unprotected old installed toolchains after confirmation checks |
| `--project PATH` | include project lockfile/vendor provenance in reachability analysis |
| `--dry-run` | report the cleanup plan without deleting anything |

Cleanup is conservative. It preserves default/project-pinned/live/running-manager toolchains, lockfile-referenced fetched checkouts, project-pinned toolchains, vendor provenance references, and project-local `ash.toml`/`ash.lock` files.

## Scenario: lock git dependencies

Project dependencies live in lower-case `ash.toml` and are resolved into exact commits in `ash.lock`. A minimal git dependency entry looks like:

```toml
[package]
name = "app"

[dependencies.dep]
git = "file:///path/to/dep.git"
tag = "v1"
```

Then run:

```bash
ashgrove lock --project /path/to/project
```

Check whether `ash.lock` is still consistent with `ash.toml`:

```bash
ashgrove lock --project /path/to/project --check
```

The lock flow rejects unpinned git dependencies, resolves accepted full or abbreviated `rev` values and tags to exact commits, preserves trust/signing metadata during rewrites, rejects untrusted remote protocols, and redacts credentials before lockfile serialization.

Current caveat: `lock` resolves metadata; it does not perform an arbitrary remote clone as part of release-channel discovery. For non-`file://` tag or abbreviated-revision resolution, provide the expected local git metadata/cache according to the project flow before locking.

## Scenario: fetch locked dependencies

After locking, fetch the exact dependency checkouts for the project manifest/lockfile pair:

```bash
ashgrove fetch --project /path/to/project
```

Fetched checkouts are materialized under the Ash cache using package/source/commit identity. `ash check` and `ash run` can then consume the locked dependency roots through module-loader integration.

## Scenario: vendor dependencies for offline deployment

Materialize locked dependencies into the default project vendor directory:

```bash
ashgrove vendor --project /path/to/project
```

By default this writes `vendor/ash/<package>/` and provenance metadata. To use another output directory:

```bash
ashgrove vendor --project /path/to/project --output /path/to/vendor-root
```

Verify an existing vendor tree without writing or fetching:

```bash
ashgrove vendor --project /path/to/project --check
```

`vendor --check` is read-only for the vendor tree, but it still needs the matching locked dependency checkouts to be present in the Ash cache so it can compare package bytes and provenance. Run `ashgrove fetch` first when the cache is empty.

Vendoring preserves package metadata and validates provenance so a deployed project can use local dependency content reproducibly. The selected toolchain stdlib remains separate from third-party dependencies and keeps precedence over stdlib-shaped packages.

## Current non-goals and fail-closed boundaries

The MVP deliberately does not claim:

- system-wide/global installation roots;
- a hosted Ash package registry;
- arbitrary SemVer dependency solving;
- hosted release-channel discovery or bare version installs/updates;
- signed release-index-as-digest resolution;
- OS package-manager integration;
- editor plugin installation;
- automatic user-project rewriting during toolchain updates;
- independent stdlib updates decoupled from toolchain updates.

Commands that would require those policies fail closed rather than performing ad hoc network or registry behavior.
