---
id: ref.tools.ashgrove.remove_cleanup
title: Ashgrove Remove and Cleanup Reference Target
kind: reference
audience: [human, agent]
authority: draft
status: draft
stability: alpha
owner: ashgrove
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
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
  code:
    - crates/ashgrove/src/lib.rs
    - crates/ashgrove/src/main.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.ashgrove
  explains:
    - ref.getting_started.cleanup
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/remove-cleanup.md changes
---

# Ashgrove Remove and Cleanup Reference Target

This draft page preserves a valid cleanup-detail link for [Clean up Ash state](../../getting-started/cleanup.md). TASK-995 owns the complete remove and cleanup procedure.

## Current Boundary

Cleanup is conservative and local. It is not a project rewrite mechanism, remote deployment rollback, or hosted registry garbage collector.
