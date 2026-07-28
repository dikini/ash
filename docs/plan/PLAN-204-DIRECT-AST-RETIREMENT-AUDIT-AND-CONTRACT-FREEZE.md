---
id: plan.204.direct-ast-retirement-audit-contract-freeze
title: Direct AST Retirement Audit and Contract Freeze
kind: plan
status: in_progress
authority: planning
owner: language-semantics
last_verified: 2026-07-28
---

# PLAN-204: Direct AST Retirement Audit and Contract Freeze

## Purpose

Freeze the finite removal scope for the direct AST interpreter before deleting it. This phase
does not change executable runtime behavior. It establishes the contract and evidence needed for
Phase 205 to leave one evaluator only:

```text
Surface Ash → checked Core → checked CPS → Engine executor → terminal envelope
```

`ash run`, daemon, `ash test`, and `ash repl` are clients of that path. Until the cutover, new or
unlisted independent AST/CPS evaluation is prohibited; manifest-listed legacy evaluator/oracle
uses remain migration debt, not approved architecture. Lean remains a separate deferred
formalization project, not a current Ash execution route.

## Boundaries and controls

- The audit is finite. It records the repository paths and symbols present at its frozen commit;
  it does not generate feature cases or claim a general direct-evaluator domain.
- The target Ash specification remains the complete domain. A wrapper, REPL behavior, or test
  behavior not defined by the amended target contracts is deferred, not implemented.
- Bounded implementation realizes exactly the target-spec domain—no less, no more. Each finite
  slice is declared explicitly and is never generated.
- Each implementation report retains the independent axes: `implemented`/`partial`/
  `not_implemented`; `proved`/`tested`/`none`; and `matches_spec`/`below_spec`.
- Planned tasks are not active semantic-task records. A task that stages semantic Rust work must
  first be promoted to **In progress**, add its rule-scoped record, coverage row, traceability
  links, and focused verification evidence in that same change.

### Lean preservation and handoff

Lean is preserved as a deferred, separate formalization project. The audit records its sources,
documentation, and links with disposition `deferred_separate_project`; it does not schedule their
deletion or treat their theorems as a current Ash execution route. Phase 204 removes or relabels
only a Lean reference that claims executable differential authority for current Ash. A later Lean
project must consume the same canonical target rules and establish any refinement bridge
separately.

## Task sequence

| Task | Outcome | Run-route impact |
|---|---|---|
| [TASK-2034](tasks/TASK-2034-direct-ast-retirement-audit-manifest.md) | Frozen manifest and enumerated deferred cases | none |
| [TASK-2035](tasks/TASK-2035-canonical-client-test-contracts.md) | Target contracts for Engine-only test and REPL clients | prerequisite |
| [TASK-2036](tasks/TASK-2036-direct-ast-reentry-guard.md) | Guard against new legacy evaluator reachability | prerequisite |

## Exit criteria

Phase 204 is complete only when the manifest has an immutable inventory digest, every entry has a
disposition and either a Phase-205 owner or explicit external-project handoff, all deferred cases
are explicitly catalogued, the target contracts authorize the selected exact source-wrapper and
REPL routes, and a guard blocks new legacy use. It is not a claim that any evaluator has been
removed.
