# PLAN-181: Legacy Authority Vocabulary Audit

**Status:** ✅ Complete (1/1 tasks complete)
**Spec:** [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md); [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md); [SPEC-099b: Target Operational Semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md); [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Notes:** [NOTE-022: Effects as Interfaces](../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md); [NOTE-023: Handler Surface Dispatch](../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md); [NOTE-025: Effect Identity via Sorts and Impls](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md); [PLAN-180: Target Docs Consistency Cleanup](PLAN-180-TARGET-DOCS-CONSISTENCY-CLEANUP.md)
**Depends on:** [PLAN-180: Target Docs Consistency Cleanup](PLAN-180-TARGET-DOCS-CONSISTENCY-CLEANUP.md)
**Task range:** TASK-1836.

## Goal

Audit older authority and capability/provider specifications so target-Ash planning can distinguish current-state compatibility references from superseded historical design. Target correctness is the priority; compatibility language is retained only when it describes current implementation evidence or a deliberate legacy input to target lowering/admission.

## Scope

- Scan specs and notes for legacy authority vocabulary such as `capability/provider`, `provider/capability`, `capability binding`, `capability invocation`, and capability-availability/admissibility wording.
- Classify affected older specs as one of:
  - target-state authority;
  - current-state compatibility / implemented substrate;
  - superseded historical reference;
  - deferred background.
- Add reconciliation fences where an older document remains useful but must not govern target-Ash authority design.
- Update orientation indexes so target work routes through NOTE-022/023/025 and SPEC-096b/097b/098b/099b/100 before older compatibility material.
- Record audit findings and verification evidence.

## Non-goals

- No Rust implementation changes.
- No compatibility work that delays or weakens the target provider/handler admission and operation-identity model.
- No deletion or full rewrite of historical specs unless required to prevent target confusion.
- No new target syntax decision beyond existing target specs and notes.

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1836](tasks/TASK-1836-legacy-authority-vocabulary-audit.md) | Classify legacy authority vocabulary docs | ✅ Complete |

## Acceptance criteria

- [x] Legacy authority vocabulary scan is recorded and reviewed.
- [x] Older specs with active-looking capability/provider vocabulary are classified as current-state compatibility, superseded historical, target-state, or deferred background.
- [x] High-risk older specs include target reconciliation fences or index routing so they cannot override target-Ash authority design.
- [x] `SPEC-INDEX.md` and `NOTE-INDEX.md` route authority work to current target docs first.
- [x] `CHANGELOG.md` records the audit.
- [x] Orientation index validation and docs gate pass.

## Verification

```bash
rg -n 'capability/provider|provider/capability|capability binding|capability bindings|CapabilityBinding|missing capability|capability invocation|capability availability|policy/capability|capability admissibility' docs/spec docs/notes -g '*.md'
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```
