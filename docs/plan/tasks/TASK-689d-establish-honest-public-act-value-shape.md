# TASK-689D: Establish honest opaque `Act` library boundary for ordinary `std::act` helpers

> **TASK-2041 status:** This completed task's older runtime references do not authorize a direct
> evaluator, non-Engine CPS executor, differential route, or client fallback.

## Status: ✅ Complete

## Description

After TASK-689C, `std::act::guard` can be implemented honestly through the narrow `act::policy_check` bridge without exposing user-destructurable runtime state. TASK-689D now focuses on replacing the remaining public-surrogate framing with the agreed runtime-managed state-monad substrate for `Act`.

The preferred direction is Option D:
- keep `Act` representationally opaque at the public/library level
- export smart constructors and algebraic combinators from `act::...`
- avoid overloading `Result` or any other public ADT as the semantic identity of `Act`
- treat domain-level success/failure as user-chosen in `A` (for example `Act<Result<A, E>>`), not as part of the `Act` substrate itself
- review and revise Phase-97 specs if any frozen wording prevents that straightforward opaque implementation path

## Specification Reference

- SPEC-047 §2.5
- SPEC-047 §7
- SPEC-047 §8

## Dependencies

- 📝 TASK-689C: prerequisite task
- 📝 TASK-689E: library type-export semantics prerequisite

## Requirements

### Functional Requirements

1. Review the frozen specs/plans for any wording that prevents an opaque public `Act` implementation path.
2. Treat `Act` as representationally opaque in ordinary library code; prefer exported smart constructors/combinators over exposing a surrogate public data representation.
3. Use the runtime-managed state substrate as the semantic target: preferred reading `builtin type ActEnv`; `type Act<A> = ActEnv -> (ActEnv, A)`, with downgrade to checked correspondence only if real engine/runtime pressure makes literal definitional equality too costly.
4. Apply the agreed builtin-boundary fallback ladder during implementation:
   - A: builtin `ActEnv`, ordinary alias `Act`
   - B: builtin `ActEnv`, builtin `Act` with explicit equation
   - C: builtin `ActEnv`, fully opaque builtin `Act`
5. Add focused evidence/tests showing where implementation pressure forces escalation away from A/definitional equality, if it does.
6. Update TASK-689 and plan surfaces honestly based on the actually landed boundary.

### Property Requirements (proptest)

```rust
// Prefer focused regression tests unless this task introduces a broader new
// tuple/public-value invariant worth property coverage.
```

## TDD Steps

### Step 1: Write Tests (Red)

Add failing tests that prove the remaining ordinary-library helper blocker is now the `Act` substrate/builtin-boundary implementation path, not policy/member access.

### Step 2: Implement (Green)

Land the smallest honest substrate needed for an opaque `Act` library boundary. Start from A plus definitional equality and escalate only if real implementation pressure justifies it under the agreed rules.

### Step 3: Integration (Green)

Verify the real `std::act` import/type/execute path for ordinary helpers.

### Step 4: Verification

Re-run focused checks and update task/plan surfaces to match reality.

## Verification Steps

- [x] Focused tests capture the remaining `Act` substrate/builtin-boundary blocker.
- [x] Ordinary-library `unit`, `bind`, and `then` are implemented honestly over the runtime-managed substrate through the A-path public boundary.
- [x] TASK-689 status is updated honestly.
- [x] `cargo fmt --check` passes.

## Dependencies for Next Task

This task determines whether TASK-689 can finally replace all remaining placeholder `std::act` helper declarations with ordinary library implementations.

## Notes

- Phase 97 is additive.
- TASK-689C resolved the policy/environment blocker for `guard`.
- Preferred design direction: Option D — keep `Act` opaque and export its algebra through `act::...` rather than blessing a surrogate public data representation.
- `Act<Result<A, E>>` remains a conventional shape for effectful computations that also return domain-level success/failure, but `Result` is not part of the `Act` substrate and users remain free to choose other domain result/value types.
- Agreed implementation decision rules:
  - prefer definitional equality for the builtin RHS over checked correspondence;
  - downgrade to checked correspondence only if real engine/runtime complexity makes definitional equality materially riskier or more complex;
  - prefer A over B unless A creates real implementation pressure; prefer B over C unless B creates real implementation pressure.
- Builtin artifact identity should be internal/flat rather than path-based; source spellings like `act::ActEnv` are bindings onto that builtin identity and may be reexported without changing meaning.
- First concrete implementation pressure discovered and addressed:
  - parser/type parsing did not accept `builtin type ...` syntax
  - parser/type parsing did not accept `Fn(...) -> ...` inside `type` aliases
  - `ash-typeck` did not yet lower `TypeExpr::Constructor { name: "Fn", ... }` into `Type::Fn(...)`
  - `ash-typeck` builtin types did not yet include `ActEnv`
- Current status after the first A-path slice:
  - parser now accepts `builtin type ActEnv;`
  - parser now accepts arrow-style function types in the relevant type surfaces
  - `ash-typeck` now recognizes `ActEnv` and lowers function-typed `Act` aliases through the import/type boundary
  - focused parser/type/engine checks are green for the preferred A-path substrate probe
  - no escalation to B or C has been justified yet
- Completion status:
  - ordinary `std::act` helpers import/typecheck and return opaque `Act` values through the public boundary
  - the runtime rejects arbitrary visible dummy arguments as fake `ActEnv` carriers for `unit`- and `then`-produced `Act` values, establishing a protected hidden-carrier boundary without escalating away from A
  - the A-path preserves closure-backed state-thread shape under internal forcing: focused `ash-interp` tests show lowered `act { ret ... }`, nested `bind`, and `invoke(...)` all require the hidden carrier and return pair-shaped internal results using the protected runtime token
  - forcing an `Act` closure no longer succeeds on a visible `ActEnvToken` alone; `Context` must carry a hidden runtime `ActEnv`, and the workflow bridge constructs and attaches that hidden object
  - `invoke(...)` dispatches through the hidden capability context when a hidden runtime `ActEnv` is present, including workflow-bridge execution with a real registered provider
  - the helper-thread compatibility path builds a tiny Tokio runtime before forcing provider futures, so Tokio-runtime-dependent providers execute through the synchronous bridge while wider async integration continues
  - `ash-interp::eval_expr_async(...)` provides an async Act-force path that drives `invoke(...)` through the hidden runtime `ActEnv` without going through the helper-runtime bridge, and focused tests cover Tokio-dependent provider success on that path
  - `Context` and `RoleContext` are `Send`/`Sync`-safe at the storage layer: their `RefCell<HashSet<_>>` obligation stores have been replaced with mutex-backed storage, and focused tests pin both Send/Sync readiness and preserved by-value clone semantics
  - the Send/Sync refactor intentionally preserves today's clone semantics by deep-copying obligation/discharge sets; future optimisation opportunities (copy-on-write, split context state, small-set storage, clone/fork API separation) are recorded in `docs/notes/NOTE-007-context-send-sync-clone-cost-and-optimization-opportunities.md`
  - workflow execution consumes the async path in the relevant expression-evaluation surfaces (`Workflow::Ret`, `Workflow::Let`, conditional conditions, `Orient`, `Decide`, `ForEach`, `Check`, `Set`, `Send`, `Spawn.init`, `Split`, `Yield` request evaluation, `ProxyResume` response evaluation, action-argument evaluation, and runtime callable argument binding), and focused task-local provider tests confirm these migrated surfaces use the async force path instead of the helper-runtime bridge
  - `ash-interp::eval_expr_async(...)` covers the full current `Expr` surface rather than only the original narrow core: recursive async evaluation exists for `Expr::FieldAccess`, `Expr::IndexAccess`, `Expr::Unary`, `Expr::Binary`, `Expr::Call`, `Expr::Constructor`, `Expr::Match`, `Expr::IfLet`, `Expr::Let`, `Expr::FnApply`, and `Expr::Split`, while `Expr::Spawn`, `Expr::CheckObligation`, and `Expr::FnDef` have explicit async-path handling so workflow execution no longer depends on a default sync-expression fallback inside the async evaluator
  - the stream-backed workflow entry path also attaches the hidden runtime `ActEnv`, so async `Send`-surface evaluation has the same hidden carrier discipline as the ordinary workflow path
  - `guard` is no longer the blocker for this task; it fails closed without policy context through the narrow `policy_check` bridge
  - no real implementation pressure justified escalation from A to B or C
  - the remaining token/list force result shape is an internal runtime compatibility detail for the current closure-backed implementation, not a public `std::act` representation; replacing it with a fully native effect-runtime carrier is broader follow-on work and should not be silently folded into TASK-689D
