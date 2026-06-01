---
id: ref.status.index
title: Reference Status Index
kind: status
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-06-01
verified_against:
  git_commit: 710340f
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-948-reference-skeleton-authority-methodology-style.md
    - docs/plan/tasks/TASK-993-reference-maintenance-metadata-and-staleness.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
    - docs/plan/tasks/TASK-996-reference-runtime-kernel-pages.md
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
    - docs/plan/tasks/TASK-999-reference-slice-2-closeout.md
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
    - reference/status/alpha-limitations.md
    - ref.status.known_limitations
    - ref.status.drift_report
    - ref.status.verification_evidence
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md changes
  - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md changes
  - reference/status/** changes
  - reference/maintenance/** changes
  - reference/stdlib/** changes
  - Phase closeout changes reference policy
---

# Reference Status Index

Status pages record what the pilot claims and what it does not claim.

- [Feature matrix](feature-matrix.md): pilot concept status.
- [Ashgrove status](ashgrove.md): current Alpha toolchain-manager claims, non-goals, and fail-closed boundaries.
- [RuntimeKernel status](runtime-kernel.md): current Alpha runtime host-mode, artifact, admission, daemon, and policy-profile evidence.
- [Alpha limitations](alpha-limitations.md): current Reference Slice 2 alpha limits.
- [Known limitations](known-limitations.md): retained Phase 124 pilot alias for older links.
- [Drift report](drift-report.md): mismatches, caveats, and next-slice recommendations.
- [Verification evidence](verification-evidence.md): R71/A75 acceptance evidence and command results.
- [Reference maintenance](reference-maintenance.md): Slice 2 metadata, staleness, and refresh status links.
