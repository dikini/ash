---
id: ref.status.index
title: Reference Status Index
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-05-23
verified_against:
  git_commit: ff1f98f
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-948-reference-skeleton-authority-methodology-style.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.index
  explains:
    - ref.status.feature_matrix
    - ref.status.known_limitations
    - ref.status.drift_report
    - ref.status.verification_evidence
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Reference Status Index

Status pages record what the pilot claims and what it does not claim.

- [Feature matrix](feature-matrix.md): pilot concept status.
- [Known limitations](known-limitations.md): current alpha and reference-pilot limits.
- [Drift report](drift-report.md): mismatches, caveats, and next-slice recommendations.
- [Verification evidence](verification-evidence.md): R71 acceptance evidence and command results.
