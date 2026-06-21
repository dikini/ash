# Core Ash Lazy and Memo Modes

This page documents the SPEC-101 lazy/memo mode features that are implemented in `ash-core` as of this phase.

## Supported Syntax And AST Carriers

Core text-mode shape now includes:

- Mode types:
  - `(strict T)`
  - `(lazy T {row})`
  - `(memo T {row})`
- Thunk values:
  - `(thunk lazy T {row} Expr)`
  - `(thunk memo T {row} Expr)`
- Mode binding form:
  - `(let-mode Name mode : Type Expr Expr)`
- Force form:
  - `(force Name ThunkAtom Expr)`

These are parsed/serialized by `core_ash_text` and covered by dedicated parse/serialize fixtures and AST tests.

## Type-Checker Semantics

Mode types are implemented in `core_ash_typecheck` with these concrete checks:

- `CoreType::Mode` is treated as an invariant type wrapper and keeps the inner type and latent row.
- `strict` forms have `latent_row == None`.
- `lazy`/`memo` forms require `latent_row` presence and shape well-formedness.
- `CoreValue::Thunk` checks result/body against the thunk annotations and preserves the thunk latent row on the bound mode type.
- `CoreExpr::LetMode` checks mode/type agreement and local row accounting:
  - `strict` path binds with local row `{}`.
  - `lazy` and `memo` bind with local row `{}` but record latent rows in mode-binding metadata.
- `CoreExpr::Force` accepts a variable-only thunk atom and contributes the thunk latent row at the force site.

## Lowering Semantics

Lowering in `core_ash_lower` maps mode constructs to existing CPS infrastructure:

- `CoreValue::Thunk` lowers to `Value::ThunkClosure` with:
  - captured thunk mode (`Lazy`/`Memo`),
  - captured lexical environment,
  - captured handler/provider chain,
  - thunk latent row.
- `CoreExpr::LetMode`:
  - strict bindings follow direct strict-lowering paths,
  - lazy/memo bindings allocate thunk closures at binding time.
- `CoreExpr::Force` lowers to `PrimOp::ForceThunk` and expects a previously bound thunk variable.
- Memo closures can carry a runtime memo-cell placeholder carried by the CPS runtime.

## Runtime Behavior

The implemented behavior follows SPEC-101 runtime semantics:

- **lazy**: every force re-runs the thunk body.
- **memo**: first force evaluates the body and records terminal outcome; later forces replay cached terminal outcome.
- **memo cached failure/trap**: terminal successes and cacheable failures/traps are replayed.
- **memo re-entrancy**: recursive forcing while evaluating the same memo thunk reports structured trap failure rather than deadlocking.
- **captured authority**: forcing executes in the chain captured at thunk construction; force-time creation chain is explicitly not the authority model.

## Trace Events

Phase 163 runtime tests assert these trace families for thunk/memo behavior:

- `ThunkConstructed`
- `ThunkForceStarted`
- `ThunkBodyEvaluationStarted`
- `ThunkBodyEvaluationCompleted`
- `ThunkForceCompleted`
- `MemoCacheFilled`
- `MemoCacheHit`
- `MemoReplayFailure`
- `MemoReentrantRejected`

## Non-Goals for This Phase

- surface-to-Core lowering for lazy/memo sugar;
- implicit mode subtyping/coercion;
- lazy/memo optimization rewrites (only explicit forms are implemented);
- cross-run persistent memo caches.

This phase intentionally keeps mode behavior explicit in Core so it is fully auditable by parser, type-checker, and lowering.
