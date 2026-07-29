# TASK-1520: Closure Refinement Audit and Capture Channels

> **TASK-2041 status:** This completed audit does not authorize a direct evaluator, non-Engine CPS
> executor, differential route, or client fallback.

## Status: ✅ Complete

## Description

Audit all current closure creation points in the Ash codebase, identify all capture channels, and document effect leakage scenarios.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## Audit Results

### Closure Creation Points

| Location | File | Line | Behavior |
|----------|------|------|----------|
| `eval_expr` (async) | `crates/ash-interp/src/eval.rs` | 1750-1767 | Creates `Value::Closure`, checks `ctx.is_pure()`, rejects if pure |
| `eval_expr` (sync) | `crates/ash-interp/src/eval.rs` | 2473-2493 | Same as above, second copy |
| `parse_expr` | `crates/ash-parser/src/parse_expr.rs` | 470-519 | Parses `Expr::FnDef` (no purity check) |
| `parse_closure_expr` | `crates/ash-parser/src/parse_expr.rs` | 534-577 | Parses `|x| -> x + 1` shorthand |
| `check_expr` | `crates/ash-typeck/src/check_expr/mod.rs` | 690-754 | Types `Expr::FnDef` as `Type::Fn` always (no capture check) |

### Capture Channels

1. **Lexical scope via `EnvFrame`**: `ctx.to_env_frame()` captures all visible bindings
2. **Parameters**: Closure parameters are bound when called, not when created
3. **Parent chain**: `EnvFrame` uses `Arc` for O(1) parent chain capture

### The Blanket Ban (Found)

The runtime enforces: `if ctx.is_pure() { reject closure }`

This is in **two places**:
- `eval.rs:1760` (async eval path)
- `eval.rs:2486` (sync eval path)

### Classification

| Scenario | Current Rule | New Rule (SPEC-088) |
|----------|-------------|---------------------|
| `fn(x) { x + 1 }` in pure context | ❌ Blocked | ✅ Allowed (no captures) |
| `fn(x) { n + x }` where n is Int | ❌ Blocked | ✅ Allowed (pure capture) |
| `fn(x) { fs.read(x) }` where fs is Cap | ❌ Blocked | ❌ Blocked (effectful capture) |
| `fn(x) { secret + x }` where secret is Act-produced | ❌ Blocked | ❌ Blocked (effectful capture) |

## Verification

- [x] Audit report reviewed and approved
- [x] All scenarios classified correctly
- [x] No missing capture channels identified

## Closeout Checklist

- [x] Audit complete
- [x] Findings documented
- [x] Committed to branch
