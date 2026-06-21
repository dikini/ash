# Core Ash Type Checking

This page documents the implemented Phase 162 Core Ash type-checking boundary in `ash-core::core_ash_typecheck`. It describes the Rust implementation, not the whole SPEC-100 target design space.

Core type checking consumes representation-validated Core from `ash-core::core_ash_validate`. Raw `.core` text follows the pipeline `parse -> validate -> type-check -> lower`; raw parsed Core must become `ValidCoreProgram` before it reaches the type checker.

## Implemented Boundary

The main entrypoint is `type_check_core_program(program, env) -> Result<TypedCoreProgram, CoreTypeCheckError>`.

The checked lowering entrypoint is `type_check_and_lower_core_program(program, env, context)`. It type-checks first, then lowers with checked row facts so CPS lowering does not recompute or contradict the Core checker's continuation and external function rows.

The primary public carriers are:

- `ValidCoreProgram`: the validated Core input boundary.
- `CoreTypeCheckEnv`: scoped names for types, values, continuations, row variables, operation signatures, and discharge metadata.
- `TypedCoreProgram`: the checked program result, including result type, local row, and facts.
- `CoreTypeCheckFacts`: metadata for later compiler stages, including jump continuation rows, refinement obligations, and discharge records.
- `CheckedLoweredCoreProgram`: the paired typed/lowered artifact returned by checked lowering.

## Algorithmic Profile

The Phase 162 checker is annotation-led. Core terms are expected to carry explicit types and rows at function, continuation, handler, and binding boundaries. The checker synthesizes local facts where the Core AST already gives enough information, but it is not full Hindley-Milner inference.

Rows are normalized before comparison. Exact duplicate row items normalize away for checking, row item namespaces are preserved, and open rows support conservative structural row solving through explicit row variables. This is structural row solving, not global row-polymorphic inference.

Function calls charge callee-local rows. `Jump` has Core local row `{}` while `CoreTypeCheckFacts` preserves the target continuation row for CPS `Jump.row`. `Raise` checks operation signatures and reports the operation-local row only. `Handle` checks affine resume types and preserves captured resume effects in the residual row.

Refinement checks record refinement obligations. Checking a plain base value as a refinement emits an obligation; using an existing refinement at its base type emits no new obligation. Predicate strings are scoped metadata in this phase.

Contract discharge checking validates discharge metadata shapes. Static/evidence discharges require proven evidence metadata, dynamic discharges require no proof evidence, and contract violations remain trap metadata rather than row items or raised operations.

## Mode-Specific Type Checking

Phase 163 adds mode constructors and lazy/memo thunk typing in the same checker architecture:

- `strict`, `lazy`, and `memo` are represented as `CoreType::Mode { mode, inner, latent_row }`.
- Mode types are invariant with respect to source and target positions.
- `strict` mode requires an omitted latent row (`None`), while `lazy`/`memo` require an explicit row.
- `CoreValue::Thunk` checks that:
  - its `result_ty` and row match the checked body type/row;
  - strict mode is rejected for thunk results;
  - its body has row `result` and the body result type is the strict inner type.
- `CoreExpr::LetMode` checks mode/type agreement at construction time and records mode metadata for checked-lowering:
  - `strict` binds the exact strict annotation and has local row `{}`;
  - `lazy` and `memo` also bind a strict inner value result type and keep the thunk latent row for force-time use.
- `CoreExpr::Force` is value-position only (`CoreAtom::Var`) and contributes the bound thunk latent row to force rows.
- Checked summaries preserve mode metadata and latent rows for exported variables and diagnostics.

## Deferred Features

The implemented boundary is intentionally smaller than the end-state type system:

- not full Hindley-Milner inference;
- not proof solving, SMT solving, QuickCheck discharge, or SmallCheck discharge;
- not typeclass solving or ad-hoc polymorphism;
- not MultiShotPure continuation semantics;
- not arbitrary user-defined algebraic effects;
- not surface-to-Core lowering;
- not session-type or MPST channel checking.

Upper layers may lower typeclasses, dictionary evidence, laws, properties, and richer surface effects into Core metadata or known operation kinds later. Phase 162 only checks the Core forms and metadata that already exist at this boundary.

## Fixture Coverage

The implemented behavior is covered by focused task tests in `crates/ash-core/tests/task_1640_*` through `task_1651_*`, plus phase-163 mode tests in `task_1665_*` through `task_1672_*` and the dedicated mode-reference consistency test in `task_1673_core_lazy_memo_docs_consistency.rs`.

The integration fixtures in `crates/ash-core/tests/task_1650_core_typecheck_integration.rs` prove valid `.core` fixtures type-check before lowering and that invalid fixtures fail at parse, validation, or type-checking before lowering. They also prove checked lowering preserves target continuation rows and external function rows from `CoreTypeCheckEnv`.
