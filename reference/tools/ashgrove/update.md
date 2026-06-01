---
id: ref.tools.ashgrove.update
title: Ashgrove Update
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
    - cargo run -p ashgrove -- update --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove
  explains:
    - ref.getting_started.update
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/update.md changes
---

# Ashgrove Update

`ashgrove update` installs a new immutable toolchain and optionally switches the default. It does not patch an existing installed toolchain in place.

Live help checked:

```bash
cargo run -p ashgrove -- update --help
```

Help-derived form:

```bash
ashgrove update [OPTIONS] [BARE_VERSION]
```

`BARE_VERSION` is rejected until a release-index policy exists.

## Source Update

Reference-only command form:

```bash
ashgrove update --to TO --from source --path PATH --switch
```

`--to TO` names the requested target toolchain identity. The source payload identity must match the requested target before publish. Source update uses the same source-payload/local-state rules as source install: ignored local state is excluded, nonignored source mutation remains fail-closed, and broad user-supplied ignore globs are not supported.

## Tarball Update

Reference-only command forms:

```bash
ashgrove update --to TO --from tarball --path PATH --digest DIGEST
ashgrove update --to TO --from tarball --url URL --digest DIGEST --switch
```

Tarball update uses the same tarball validation, safe extraction, digest, runtime-support, and trust boundaries as tarball install. Explicit-digest local `file://` URL updates are the supported URL boundary. Hosted release-channel discovery and unsigned/unbound release indexes fail closed.

## Selector Behavior

Without `--switch`, update preserves the existing default. With `--switch`, the user default changes only after the new toolchain is installed and verified.

Update does not rewrite project source files, update third-party dependencies, remove older toolchains, or update the stdlib independently. The bundled stdlib moves with the selected toolchain.

## Fail-Closed Boundaries

Update fails before publish if the target identity does not match the payload, required source/archive/tarball/runtime-support/trust metadata is missing, a digest or signature check fails, source payload mutates, or the requested flow requires an unsupported hosted release-index/channel policy.

## Non-Goals

Update does not implement hosted registry updates, global/system updates, OS package-manager integration, arbitrary SemVer solving, broad source-ignore glob CLI, or automatic user-project rewrites.
