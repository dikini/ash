# TASK-1664: Implement CPS force runtime behavior

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Implement lazy and memo thunk forcing in the CPS interpreter/runtime.

## Specification Reference

- [SPEC-101 §6](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#6-operational-semantics)
- [SPEC-101 §11](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#11-core-to-cps-lowering)

## Dependencies

- [TASK-1663](TASK-1663-cps-thunk-carrier.md)

## Existing Code Touchpoints

- `crates/ash-interp/src/cps/mod.rs`: update `eval_unchecked`, `eval_checked`, `eval_term`,
  `eval_value`, `eval_letprim`, `eval_prim`, `eval_atom_to_value`, and thunk-force helper code.
- `crates/ash-core/src/cps.rs`: use `Value::ThunkClosure`, `PrimOp::ForceThunk`, `MemoCellId`,
  `ConsumedFlag`, `Env`, `HandlerChain`, and `EffectRow`.
- `crates/ash-interp/src/cps/validate.rs`: validate `PrimOp::ForceThunk` arity and
  `ThunkClosure` body shape.

## Requirements

1. Lazy force evaluates the thunk body every time.
2. Memo force evaluates once, then replays cached success, recoverable failure, or trap.
3. Divergence never fills a memo cell.
4. Re-entrant memo force deterministically traps with a structured runtime diagnostic.
5. Force evaluates the body under the thunk's captured env and captured handler/provider chain.
6. Add `PrimOp::ForceThunk` and special-case it in `eval_letprim`; do not route force through
   ordinary pure `eval_prim`.
7. Cache current CPS terminal outcomes in interpreter-owned state as
   `CachedThunkOutcome::Success(Atom)` or `CachedThunkOutcome::Failure(CpsError)`.
8. Re-entrant force must return
   `CpsError::Trap(TrapReason::Custom("re-entrant memo force".to_string()))`.
9. Add runtime-aware internals, including `eval_unchecked_with_runtime(term, env, chain, runtime)`;
   keep existing public entrypoints by making them create a fresh `CpsRuntime` per top-level call.
10. Change value construction evaluation so `ThunkClosure` captures the current `Env` and
    `HandlerChain` at runtime when its capture fields are empty/default placeholders.
11. `eval_value_with_runtime` overwrites empty/default thunk capture placeholders exactly once at
    construction time. If a programmatic test passes a `ThunkClosure` with non-empty capture fields,
    treat it as already runtime-constructed and preserve those fields. `.core` and `.cps` fixture
    text cannot provide live runtime captures.
12. Force a thunk body by installing a synthetic continuation named by the zero-argument lambda's
    `cont` field. The continuation must bind `__force_result` and return it.
13. `ForceThunk` must require memo thunks to already carry `memo_cell: Some(id)`; allocation is a
    construction-time responsibility from TASK-1663.
14. Add `CpsError::ExpectedThunk(Value)` and use it when `ForceThunk` receives a non-thunk value.
15. Do not reuse `InvalidPrimArgs` for a non-thunk force argument.

## Terminal Outcome Model

For Phase 163, memo cells cache `CpsResult<Atom>` outcomes from well-formed thunk bodies:

- cache `Ok(atom)`;
- cache `Err(CpsError::Trap(reason))`;
- cache `Err(CpsError::UnhandledEffect(op))` as the lowered recoverable failure/unhandled-effect
  representation until a later phase gives failure a narrower runtime carrier;
- cache another deterministic `CpsError` only if the test names and justifies that case;
- never fill a memo cell for divergence, panic, process termination, or non-returning execution.

## Runtime API

Implement these exact runtime-aware helper signatures:

```rust
pub fn eval_unchecked_with_runtime(
    term: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom>;

fn eval_value_with_runtime(
    value: &Value,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value>;

fn eval_letprim_with_runtime(
    name: &Name,
    op: &PrimOp,
    args: &[Atom],
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom>;

fn eval_force_thunk_binding(
    name: &Name,
    args: &[Atom],
    continuation_body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom>;

fn run_thunk_body_with_runtime(
    thunk: &Value,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom>;
```

Keep the existing public `eval_unchecked`, `eval_checked`, and `eval_term` entrypoints as wrappers
that allocate a fresh `CpsRuntime`. All recursive evaluator calls must use the runtime-aware
helpers. Do not use thread-local or process-global memo stores.

## ForceThunk Algorithm

`PrimOp::ForceThunk` takes exactly one argument. The resolved argument must be
`Value::ThunkClosure`; otherwise return `CpsError::ExpectedThunk(value)`. Lazy thunks always
evaluate the body. Memo thunks require an existing `MemoCellId`, transition
`Empty -> Evaluating -> Filled(outcome)`, replay cached `Filled` outcomes, and reject
`Evaluating` with the exact re-entrant trap above. On cacheable `Err(err)`, store
`Filled(Failure(err.clone()))` before returning `Err(err)`. On non-cacheable non-returning
outcomes, restore `Empty`.

Do not keep a mutable borrow of `runtime.memo_cells` while evaluating a thunk body. Read or clone
the current cell state, release the map borrow, set the next state, then call
`run_thunk_body_with_runtime`.

The zero-argument body lambda is invoked under `thunk.captured_env` and `thunk.captured_chain` by
binding its `cont` name to:

```text
Value::Cont {
  param: "__force_result",
  body: Return { value: Var("__force_result") },
  captured_env: Env::new(),
  captured_chain: thunk.captured_chain.clone(),
  consumed: fresh ConsumedFlag,
  row: EffectRow::default(),
}
```

The fixed force control flow is:

1. Resolve the force argument to a `Value::ThunkClosure`.
2. Evaluate the thunk body under `thunk.captured_env` and `thunk.captured_chain`.
3. Return an `Atom` through the synthetic continuation.
4. Bind that atom as `Value::Atom(atom)` to the `LetPrim.name`.
5. Resume the original force-site body under the original force-site `env` and `chain`.

## Re-Entrant Test Shape

Create a direct CPS runtime test that constructs a memo `ThunkClosure`, inserts it into its
captured environment under its own name, and gives it a zero-argument body that forces that same
name via `PrimOp::ForceThunk`. The outer force sets the memo cell to `Evaluating`; the nested force
observes `Evaluating` and traps.

## TDD Steps

Implement TASK-1664 as four mandatory sub-assignments. Each sub-assignment must add failing tests,
make only the minimum implementation needed for that slice, run the focused test, and record its
evidence before moving on.

1. Runtime-aware evaluator plumbing:
   - Add tests proving `eval_unchecked`, `eval_checked`, and `eval_term` still work through a fresh
     `CpsRuntime`.
   - Add tests proving `eval_unchecked_with_runtime` preserves state across two calls using the
     same runtime.
   - Implement only the runtime-aware helper threading.
2. Lazy force:
   - Add tests for `ExpectedThunk`, `ForceThunk` arity validation, synthetic continuation return,
     and lazy re-run.
   - Implement only lazy force behavior.
3. Memo success cache:
   - Add tests for construction-time memo-cell allocation and successful cache hit.
   - Implement only `Empty -> Evaluating -> Filled(Success(atom))` and success replay.
4. Cached failure/trap and re-entrant rejection:
   - Add tests for cached trap replay, cached unhandled-effect replay, non-cacheable reset to
     `Empty`, and re-entrant memo rejection.
   - Implement failure caching and re-entrant handling.

Run after every sub-assignment:

```bash
cargo test -p ash-interp --test task_1664_cps_force_runtime
```

Run after all four sub-assignments:

```bash
cargo test -p ash-interp
```

## Completion Checklist

- [x] Lazy force re-runs.
- [x] Memo force caches terminal outcomes.
- [x] Re-entrant memo force traps.
- [x] Non-thunk force returns `CpsError::ExpectedThunk(Value)`.
- [x] Captured chain is restored for thunk body execution.
- [x] Existing public eval entrypoints are compatibility wrappers over `CpsRuntime`.
- [x] Force resumes the force-site body under the original force-site env/chain.
- [x] Empty/default capture placeholders are overwritten exactly once at runtime construction.
- [x] Programmatic non-empty capture fields are preserved as already constructed runtime values.
- [x] Fixture text cannot encode live captured env/chain fields.
- [x] TASK-1664 was executed in the four mandatory sub-assignments above.
