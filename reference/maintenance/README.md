---
id: ref.maintenance.index
title: Reference Maintenance Index
kind: index
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
    - docs/plan/tasks/TASK-993-reference-maintenance-metadata-and-staleness.md
  code:
    - tools/reference/check_frontmatter.py
    - tools/reference/check_staleness.py
  tests:
    - check_frontmatter full reference validation
    - check_staleness maintenance path audit
  examples:
    []
related:
  depends_on:
    - ref.meta
    - ref.methodology
  explains:
    - ref.maintenance.metadata
    - ref.maintenance.staleness
    - ref.maintenance.refresh
    - ref.maintenance.stale_triage
    - ref.maintenance.release_checklist
    - ref.maintenance.agent_cards
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - tools/reference/check_frontmatter.py changes
  - tools/reference/check_staleness.py changes
  - reference/maintenance/** changes
---

# Reference Maintenance Index

Maintenance pages define how the reference corpus stays current. Ordinary reference pages should link here when they need maintenance context, but they should not embed their own maintenance playbooks.

## Procedures

- [Metadata reference](metadata-reference.md): field meanings for Slice 2 frontmatter, evidence, declared status, and derived inspection state.
- [Staleness inspection](staleness-inspection.md): diff-based inspection from `verified_against.git_commit` to `HEAD`.
- [Refresh procedure](refresh-procedure.md): how to recheck evidence and update page metadata.
- [Stale-doc triage](stale-doc-triage.md): how to classify and handle stale, partial, or superseded pages.
- [Release checklist](release-checklist.md): reference closeout checks for phase or release boundaries.
- [Agent-card procedure](agent-card-procedure.md): how derivative cards stay linked to canonical reference pages.

## Tooling

Use [check_frontmatter.py](../../tools/reference/check_frontmatter.py) for SPEC-071 metadata validation. Use [check_staleness.py](../../tools/reference/check_staleness.py) for deterministic path-based staleness inspection; it is an aid, not a semantic proof.
