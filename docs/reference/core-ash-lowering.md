# Core Ash Lowering

This page documents the implemented Phase 161 Core-to-CPS lowering boundary. It describes the Rust implementation in `ash-core::core_ash_lower`, not the whole SPEC-099 design space.

Core-to-CPS lowering consumes validated Core from `ash-core::core_ash_validate` and produces the existing `ash-core::cps::Term` tree. Raw parsed `.core` files must pass validation first; the lowering API is intentionally shaped around validated Core.

## Implemented Input Boundary

The lowering pass handles the Phase 161 fixture subset:

- atoms and literal values;
- lambda, record, tuple, and discharge-marker values where the current CPS carrier can represent them;
- `let-val`, `let-rec`, `let-prim`, `let-call`, `if`, tail `call`, and `jump`;
- capability, channel, process, and failure `raise` operations;
- single-clause `handle` with affine resume metadata;
- `record-discharge`;
- `trap`.

The fixture examples live under `crates/ash-core/tests/fixtures/core/`:

- `call_non_tail.core` shows a direct-style tail call.
- `let_call.core` shows a non-tail direct-style call.
- `raise_handle.core` shows operation raising and handling.
- `contract_trap.core` shows contract discharge plus trap lowering.

Each valid fixture has a matching `.cps.golden` file checked by `task_1629_core_end_to_end.rs`.

## Lowering Rules

Pure bindings lower structurally:

- Core `LetVal` lowers to CPS `LetVal`.
- Core `LetRec` lowers to CPS `LetRec`.
- Core `LetPrim` lowers to CPS `LetPrim`.
- Core `If` lowers to CPS `If`; the CPS `If.row` is the union of local branch rows, not the current continuation row.
- Core `Jump` lowers to CPS `Jump`; its row is the target continuation row.

Tail calls lower to CPS `Call` with the current continuation. The call row is the union of the callee body row and the current continuation row, matching SPEC-098b's call-row split.

Non-tail direct-style calls lower through CPS `LetCont`. Core `LetCall` creates a fresh continuation label for the rest of the Core expression, then emits a CPS `Call` that resumes at that label. `let_call.core` is the smallest committed fixture for this path.

Core lambda values lower to CPS lambdas by adding a fresh continuation parameter. A Core lambda body that is an atom becomes a CPS jump to that continuation parameter.

## Effects, Handlers, And Contracts

Core `Raise` lowers to CPS `Raise` with the current continuation as `resume`. The CPS `Raise.row` contains the raised operation's local operation row only.

Core `Handle` lowers to CPS `Handle` with the current continuation in `Handle.cont`. `Handle.row is local residual row`: it comes from the handler clause row and excludes the outer continuation row. The total behavior is accounted for by combining this local residual row with the continuation row at the surrounding call/continuation boundary.

Handler resume parameters lower as affine continuation variables. Phase 161 validation rejects duplicate direct resume jumps, ordinary argument passing, lambda capture, and record/tuple storage of the resume variable. `MultiShotPure is out of scope` for this implementation slice.

Contract discharge lowers to CPS `RecordDischarge`. `ContractViolation is trap metadata`: dynamic contract failure lowers to `Trap { reason: ContractViolation }` under the discharge record, not to a contract row item and not to a raised operation. Recoverable behavior must use an explicit failure operation upstream.

## Out Of Scope

The following features are intentionally not implemented in Phase 161:

- surface-to-Core lowering is out of scope;
- typeclass solving is out of scope;
- user-defined algebraic effects are out of scope;
- MultiShotPure is out of scope;
- Core Match is out of scope;
- full type checker is out of scope.

The Core AST and SPEC-099 may mention some of these shapes as target design hooks. This reference only claims the implemented subset proven by the Phase 161 tests.
