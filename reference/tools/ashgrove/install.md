---
id: ref.tools.ashgrove.install
title: Ashgrove Install
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: ashgrove
last_verified: 2026-06-01
verified_against:
  git_commit: e06944a
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ashgrove/src/main.rs
  tests:
    - cargo run -p ashgrove -- install --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove
  explains:
    - ref.getting_started.install
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/install.md changes
---

# Ashgrove Install

`ashgrove install` installs one coherent user-local Ash toolchain. It does not install into global/system roots and does not delegate to an OS package manager.

Live help checked:

```bash
cargo run -p ashgrove -- install --help
```

Help-derived form:

```bash
ashgrove install [OPTIONS] [BARE_VERSION]
```

`BARE_VERSION` is intentionally rejected until a release-index policy exists. Use explicit source or tarball evidence instead.

## Source Install

Reference-only command form from live help:

```bash
ashgrove install --from source --path PATH --switch
```

Source installs build from a local source root or source archive-shaped directory into an isolated build/staging area, then publish an immutable toolchain if validation succeeds.

Important flags:

| Flag | Meaning |
| --- | --- |
| `--from source` | Select the source install path. |
| `--path PATH` | Source root or source archive-shaped directory. |
| `--version VERSION` | Optional expected version/identity input. |
| `--rev REV` | Optional source revision evidence. |
| `--allow-dirty-source` | Explicitly accept nonignored dirty source payload and mark the install non-reproducible. |
| `--allow-unidentified-source` | Explicitly accept missing source identity and mark the install non-reproducible. |
| `--switch` | Switch the user default after successful install. |

Source-root installs preserve the SPEC-074 payload boundary. Gitignored and known local-state files are excluded from source-root payload digest and isolated build copy. Nonignored source changes remain fail-closed unless `--allow-dirty-source` is explicit. See [Source payload and local state](source-payload.md).

Source archives remain governed by source-archive metadata and attestation rules. Ashgrove must not weaken source-archive integrity by treating arbitrary archives as developer checkouts with broad ignore rules.

## Tarball Install

Reference-only command form from live help:

```bash
ashgrove install --from tarball --path PATH --digest DIGEST --switch
```

Local tarball installs validate archive shape, schema, identity, executable bits, stdlib metadata, runtime-support metadata, and digest evidence before publishing.

`--url URL` is available for explicit-digest tarball URL installs. The implemented URL boundary is explicit-digest local `file://` tarball evidence; unsupported network lookup and unsigned/unbound release-index flows fail closed.

Reference-only `file://` form:

```bash
ashgrove install --from tarball --url URL --digest DIGEST --switch
```

## Installed Results

A successful install creates an immutable toolchain under the user-local XDG data tree and installs or refreshes stable launcher shims. Installed toolchains include `ash`, `ashgrove`, stdlib payload, runtime-support metadata, manifest metadata, and install metadata.

First install may initialize the default selector. Later installs preserve the existing default unless `--switch` is passed or `ashgrove default TOOLCHAIN_ID` is run separately.

## Fail-Closed Boundaries

Install fails before publish for missing required metadata, unsafe tarball entries, digest mismatch, dirty nonignored source without override, unidentified source without override, missing source-archive attestation when required, missing required tarball signature evidence, unsigned/unbound release-index metadata, and unsupported network/release-channel lookup.

## Non-Goals

Install does not provide a hosted registry, global/system install, OS package-manager integration, arbitrary SemVer resolver, broad source-ignore glob CLI, or independent stdlib update path.
