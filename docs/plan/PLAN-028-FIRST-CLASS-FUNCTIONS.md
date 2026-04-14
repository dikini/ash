# PLAN-028: First-Class Functions and Closure Values

**Status:** Planned
**Date:** 2026-04-14
**Spec:** SPEC-031-FIRST-CLASS-FUNCTIONS.md (v0.4, approved)
**Phase:** 80

## Summary

Implement first-class function values in Ash. Local function definitions become expressions that produce closure values. The interpreter (`ash-interp`) handles function definition and application natively, eliminating the `pure_runtime.rs` workaround (476 lines of duplicate interpreter).

## Goals

1. `fn(params) { body }` is an expression producing `Value::Closure`
2. Named local functions desugar to `let name = fn(params) { body }`
3. Closures capture their lexical environment via `Arc<EnvFrame>` (shared, O(1))
4. Recursion via `BindingSlot::Late` (mutable placeholder filled after construction)
5. Higher-order functions (passing/returning closures)
6. `Expr::FnApply` for user function calls, `Expr::Call` for built-ins only
7. Type system uses existing `Type::Fn` (pure) / `Type::Fun` (effectful)
8. Three-vertex boundary enforcement via type checker
9. Delete `pure_runtime.rs` and all inlining/dispatch hacks

## Task Breakdown

Tasks are organized by SPEC-031 migration phases. Dependencies are sequential within each phase; phases are sequential.

### Phase A: Core IR + Interpreter (TASK-551)

Foundation: new types, serialization, interpreter eval cases. Non-breaking -- new variants exist but nothing produces them yet.

### Phase B: Lowering + Type Checker (TASK-552, TASK-553, TASK-554)

Wire the new IR through lowering and type checking. Still non-breaking -- `pure_runtime` remains active.

### Phase C: Delete pure_runtime (TASK-555)

Remove the duplicate interpreter and all dispatch/inlining code. This is the breaking change.

### Phase D: Parser Expression Forms (TASK-556, TASK-557)

Parse `fn(params) { body }` and `|x| => body` as expressions. Post-parse validation.

### Phase E: Effect Typing (TASK-558)

Three-vertex enforcement: `Type::Fun` closures rejected in pure `fn` parameters.

### Validation (TASK-559)

End-to-end tests, CHANGELOG, task status updates.

## Dependency Graph

```
TASK-551 (core IR + interp eval)
    │
    ├──→ TASK-552 (lowering)
    │       │
    │       ├──→ TASK-553 (type checker)
    │       │
    │       └──→ TASK-554 (engine: inline into FnApply)
    │               │
    │               └──→ TASK-555 (delete pure_runtime)
    │                       │
    │                       ├──→ TASK-556 (parse fn expressions)
    │                       │       │
    │                       │       └──→ TASK-557 (closure syntax |x| => ...)
    │                       │
    │                       └──→ TASK-558 (effect typing / three-vertex)
    │                               │
    │                               └──→ TASK-559 (validation)
    └──────────────────────────────────┘
```

## Files Affected

| File | Change | Task |
|------|--------|------|
| `crates/ash-core/src/ast.rs` | Add `Expr::FnDef`, `Expr::FnApply` | TASK-551 |
| `crates/ash-core/src/value.rs` | Add `Value::Closure` (custom serde) | TASK-551 |
| `crates/ash-core/src/env_frame.rs` | NEW: `EnvFrame`, `BindingSlot` | TASK-551 |
| `crates/ash-core/src/lib.rs` | Export `env_frame` module | TASK-551 |
| `crates/ash-interp/src/context.rs` | Add `to_env_frame`, `from_env_frame` | TASK-551 |
| `crates/ash-interp/src/eval.rs` | Add `FnDef`/`FnApply` eval cases | TASK-551 |
| `crates/ash-interp/src/error.rs` | Add `NotCallable`, `BoundaryViolation` | TASK-551 |
| `crates/ash-parser/src/lower.rs` | Built-in registry, `lower_fn_def`, `FnApply` | TASK-552 |
| `crates/ash-typeck/src/check_expr.rs` | `FnDef`/`FnApply` type checking | TASK-553 |
| `crates/ash-engine/src/lib.rs` | Inline into `FnApply`, remove dispatch | TASK-554 |
| `crates/ash-engine/src/pure_runtime.rs` | DELETE | TASK-555 |
| `crates/ash-parser/src/parse_expr.rs` | `fn(params){}` expression, closure syntax | TASK-556, TASK-557 |
| `crates/ash-parser/src/parse_workflow.rs` | `fn name(...)` in workflow body | TASK-556 |

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| `Value` custom serde breaks existing serialization | Medium | Test all existing serde round-trips before/after |
| `Expr` size increase from `FnDef`/`FnApply` | Low | Both use `Box<Expr>` for body; measure enum size |
| Interpreter performance regression | Low | Benchmark before/after Phase C; closures only created when `FnDef` is evaluated |
| Incomplete built-in registry causes wrong `Call`/`FnApply` split | Medium | Extract complete list from `eval_function_call`; add coverage test |
| `BindingSlot::Late` Mutex contention in concurrent execution | Low | Mutex is per-binding, held only during closure construction and recursion |
