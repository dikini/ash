---
id: ref.tools.ashgrove
title: Ashgrove Reference Target
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
    - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md
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
    - ref.tools.index
  explains:
    - ref.tools.ashgrove.install
    - ref.tools.ashgrove.update
    - ref.tools.ashgrove.remove_cleanup
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md changes
  - docs/spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md changes
  - crates/ashgrove/src/** changes
  - reference/tools/ashgrove/** changes
---

# Ashgrove Reference Target

Ashgrove is the Alpha toolchain and local deployment manager. This draft page exists as a stable link target for TASK-994; TASK-995 owns the full Ashgrove reference.

## Current Procedure Targets

- [Install](ashgrove/install.md)
- [Update](ashgrove/update.md)
- [Remove and cleanup](ashgrove/remove-cleanup.md)

## Limitation

This page is not the complete Ashgrove manual. It must not be read as support for hosted registry service, global/system install roots, OS package-manager integration, arbitrary SemVer solving, or unsigned release-index lookup.
