---
id: ref.getting_started.cleanup
title: Clean Up Ash State
kind: guide
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 598a8f6
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ashgrove/src/main.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.getting_started.index
    - ref.tools.ashgrove.remove_cleanup
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/remove-cleanup.md changes
---

# Clean Up Ash State

Use Ashgrove for local toolchain removal and cleanup planning. The cleanup model is intentionally conservative: project files and lockfiles are not deleted just because cleanup runs.

Use the [Ashgrove remove and cleanup detail page](../tools/ashgrove/remove-cleanup.md) for exact command forms, dry-run behavior, protected toolchains, and cache cleanup boundaries.

## Current Boundaries

Do not treat cleanup as a registry garbage collector, remote deployment rollback, or project rewrite tool. Broader practical deployment advice remains future scope.
