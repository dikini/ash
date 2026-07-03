# PLAN-184: Handler / Provider Semantics

**Status:** Complete (8/8 tasks complete)
**Spec:** [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md); [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md); [SPEC-099b: Target Operational Semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md); [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Notes:** [NOTE-023: Handler Surface Dispatch](../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md); [NOTE-025: Effect Identity via Sorts and Impls](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md); [PLAN-183: Operation And Authority Model](PLAN-183-OPERATION-AUTHORITY-MODEL.md)
**Depends on:** [PLAN-183: Operation And Authority Model](PLAN-183-OPERATION-AUTHORITY-MODEL.md)
**Task range:** TASK-1855 through TASK-1862.

## Goal

Make the handler/provider operational model executable for the target row semantics: handler and provider frames discharge operation requirements, raise/handle dispatch uses frame-stack order, missing discharge fails closed, and nesting/shadowing are observable.

## Scope

- Add the Phase 184 plan/task packet.
- Audit existing CPS handler/provider behavior against target specs.
- Add admission-side frame proof metadata for operation requirements.
- Fix CPS dispatch so handler and provider frames are searched in one innermost-to-outermost pass.
- Add regressions for raise/handle behavior, provider discharge, missing discharge, and handler/provider shadowing.
- Reconcile specs and orientation indexes so rows point to operational handler/provider semantics.

## Non-goals

- No new surface `handler`, `on`, or `handle ... with` parser syntax.
- No full host FFI/provider API redesign.
- No row-polymorphic inference or monomorphization changes.
- No broad rewrite of the historical CPS interpreter beyond frame lookup and targeted tests.

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1855](tasks/TASK-1855-handler-provider-plan-packet.md) | Create the Phase 184 plan packet | Complete |
| [TASK-1856](tasks/TASK-1856-handler-provider-boundary-audit.md) | Audit handler/provider semantics boundaries | Complete |
| [TASK-1857](tasks/TASK-1857-admission-frame-proof-model.md) | Add admission frame proof model | Complete |
| [TASK-1858](tasks/TASK-1858-cps-frame-ordered-dispatch.md) | Fix CPS frame-ordered dispatch | Complete |
| [TASK-1859](tasks/TASK-1859-raise-handle-provider-regressions.md) | Add raise/handle/provider regressions | Complete |
| [TASK-1860](tasks/TASK-1860-missing-discharge-failure-diagnostics.md) | Define missing-discharge failures | Complete |
| [TASK-1861](tasks/TASK-1861-handler-provider-spec-reconciliation.md) | Reconcile handler/provider specs | Complete |
| [TASK-1862](tasks/TASK-1862-handler-provider-closeout.md) | Close out Phase 184 | Complete |

## Acceptance criteria

- [x] Target docs define handler and provider frames as operation-discharge frames.
- [x] Admission has an explicit proof model for operation rows discharged by handler/provider frames.
- [x] CPS raise dispatch searches handler/provider frames in one innermost-to-outermost order.
- [x] Missing handler/provider/provider authority fails closed with a structured unhandled-effect or admission failure.
- [x] Tests cover handler dispatch, provider dispatch, missing discharge, handler nesting, and provider/handler shadowing.
- [x] `CHANGELOG.md` records the phase.
- [x] Required docs, Rust, and changed-crate gates pass.

## Verification

```bash
cargo test -p ash-engine --test task_1857_admission_frame_proof_model
cargo test -p ash-interp --test task_1858_1859_handler_provider_semantics
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
cargo fmt --check
git diff --check
```
