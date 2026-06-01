---
id: ref.meta
title: Reference Metadata Contract
kind: guide
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 4fa1eba
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-948-reference-skeleton-authority-methodology-style.md
    - docs/plan/tasks/TASK-993-reference-maintenance-metadata-and-staleness.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.root
  explains:
    - ref.authority
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/maintenance/metadata-reference.md changes
  - Phase closeout changes reference policy
---

# Reference Metadata Contract

Every Markdown page in `reference/` carries SPEC-071 frontmatter. Required fields identify the page, audience, authority class, lifecycle status, stability, owner, verification sources, related records, and refresh triggers.

Use repo-relative paths inside `verified_against`. Use `ref.*` IDs in `related` when pointing to reference pages. Use `historical_rationale` for old plans or design notes that explain why a feature exists but are not current authority.

The pilot validator is `tools/reference/check_frontmatter.py --pilot`.

Slice 2 maintenance semantics are defined in [Reference Maintenance Metadata](maintenance/metadata-reference.md). Diff-based freshness inspection is defined in [Reference Staleness Inspection](maintenance/staleness-inspection.md).
