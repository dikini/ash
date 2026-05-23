---
id: ref.status.feature_matrix
title: Pilot Feature Matrix
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
    - docs/plan/tasks/TASK-952-reference-examples-and-status-classification.md
  code:
    []
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.language.functions
    - ref.language.act
    - ref.language.proc
    - ref.language.workflow
    - ref.language.generalized_do
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-071 changes
  - Phase closeout changes reference policy
---

# Pilot Feature Matrix

| Feature | Pilot status | Stability | Reference page | Evidence note |
| --- | --- | --- | --- | --- |
| Pure functions | current-partial | alpha | [functions](../language/functions.md) | Basic pure behavior only; not full language manual. |
| Act | current-partial | alpha | [Act](../language/effects-act.md) | Opaque runtime-managed effect; not Result. |
| Proc | current-partial | alpha | [Proc](../language/processes-proc.md) | Explicit tower crossing required. |
| Workflow | current-partial | alpha | [Workflow](../language/workflows.md) | Runtime admission boundary preserved. |
| Generalized do | current-partial | alpha | [generalized do](../language/generalized-do.md) | Evidence-driven `Monad<K>` lowering; no implicit lifts/final expr. |
| Reference metadata validator | current-pilot | alpha | [verification evidence](verification-evidence.md) | Frontmatter/path/link/ID checks for pilot pages. |
