---
id: ref.tools.ashgrove.list_current_default
title: Ashgrove List Current and Default
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
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ashgrove/src/main.rs
  tests:
    - cargo run -p ashgrove -- list --help
    - cargo run -p ashgrove -- current --help
    - cargo run -p ashgrove -- default --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove
  explains:
    - ref.getting_started.install
    - ref.getting_started.update
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/list-current-default.md changes
---

# Ashgrove List Current and Default

These commands inspect and set local toolchain selectors. They do not install, update, remove, or rewrite project files.

Live help checked:

```bash
cargo run -p ashgrove -- list --help
cargo run -p ashgrove -- current --help
cargo run -p ashgrove -- default --help
```

## List

Help-derived form:

```bash
ashgrove list
```

`list` reports installed toolchains from Ashgrove's user-local toolchain root. Broken or incomplete toolchain metadata is treated as invalid rather than silently accepted.

## Current

Help-derived forms:

```bash
ashgrove current
ashgrove current --project PROJECT
```

`current` prints the selected toolchain. With `--project PROJECT`, project toolchain selection participates in the result. Without `--project`, the command reports the user/default selection surface.

## Default

Help-derived form:

```bash
ashgrove default TOOLCHAIN_ID
```

`default` sets the user default to an installed exact toolchain id. It is selector-only: it does not rewrite project manifests and does not mutate the selected toolchain directory.

## Selection Order

Launcher selection is fail-closed and ordered:

1. explicit `ASH_TOOLCHAIN` override;
2. project pin in `ash.toml`;
3. user default selector.

If the selected toolchain is missing, incomplete, or unsafe, the launcher fails rather than falling back to an unrelated toolchain.

## Non-Goals

Selector commands do not solve arbitrary SemVer dependency ranges from a registry, install missing toolchains from a hosted channel, update stdlib independently, rewrite projects, or integrate with global/system package managers.
