---
id: ref.methodology
title: Reference Methodology
kind: methodology
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
    - ref.authority
  explains:
    - ref.status.verification_evidence
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Reference Methodology

Reference pages are written from evidence outward. The author checks current specs, live stdlib/code paths, tests, examples, task closeouts, and known limitations before making a claim.

Claims use current tense only when evidence is current. Older design or example material is linked as historical rationale. Proposed or non-executable material is labeled as aspirational, historical, or reference-only.

Pilot refresh triggers are intentionally broad: changes to SPEC-069, SPEC-070, SPEC-071, public `std/src/{act,proc,workflow,result}.ash`, generalized do lowering, RuntimeKernel admission, examples cited by the page, or agent-card policy.
