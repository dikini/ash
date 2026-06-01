---
id: ref.tools.ashgrove.vendor_deploy
title: Ashgrove Vendor and Deploy
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
    - cargo run -p ashgrove -- vendor --help
    - cargo run -p ash-cli -- check --help
    - cargo run -p ash-cli -- run --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove.project_dependencies
  explains:
    - ref.tools.cli
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - crates/ashgrove/src/** changes
  - crates/ash-engine/src/** changes
  - crates/ash-cli/src/** changes
  - reference/tools/ashgrove/vendor-deploy.md changes
---

# Ashgrove Vendor and Deploy

`ashgrove vendor` materializes locked dependencies for offline deployment. It is a local project operation, not a hosted registry publish flow.

Live help checked:

```bash
cargo run -p ashgrove -- vendor --help
```

## Vendor

Help-derived forms:

```bash
ashgrove vendor
ashgrove vendor --project PROJECT
ashgrove vendor --project PROJECT --output OUTPUT
ashgrove vendor --project PROJECT --check
```

The default vendor directory is project-local `vendor/ash/`. Vendored packages include provenance metadata tying package bytes back to `ash.lock` entries.

`vendor --check` is read-only for the vendor tree. It verifies provenance and package bytes against the lockfile and required cache evidence rather than fetching or rewriting the vendor directory.

## Deployable Project Flow

Reference-only sequence. Paths and package contents are illustrative:

```bash
ashgrove lock --project PROJECT
ashgrove fetch --project PROJECT
ashgrove vendor --project PROJECT
ashgrove vendor --project PROJECT --check
ash check PATH
ash run PATH
```

The `ash check` and `ash run` command forms above are from live help. This page does not define a new project entrypoint shortcut; use explicit file paths until a later CLI/entrypoint spec says otherwise.

## Offline Boundary

A vendored project can rely on project-local `vendor/ash/` plus `ash.lock` evidence for dependency roots. The selected toolchain still provides the stdlib and runtime-support metadata. A vendored package shaped like stdlib must not override the selected toolchain stdlib.

## Fail-Closed Checks

Ashgrove and Ash consumers fail closed for malformed lock commits, missing fetched checkouts when required, checkout `HEAD` mismatch, package-name mismatch, provenance mismatch, untrusted git protocols, lock signature mismatch when required, and credential-bearing origins.

## Non-Goals

Vendoring does not implement a hosted package registry, publish protocol, deployment rollback service, arbitrary SemVer solver, global/system install path, OS package-manager integration, or broad source-ignore glob CLI.
