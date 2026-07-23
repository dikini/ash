---
id: ref.methodology
title: Reference Methodology
kind: methodology
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
    - ref.authority
  explains:
    - ref.status.verification_evidence
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/maintenance/** changes
  - Phase closeout changes reference policy
---

# Reference Methodology

Reference pages are written from evidence outward. The author checks current specs, live stdlib/code paths, tests, examples, task closeouts, and known limitations before making a claim.

Claims use current tense only when evidence is current. Older design or example material is linked as historical rationale. Proposed or non-executable material is labeled as aspirational, historical, or reference-only.

Pilot refresh triggers are intentionally broad: changes to SPEC-070, SPEC-071, current pure-data and algebra stdlib sources, checked-function lowering, RuntimeKernel admission, examples cited by the page, or agent-card policy.

Slice 2 maintenance procedures live under [Reference Maintenance](maintenance/README.md), with separate pages for [metadata](maintenance/metadata-reference.md), [staleness inspection](maintenance/staleness-inspection.md), and [refresh](maintenance/refresh-procedure.md).
