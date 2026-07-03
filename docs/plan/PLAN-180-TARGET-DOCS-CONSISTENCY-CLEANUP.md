# PLAN-180: Target Docs Consistency Cleanup

**Status:** ✅ Complete (1/1 tasks complete)
**Spec:** [SPEC-095b: Target Grammar](../spec/SPEC-095b-TARGET-GRAMMAR.md); [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md); [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md); [SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md); [SPEC-099b: Target Operational Semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md); [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Notes:** [NOTE-015: Current-to-Target Language Forms](../notes/NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md); [NOTE-018: Boundary Discipline](../notes/NOTE-018-BOUNDARY-DISCIPLINE.md); [NOTE-019: Target Ash Convergence Plan](../notes/NOTE-019-TARGET-ASH-CONVERGENCE-PLAN.md); [NOTE-022: Effects as Interfaces](../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md); [NOTE-023: Handler Surface Dispatch](../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md); [NOTE-025: Effect Identity via Sorts and Impls](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md)
**Depends on:** [PLAN-179: Explicit Row Admission Runtime Wiring](PLAN-179-EXPLICIT-ROW-ADMISSION-RUNTIME-WIRING.md)
**Task range:** TASK-1835.

## Goal

Reconcile stale target-Ash docs so future planning reads the interface/impl-qualified operation model, provider/handler admission model, and ambient workflow-fact model as current authority, while legacy `capability binding`, `effect` declaration, and `WorkflowForm` material is clearly marked as historical/current-state context.

## Scope

- Reclassify or fence NOTE-009-era capability/resource specs that still read as target-state authority.
- Fence protocol and operational-semantics docs that still expose legacy capability lookup/binding vocabulary without target routing.
- Replace stale target convergence prose that says current capabilities migrate to `effect` declarations with interface/impl operation declarations per NOTE-022/025.
- Fence WorkflowForm-era normative language in SPEC-056 so it cannot be mistaken for target-state workflow design.
- Refresh orientation indexes and read paths for target-Ash work.
- Record a focused stale-term scan and docs-gate evidence.

## Non-goals

- No Rust implementation changes.
- No complete rewrite of historical specs; this phase may add target reconciliation fences rather than deleting useful implementation history.
- No new target syntax decision beyond the existing NOTE-022/023/025 decisions.

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1835](tasks/TASK-1835-target-docs-consistency-cleanup.md) | Reconcile stale target-Ash specs and notes | ✅ Complete |

## Acceptance criteria

- [x] SPEC-052 and SPEC-053 no longer present NOTE-009 capability-binding vocabulary as target-state authority without a target reconciliation fence.
- [x] NOTE-011 and SPEC-004 no longer present legacy capability-binding/lookup vocabulary as target-state guidance without target routing.
- [x] NOTE-015, NOTE-018, and NOTE-019 use interface/impl operation vocabulary for target operation declarations.
- [x] SPEC-056 WorkflowForm language is clearly historical in the sections most likely to be cited by future implementation work.
- [x] SPEC-INDEX and NOTE-INDEX route target-Ash work toward current target specs and notes before historical material.
- [x] CHANGELOG records the cleanup.
- [x] Orientation index validation and docs gate pass.

## Verification

```bash
rg -n 'effect declarations|effect declaration|capability binding|WorkflowForm is the semantic source|WorkflowForm grammar preserves|Current capability declarations are subsumed' docs/spec docs/notes -g '*.md'
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```
