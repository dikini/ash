---
id: ref.tools.ashgrove.project_dependencies
title: Ashgrove Project Dependencies
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
    - crates/ash-engine/src/lib.rs
    - crates/ash-cli/src/main.rs
  tests:
    - cargo run -p ashgrove -- lock --help
    - cargo run -p ashgrove -- fetch --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove
  explains:
    - ref.tools.ashgrove.vendor_deploy
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - crates/ashgrove/src/** changes
  - crates/ash-engine/src/** changes
  - reference/tools/ashgrove/project-dependencies.md changes
---

# Ashgrove Project Dependencies

Ashgrove's Alpha dependency model is git-pinned and lockfile-first. A project uses lower-case `ash.toml` for dependency intent and `ash.lock` for exact execution truth.

Live help checked:

```bash
cargo run -p ashgrove -- lock --help
cargo run -p ashgrove -- fetch --help
```

## Manifest and Lockfile Boundary

`ash.toml` records package identity, toolchain preference, and git dependencies. `ash.lock` records exact resolved commits and package metadata. Tags are intent; resolved commits are execution truth.

Illustrative manifest fragment, not a complete executable project:

```toml
[package]
name = "app"
version = "0.1.0"

[dependencies.dep]
git = "file:///path/to/dep.git"
tag = "v1"
```

Dependency entries are explicit git dependencies pinned by `tag` or `rev`. Unpinned or floating dependencies are rejected unless a later development override is explicitly specified by a future task. Hosted registry lookup and arbitrary SemVer solving are not implemented.

## Lock

Help-derived forms:

```bash
ashgrove lock
ashgrove lock --project PROJECT
ashgrove lock --project PROJECT --check
```

`lock` resolves accepted dependency references to exact commits and writes `ash.lock`. `--check` verifies that the existing lockfile still matches the manifest without silently accepting drift.

Lock rewrites preserve trust/signing metadata where the current schema supports preservation. Lock processing also enforces fail-closed trust/source boundaries, including untrusted git protocols and credential-bearing lockfile origins.

## Fetch

Help-derived forms:

```bash
ashgrove fetch
ashgrove fetch --project PROJECT
```

`fetch` reads the project manifest/lockfile pair and materializes exact git dependency checkouts in the Ash cache. Fetched checkout identity is commit-bound; missing or mismatched checkouts fail closed when consumed by `ash check` or `ash run`.

## Module Resolution

Locked dependencies become visible to `ash check` and `ash run` through project manifest plus lockfile discovery, direct fetched-cache checkouts, or vendored roots. The selected toolchain stdlib has precedence and is not fetched as an ordinary third-party dependency.

## Non-Goals

Project dependency handling does not provide a hosted registry, arbitrary SemVer solver, branch/channel resolver, broad source-ignore glob CLI, OS package-manager integration, or best-effort remote lookup. Unsupported dependency sources fail closed.
