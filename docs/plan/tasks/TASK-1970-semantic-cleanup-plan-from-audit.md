# TASK-1970: Semantic Cleanup Plan From Audit

**Status:** Complete
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Description

Elaborate a detailed deletion/refactor plan from TASK-1969's semantic-removal audit and the target
Ash specs. The plan must turn every unproven retained stale mechanism into concrete cleanup work so
the repository converges on target Ash: ordinary functions with effect rows, target row admission,
provider profiles, contract/evidence helpers, process/channel primitives, application runtime
reports, and current docs/tests.

The output is a plan, not the implementation. It must be detailed enough that later code cleanup
tasks can remove stale code instead of repeating rename-only work.

## Requirements

- Read TASK-1969's semantic-removal audit and all rows marked `Delete now`,
  `Refactor to target primitive`, or `Plan required`.
- Reconcile every row with target authority:
  - `SPEC-095b` target grammar,
  - `SPEC-096b` target effect system,
  - `SPEC-097b` target type system,
  - `SPEC-098c` surface-to-Core lowering,
  - Phase 185/186 function-first entry plans,
  - Phase 177 through 179 row/effect/admission plans,
  - Phase 194 contracts/evidence,
  - Phase 195 process/concurrency,
  - Phase 196 application runtime,
  - Phase 198/199 libraries/templates,
  - Phase 200 tooling/migration polish.
- Produce a detailed implementation plan, preferably
  `docs/plan/PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md`, unless the work is integrated into
  `PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md`.
- The plan must include task breakdowns for code, tests, docs, and gates. Each task must state:
  - stale mechanism to remove,
  - target replacement,
  - files/modules likely affected,
  - tests that must be deleted, rewritten, or added,
  - docs/reference changes,
  - risk and sequencing constraints.
- The plan must specifically address the function-entry unification concern: if functions with
  effect rows are target Ash entries, any separate callable-entry/workflow registry must be removed
  or reduced to an internal function metadata/cache with no separate semantic category.
- The plan must distinguish implementation details from target concepts. Names in public APIs,
  docs, diagnostics, tests, and productive examples must not teach stale distinctions.

## Required Plan Sections

The follow-up plan must include:

- **Goal:** no stale code, no stale docs, no stale tests, no rename-only completion claims.
- **Target Invariants:** functions/effect rows stand in for entries; old workflow/tower forms are
  not current Ash; provider/profile/row/evidence/runtime mechanisms are the only retained target
  abstractions.
- **Workstreams:** parser/lowering, type/effect/Core/TCIR/AMIR, runtime/engine/interpreter,
  tooling/LSP/formatter/CLI/templates, docs/reference/examples/tests.
- **Deletion Tasks:** concrete tasks for each stale mechanism that can be removed.
- **Refactor Tasks:** concrete tasks for mechanisms that must be folded into target primitives.
- **Documentation Tasks:** tasks that rewrite or delete stale docs/tests instead of renaming terms.
- **Gates:** fail-closed searches/tests that prove old mechanisms cannot re-enter.
- **Closeout Audit:** requirement-by-requirement proof that Phase 201 removed functionality, not
  just vocabulary.

## Completion Checklist

- [x] Follow-up semantic cleanup plan exists and is indexed.
- [x] Every TASK-1969 unproven retained mechanism is assigned to a deletion/refactor/docs/gate task.
- [x] The plan includes concrete work for function/effect-row unification and callable-entry
      registry cleanup.
- [x] The plan identifies tests/docs that should be removed or rewritten because they preserve old
      semantics.
- [x] The plan's acceptance criteria require proof of behavior removal.
- [x] `CHANGELOG.md` records the plan addition.
- [x] Docs/index verification passes.

## Evidence

- Added [PLAN-201 Semantic Cleanup Follow-up](../PLAN-201-SEMANTIC-CLEANUP-FOLLOWUP.md), which
  converts the TASK-1969 audit findings into parser/lowering, Core/TCIR/AMIR, runtime, tooling,
  docs, and gate workstreams.
- The plan assigns concrete follow-up tasks for callable-entry registry cleanup, child-entry
  registry cleanup, entry projection, entry artifacts, contract/evidence integration, ambient
  effects, documentation quarantine, and behavior-removal gates.
- Verification recorded in this change:
  `python3 tools/docs/validate_orientation_indexes.py --self-test`;
  `bash scripts/check-docs-gate.sh`;
  `git diff --check`.
