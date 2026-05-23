---
id: ref.examples.index
title: Reference Example Classification
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
    - examples/07-phase105/01-do-act.ash
    - examples/07-phase105/03-do-proc-from-act.ash
    - examples/08-phase106/03-deferred-pure-targets.ash
    - examples/09-phase108/01-do-workflow-unit.ash
    - examples/09-phase108/04-workflow-explicit-lifts.reference.ash
    - examples/09-phase108/06-legacy-workflow-migration-warning.ash
related:
  depends_on:
    - ref.status.feature_matrix
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

# Reference Example Classification

Labels used by the pilot:

- normative-pass: current passing evidence for the cited behavior.
- illustrative-pass: a small example believed to pass for the illustrated shape, but not a full normative contract.
- expected-fail: an example whose failure or diagnostic is the point.
- aspirational: a future-direction sketch; do not cite as current behavior.
- historical: older corpus material preserved as history or migration context.
- reference-only: explanatory material not claimed as executable.

| Example | Label | Used by | Note |
| --- | --- | --- | --- |
| examples/09-phase108/01-do-workflow-unit.ash | normative-pass | Workflow | Current pilot workflow do shape. |
| examples/07-phase105/01-do-act.ash | illustrative-pass | Act | Small Act/do example; page avoids broader claims. |
| examples/08-phase106/03-deferred-pure-targets.ash | expected-fail | Generalized do | Records deferred pure/general target behavior. |
| examples/09-phase108/02-do-workflow-contract-statements.ash | illustrative-pass | Generalized do | Workflow contract statement shape. |
| examples/04-real-world/customer-support.ash | aspirational | none in pilot | Do not use as current semantic authority. |
| examples/01-basics/03-expressions.ash | historical | Functions | Basic historical corpus example. |
| examples/09-phase108/04-workflow-explicit-lifts.reference.ash | reference-only | Workflow | Shows explicit lift shape without executable claim. |
| examples/09-phase108/06-legacy-workflow-migration-warning.ash | historical | Workflow | Migration warning context. |
