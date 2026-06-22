# Core Ash Lowering

This page documents the implemented Phase 161/163 Core-to-CPS lowering boundary. It describes the Rust implementation in `ash-core::core_ash_lower`, not the whole SPEC-099 design space.

Core-to-CPS lowering consumes validated Core from `ash-core::core_ash_validate` and produces the existing `ash-core::cps::Term` tree. Raw parsed `.core` files must pass validation first; the lowering API is intentionally shaped around validated Core.

## Implemented Input Boundary

The lowering pass handles the Phase 161/163 fixture subset:

- atoms and literal values;
- lambda, record, tuple, and discharge-marker values where the current CPS carrier can represent them;
- `let-val`, `let-rec`, `let-prim`, `let-call`, `if`, tail `call`, and `jump`;
- capability, channel, process, and failure `raise` operations;
- single-clause `handle` with affine or legal multi-shot-pure resume metadata;
- `record-discharge`;
- `trap`;
- `let-mode` and `force` mode forms;
- `thunk` values.

The fixture examples live under `crates/ash-core/tests/fixtures/core/`:

- `call_non_tail.core` shows a direct-style tail call.
- `let_call.core` shows a non-tail direct-style call.
- `raise_handle.core` shows operation raising and handling.
- `contract_trap.core` shows contract discharge plus trap lowering.
- `mode_forms.core` shows `let-mode`, `thunk`, and `force` lowering.

Each valid fixture has a matching `.cps.golden` file checked by `task_1629_core_end_to_end.rs`.

Mode-aware fixtures and mode-specific lowering are also exercised by `task_1669_core_mode_lowering.rs`, `task_1670_core_thunk_capture_authority.rs`, and `task_1672_core_mode_tracing_docs_consistency.rs`.

## Lowering Rules

Pure bindings lower structurally:

- Core `LetVal` lowers to CPS `LetVal`.
- Core `LetRec` lowers to CPS `LetRec`.
- Core `LetPrim` lowers to CPS `LetPrim`.
- Mode `let-mode` forms are lowered to strict binding paths for `strict` and thunk binding paths for `lazy`/`memo`.
- Core `If` lowers to CPS `If`; the CPS `If.row` is the union of local branch rows, not the current continuation row.
- Core `Jump` lowers to CPS `Jump`; its row is the target continuation row.
- `thunk` values lower to `Value::ThunkClosure`.
- `force` lowers to `PrimOp::ForceThunk` with the checked force result name/row.

Tail calls lower to CPS `Call` with the current continuation. The call row is the union of the callee body row and the current continuation row, matching SPEC-098b's call-row split.

Non-tail direct-style calls lower through CPS `LetCont`. Core `LetCall` creates a fresh continuation label for the rest of the Core expression, then emits a CPS `Call` that resumes at that label. `let_call.core` is the smallest committed fixture for this path.

Core lambda values lower to CPS lambdas by adding a fresh continuation parameter. A Core lambda body that is an atom becomes a CPS jump to that continuation parameter.

## Effects, Handlers, And Contracts

Core `Raise` lowers to CPS `Raise` with the current continuation as `resume`. The CPS `Raise.row` contains the raised operation's local operation row only.

Core `Handle` lowers to CPS `Handle` with the current continuation in `Handle.cont`. `Handle.row is local residual row`: it comes from the handler clause row and excludes the outer continuation row. The total behavior is accounted for by combining this local residual row with the continuation row at the surrounding call/continuation boundary.

Handler resume parameters lower to CPS handler metadata. Checked lowering emits
`HandlerClause.resume_row = Known(row)` and maps Core `affine`/`multi-shot-pure` to CPS
`Affine`/`MultiShotPure`. Core `(let-cont-call name cont-ref atom body)` lowers to CPS
`Term::LetContCall` with the checked continuation row. See
[`core-cps-continuation-multiplicity.md`](core-cps-continuation-multiplicity.md) for the Phase 164
reference behavior.

Contract discharge lowers to CPS `RecordDischarge`. `ContractViolation is trap metadata`: dynamic contract failure lowers to `Trap { reason: ContractViolation }` under the discharge record, not to a contract row item and not to a raised operation. Recoverable behavior must use an explicit failure operation upstream.

Mode lowering preserves thunk semantics explicitly:

- `Value::ThunkClosure` carries the mode (`Lazy`/`Memo`), row, and placeholder capture metadata.
- `memo` thunk closures preserve memo identity placeholders for runtime memo cells.
- force sites preserve the checked latent-row obligations from the forced mode type at the lowering row level.

## Out Of Scope

## Runtime Notes

- Forcing `ForceThunk` at runtime is where lazy re-run and memo replay behavior are enforced.
- Lazy thunks re-evaluate body on each force and contribute the thunk latent row to the force-local row.
- Memo thunks run at most once unless cache replay and preserve terminal failure outcomes.
- Re-entrant memo forcing reports a trap to avoid inconsistent recursive memo-cell state.

## Out Of Scope

The following features are intentionally not implemented in this phase:

- surface-to-Core lowering is out of scope;
- typeclass solving is out of scope;
- user-defined algebraic effects are out of scope;
- Core Match is out of scope;
- full type checker is out of scope.

The Core AST and SPEC-099 may mention some of these shapes as target design hooks. This reference only claims the implemented subset proven by the Phase 161 tests.
