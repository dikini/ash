# PLAN-183: Operation And Authority Model

**Status:** Complete (8/8 tasks complete)
**Spec:** [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md); [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md); [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md); [SPEC-098c: Surface-to-Core Lowering](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md); [SPEC-099b: Target Operational Semantics](../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md); [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
**Notes:** [NOTE-022: Effects as Interfaces](../notes/NOTE-022-EFFECTS-AS-INTERFACES-DECLARATION-SIDE.md); [NOTE-023: Handler Surface Dispatch](../notes/NOTE-023-HANDLER-SURFACE-DISPATCH-SIDE.md); [NOTE-025: Effect Identity via Sorts and Impls](../notes/NOTE-025-EFFECT-IDENTITY-VIA-SORTS-AND-IMPLS.md); [PLAN-179: Explicit Row Admission Runtime Wiring](PLAN-179-EXPLICIT-ROW-ADMISSION-RUNTIME-WIRING.md); [PLAN-182: Core Computation Model Conformance](PLAN-182-CORE-COMPUTATION-MODEL-CONFORMANCE.md)
**Depends on:** [PLAN-182: Core Computation Model Conformance](PLAN-182-CORE-COMPUTATION-MODEL-CONFORMANCE.md)
**Task range:** TASK-1847 through TASK-1854.

## Goal

Make the target operation and authority model explicit and executable for a bounded slice: operations are interface methods, operation identity is impl/type-qualified, rows require operations without granting authority, and discharge paths are modeled separately for operation, resource, role, policy, evidence, and failure rows.

## Scope

- Add the Phase 183 plan/task packet.
- Audit existing target specs and implementation seams for operation identity and authority discharge.
- Reconcile target docs so operation authority comes before richer syntax.
- Introduce an admission-side discharge model that distinguishes operation authority from resource, role, policy, evidence, failure, and unsupported row families.
- Preserve impl/type-qualified operation identity (`PosixFs::read`, `F::read`) through row metadata and admission diagnostics.
- Add tests proving rows do not grant authority and that each supported/unsupported family reports its own discharge path.

## Non-goals

- No full handler execution surface or `handle ... with` runtime implementation.
- No row-polymorphic inference or specialization of generic operation identities beyond existing typechecker validation.
- No broad standard-library or example migration.
- No compatibility behavior that makes legacy provider/capability vocabulary the target authority model.

## Tasks

| Task | Title | Status |
|---|---|---|
| [TASK-1847](tasks/TASK-1847-operation-authority-plan-packet.md) | Create the Phase 183 plan packet | Complete |
| [TASK-1848](tasks/TASK-1848-operation-authority-boundary-audit.md) | Audit operation authority boundaries | Complete |
| [TASK-1849](tasks/TASK-1849-operation-authority-spec-reconciliation.md) | Reconcile operation authority specs | Complete |
| [TASK-1850](tasks/TASK-1850-admission-discharge-model.md) | Add admission discharge model | Complete |
| [TASK-1851](tasks/TASK-1851-impl-qualified-operation-authority-fixtures.md) | Add impl/type-qualified operation authority fixtures | Complete |
| [TASK-1852](tasks/TASK-1852-row-family-discharge-diagnostics.md) | Separate row-family discharge diagnostics | Complete |
| [TASK-1853](tasks/TASK-1853-operation-authority-non-grant-regressions.md) | Prove rows do not grant authority | Complete |
| [TASK-1854](tasks/TASK-1854-operation-authority-closeout.md) | Close out Phase 183 | Complete |

## Acceptance criteria

- [x] Target docs state that operations are interface methods and row identities are impl/type-qualified.
- [x] Admission-side code names operation identity/discharge without legacy provider/capability wording as the target model.
- [x] Operation row requirements discharge only through existing registered operation authority; rows never register authority.
- [x] Resource, role, policy, evidence, and failure rows have distinct admission discharge families and diagnostics.
- [x] Tests cover impl/type-qualified operation rows and separate row-family discharge outcomes.
- [x] `CHANGELOG.md` records the phase.
- [x] Required docs, Rust, and changed-crate gates pass.

## Verification

```bash
cargo test -p ash-engine task_1850
cargo test -p ash-engine task_1851
cargo test -p ash-engine task_1852
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
cargo fmt --check
git diff --check
```
