---
id: ref.tools.index
title: Tools Reference Index
kind: index
audience: [human, agent]
authority: draft
status: draft
stability: alpha
owner: toolchain
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
    - crates/ash-cli/src/main.rs
    - crates/ashgrove/src/main.rs
  tests:
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.index
  explains:
    - ref.tools.cli
    - ref.tools.ashgrove
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - crates/ash-cli/src/** changes
  - crates/ashgrove/src/** changes
  - reference/tools/** changes
---

# Tools Reference Index

This is a draft detail-target index created so the getting-started journey can link to tool references. TASK-995 owns the complete `ash` and `ashgrove` tool pages.

## Current Targets

- [CLI tools](cli.md)
- [Ashgrove](ashgrove.md)
- [Ashgrove install](ashgrove/install.md)
- [Ashgrove update](ashgrove/update.md)
- [Ashgrove remove and cleanup](ashgrove/remove-cleanup.md)

## Limitation

This page is not a full command manual yet. It must not be used as evidence for hosted registries, OS package-manager integration, global installs, or deployment advice beyond the linked specs and later TASK-995 work.
