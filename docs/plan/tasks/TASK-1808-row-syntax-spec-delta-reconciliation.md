# TASK-1808: Reconcile target row/effect syntax deltas into implementation decisions

## Status: ✅ Complete

## Description

Turn the relevant pre-spec deltas from NOTE-021, NOTE-022, NOTE-023, and NOTE-025 into Phase 177 implementation decisions. This is a docs/spec alignment task that prevents implementation from following stale target-spec examples where later notes refined the design.

## Specification Reference

- [PLAN-177: Target Ash Row Syntax and Core/CPS Alignment](../PLAN-177-TARGET-ASH-ROW-SYNTAX-AND-CORE-CPS-ALIGNMENT.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-098c: Surface-to-Core Lowering](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md)
- [NOTE-021](../../notes/NOTE-021-ROW-CALLABLE-WHERE-AND-FACT-SYNTAX.md)
- [NOTE-022](../../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md)
- [NOTE-023](../../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md)
- [NOTE-025](../../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)

## Dependencies

- ✅ TASK-1806 planning packet exists.
- ✅ TASK-1807 provided the live seam audit before implementation decisions were finalized.

## Requirements

### Functional Requirements

1. Add a Phase 177 implementation-decision section to the plan or a compact audit/design artifact under `docs/audit/`.
2. Record that source kind spelling is `Row`, while older spec prose may still say `EffectRow`.
3. Record that inline rows and `where row { ... }` are alternate layouts for one callable row.
4. Record duplicate row spelling as an error.
5. Record that row items contain evidence requirements rather than raw predicate/law bodies for this phase.
6. Record that operation declarations use `interface`, while row operation identity is impl-type-qualified (`F::read`, `PosixFs::read`) per NOTE-025.
7. Record that `handler` is a marker and handler execution surface remains out of Phase 177 except for row/identity carriers.
8. Patch touched spec index/read paths only if new docs are added or authority wording changes.

### Property Requirements

- Do not rewrite target specs wholesale.
- Preserve caveats: this task aligns Phase 177 implementation decisions; it does not make target specs implemented.
- Any unresolved design question must become a fail-closed implementation gate or future-phase seed.

## TDD Steps

### Step 1: Compare target specs and notes

Read the "Pre-Spec Delta" sections of NOTE-021, NOTE-023, and NOTE-025 plus SPEC-095b/096b/097b/098b/098c row sections.

### Step 2: Write decision artifact

Record the bounded decisions Phase 177 will implement and the design surfaces it will not implement.

### Step 3: Patch downstream task wording

Update TASK-1809 through TASK-1814 if any task uses stale interface-qualified operation identities or treats rows as authority.

## Dispatch

```yaml
agent: codex
reasoning: medium
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - python3 tools/docs/validate_orientation_indexes.py --self-test
  - bash scripts/check-docs-gate.sh
checklist:
  - [x] Phase 177 implementation decisions are recorded.
  - [x] Row/EffectRow terminology boundary is explicit.
  - [x] Impl-qualified operation identity is explicit.
  - [x] Handler execution remains out of scope except for row carriers.
```

## Dependencies for Next Task

This task feeds TASK-1809 through TASK-1814.

## Completion Evidence

- Recorded Phase 177 implementation decisions in [PHASE-177-row-syntax-core-cps-seams.md](../../audit/PHASE-177-row-syntax-core-cps-seams.md), including `Row` terminology, duplicate row errors, evidence row requirements, impl-qualified operation identities, and handler execution scope.
- Preserved the caveat that this task reconciles Phase 177 decisions without declaring target specs implemented.
