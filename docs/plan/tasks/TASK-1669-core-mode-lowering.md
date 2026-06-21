# TASK-1669: Lower Core mode forms to CPS thunk runtime

**Status:** Planned
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Lower `Thunk`, `LetMode`, and `Force` into CPS values and existing CPS tail terms plus runtime force behavior.

## Specification Reference

- [SPEC-101 §11](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#11-core-to-cps-lowering)

## Dependencies

- [TASK-1664](TASK-1664-cps-force-runtime.md)
- [TASK-1667](TASK-1667-core-letmode-force-typechecking.md)

## Existing Code Touchpoints

- `crates/ash-core/src/core_ash_lower.rs`: lower `CoreValue::Thunk`, `CoreExpr::LetMode`, and
  `CoreExpr::Force`; preserve checked row facts.
- `crates/ash-core/src/core_ash.rs`: consume the exact Core mode AST variants from TASK-1660.
- `crates/ash-core/src/cps.rs`: emit `Value::ThunkClosure` and `PrimOp::ForceThunk` with empty
  `captured_env`/`captured_chain` placeholders.
- `crates/ash-core/tests/task_1627*`, `task_1628*`, and `task_1650*`: keep existing lowering and
  checked-lowering expectations green.

## Requirements

1. Lower lazy/memo thunk construction to `ThunkClosure`, not a plain lambda.
2. Lower `Force` to `Term::LetPrim { op: PrimOp::ForceThunk, args: [thunk], body }` using
   the current continuation body; do not add a new CPS tail-term variant.
3. Emit `ThunkClosure` values with empty/default `captured_env` and `captured_chain` placeholders;
   runtime `eval_value` fills creation-time capture.
4. Lower `LetMode Strict` by evaluating the initializer immediately through the existing strict
   expression lowering path, then binding the strict value before lowering the body.
5. Lower `LetMode Lazy`/`LetMode Memo` by wrapping the initializer expression as a zero-argument
   thunk body without evaluating it at binding time.
6. Ensure `Force` unwraps `Lazy A` or `Memo A` to strict inner type `A` and binds that value to
   `Force.name`.
7. Do not add named Core builtins for `delay`, `delay_memo`, `force_unsafe`,
   `memoize_unsafe`, or `strip_cache_unsafe` in this phase.
8. Document and test their Core translations instead:
   - `delay(v)` -> lazy thunk whose body returns `v`;
   - `delay_memo(v)` -> memo thunk whose body returns `v`;
   - `force_unsafe(t)` -> `CoreExpr::Force`;
   - `memoize_unsafe(lazy_t)` -> memo thunk whose body forces `lazy_t`;
   - `strip_cache_unsafe(memo_t)` -> lazy thunk whose body forces `memo_t`.
9. Preserve checked row facts for force sites.
10. Keep existing lowering tests green.
11. Lazy/memo latent rows must come from checked-lowering metadata or explicit
    `CoreValue::Thunk.row` / `CoreType::Mode.latent_row`.
12. Lowering must not infer latent rows from generated CPS terms.
13. Lowering must not require the lazy/memo initializer expression to already be a
    `CoreValue::Thunk`.
14. `CoreLoweringContext` gets a dedicated `mode_binding_latent_rows` table copied from
    `CoreTypeCheckFacts::mode_binding_latent_rows()`.
15. `CoreExpr::Force` lowering reads `mode_binding_latent_rows[name]` when the thunk atom is
    `CoreAtom::Var(name)`.
16. For Phase 163, `CoreExpr::Force` lowering supports only `CoreAtom::Var(name)`. If a
    non-variable force atom reaches lowering, report an internal checked-lowering invariant
    violation instead of inventing a latent-row source.
17. Add exact lowering-context helpers
    `with_mode_binding_latent_row(self, name: CoreName, row: CoreRow) -> Self` and
    `mode_binding_latent_row(&self, name: &str) -> Option<&CoreRow>`.
18. Lowering must not mutate source Core types or recompute lazy/memo initializer rows; it consumes
    the checked facts recorded by TASK-1667.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1669_core_mode_lowering.rs`.
2. Include lowering assertions for thunk carrier shape and force-site rows.
3. Run the focused test and confirm missing lowering support.
4. Implement lowering in `core_ash_lower.rs`.
5. Re-run `task_1669`, `task_1627`, `task_1628`, and `task_1650`.

## Completion Checklist

- [ ] Effectful thunks do not lower to plain calls.
- [ ] Force uses runtime force semantics.
- [ ] Lowered rows match checked rows.
- [ ] Conversion operations are documented as Core translations, not new Phase 163 builtins.
- [ ] Lowering does not pretend to know runtime handler/provider chain state.
- [ ] Lowering consumes checker-provided latent-row facts rather than recomputing them.
- [ ] `CoreLoweringContext` carries `mode_binding_latent_rows`.
- [ ] `CoreLoweringContext` exposes the exact latent-row helper methods named above.
- [ ] Non-variable force atoms are treated as impossible after checked validation/type checking.
