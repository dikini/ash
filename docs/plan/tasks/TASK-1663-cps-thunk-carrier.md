# TASK-1663: Add CPS thunk carrier

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Add the value-level CPS/runtime carrier needed for SPEC-101 thunk execution without adding CPS tail-term variants.

## Specification Reference

- [SPEC-101 §11](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#11-core-to-cps-lowering)

## Dependencies

- [TASK-1660](TASK-1660-core-mode-ast-carriers.md)

## Requirements

1. Add `ThunkMode::{Lazy, Memo}` in `crates/ash-core/src/cps.rs`.
2. Add `Value::ThunkClosure { mode, body, captured_env, captured_chain, row, memo_cell }`.
3. `body` must be a zero-argument `Value::Lam`; CPS validation rejects any other thunk body.
4. Add an opaque `MemoCellId` carrier in `ash-core` with a private inner field,
   `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`, and exact API
   `pub fn new(raw: u64) -> Self` and `pub fn raw(self) -> u64`; do not put interpreter
   `CpsError` or cached runtime outcomes in `ash-core`.
5. Add interpreter-owned `CpsRuntime { next_memo_cell, memo_cells, trace: Vec<TraceEvent> }`,
   `MemoCellState::{Empty, Evaluating, Filled}`, and `CachedThunkOutcome` scaffolding in
   `crates/ash-interp/src/cps/`. `MemoCellState` and `CachedThunkOutcome` derive
   `Debug, Clone, PartialEq`.
6. `CpsRuntime::allocate_memo_cell()` returns a fresh `MemoCellId` and inserts `Empty`; runtime
   `eval_value` must call it while constructing a memo `ThunkClosure` and return the closure with
   `memo_cell: Some(id)` before binding it.
7. `memo_cell` is process-local runtime state and must use exactly
   `#[serde(skip, default)]` on `Value::ThunkClosure.memo_cell`.
8. Preserve serde round-trip behavior for serializable CPS values.
9. Cloned `ThunkClosure` values with the same `MemoCellId` share one memo cell within the same
   `CpsRuntime`; separate `eval_checked`/`eval_unchecked` top-level calls use fresh runtimes.
10. `.cps` fixture/debug text serializers omit `memo_cell`; human diagnostics that must mention
    memo identity render only `<memo-cell>`.
11. Add only the `trace: Vec<TraceEvent>` sink in this task; do not add thunk-specific trace
    variants or emissions until TASK-1672.

## Existing Code Touchpoints

- `crates/ash-core/src/cps.rs`: add `ThunkMode`, `MemoCellId`, `Value::ThunkClosure`, and
  `PrimOp::ForceThunk`.
- `crates/ash-interp/src/cps/mod.rs`: add `CpsRuntime`, memo-cell state, and runtime-aware
  evaluation scaffolding.
- `crates/ash-interp/src/cps/validate.rs`: reject malformed `ThunkClosure` bodies that are not
  zero-argument lambdas.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1663_cps_thunk_carrier.rs`.
2. Add failing tests in `crates/ash-interp/tests/task_1663_cps_runtime_scaffold.rs`.
3. Run `cargo test -p ash-core --test task_1663_cps_thunk_carrier`; expect missing carrier failures.
4. Run `cargo test -p ash-interp --test task_1663_cps_runtime_scaffold`; expect missing runtime scaffold failures.
5. Add the CPS value and memo-state data structures.
6. Re-run `task_1663`, `task_1590`-style CPS data tests, serde tests, and the `ash-interp`
   runtime scaffold test.

## Completion Checklist

- [x] CPS values can represent lazy and memo thunk closures.
- [x] `MemoCellId` has a private field plus `new(raw)` and `raw()` methods; no public tuple field.
- [x] Memo cell state is process-local runtime state.
- [x] `CpsRuntime.trace` is `Vec<TraceEvent>`, not a separate internal event type.
- [x] TASK-1663 adds no thunk-specific trace variants or emissions.
- [x] Serialization does not leak runtime memo storage internals.
- [x] Serde uses `#[serde(skip, default)]` for `memo_cell`, and fixture text omits it.
- [x] CPS validation rejects malformed thunk bodies that are not zero-argument lambdas.
- [x] `ash-core` does not depend on `ash-interp` memo outcome or error types.
- [x] Runtime tests can create an explicit `CpsRuntime` and observe shared memo cells.
- [x] Memo cell allocation happens at thunk construction, not at force time.
