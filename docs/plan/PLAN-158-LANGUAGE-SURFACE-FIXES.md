# PLAN-158: Language Surface Fixes

**Status:** ✅ Complete; Language surface fixes implemented
**Spec:** [SPEC-094: Language Surface Fix Specification](../spec/SPEC-094-LANGUAGE-SURFACE-FIX.md)
**Builds on:** [PLAN-157](PLAN-157-LIST-MIGRATION-HARDENING.md)
**Task range:** TASK-1580 through TASK-1584
**Completion Date:** 2026-06-17

## Goal

Fix three language surface issues that prevent idiomatic usage of pure algebraic data types and higher-order functions in Ash.

## Background

During Phase 157 (List Migration Hardening), three language limitations were identified:

1. **Module-level functions not visible in closures**: When a closure defined in a workflow calls a module-level function, the interpreter fails with `UndefinedVariable`. **Status: Deferred** - requires power tower lifting in parser.
2. **Function vs capability name collision**: The lowerer treats function calls (like `reverse(list)`) as capability lookups, causing "unresolved symbolic capability" errors. **Status: Fixed**.
3. **Closure expression parsing limitations**: `fn(x) { x }` cannot be parsed in general expression positions like function arguments. **Status: Fixed**.

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1580](tasks/TASK-1580-closure-module-function-visibility.md) | Fix module-level function visibility inside closures | 📝 Deferred; Requires power tower lifting in parser (distinguish pure fn calls from Act) |
| [TASK-1581](tasks/TASK-1581-function-vs-capability-resolution.md) | Distinguish function calls from capability calls in lowerer | ✅ Complete |
| [TASK-1582](tasks/TASK-1582-closure-expression-parsing.md) | Enable `fn` expression parsing in all expression contexts | ✅ Complete |
| [TASK-1583](tasks/TASK-1583-verification-and-regression-tests.md) | Add verification tests and ensure no regressions | ✅ Complete |
| [TASK-1584](tasks/TASK-1584-phase-158-closeout.md) | Close out Phase 158 with documentation and changelog | ✅ Complete |

## Implementation Summary

### TASK-1581: Function vs Capability Resolution

**Problem:** The lowerer conflated function calls with capability calls. Imported functions like `reverse` were treated as symbolic capabilities, causing "unresolved symbolic capability" errors.

**Fix:** In `crates/ash-parser/src/lower.rs`, the `SurfaceWorkflow::Act` handling now checks if a symbolic name is a function (not in `BUILTIN_FUNCTIONS` and not an effectful name) before treating it as a capability. If it's a function, it's lowered as `Workflow::Orient` wrapping a `FnApply` expression.

**Files modified:**
- `crates/ash-parser/src/lower.rs` - Added function vs capability check in Act lowering

### TASK-1582: Closure Expression Parsing

**Problem:** `fn` literals could not be parsed in general expression positions like function arguments.

**Fix:** Added `parse_fn_expr` to `primary_expr()` in `crates/ash-parser/src/parse_expr.rs`, enabling `fn` expressions anywhere a primary expression is expected.

**Files modified:**
- `crates/ash-parser/src/parse_expr.rs` - Added `parse_fn_expr` to `primary_expr()`

**Test added:**
- `crates/ash-engine/tests/fn_expr_parsing.rs` - Verifies `fn(x) { x + 1 }` works as `map()` argument

### TASK-1580: Module-Level Function Visibility (Deferred)

**Problem:** Module-level functions are not accessible from within closures defined in workflows.

**Root Cause:** The parser treats `f(5)` where `f` is a local variable as a workflow action (Act), not a function call. This is because the parser doesn't track local variables and can't distinguish between function calls and capability calls.

**Why Deferred:** Fixing this requires power tower lifting in the parser - the parser needs to understand the distinction between pure functions (bottom of tower), Act/Proc (middle), and Workflow (top). When a pure function call appears in a workflow context, it needs to be lifted into the workflow level. This is a significant architectural change that requires careful design.

## Verification

- ✅ All ash-engine tests pass (except pre-existing `task_870` failure)
- ✅ `fn` expressions work in function arguments
- ✅ Imported functions no longer treated as capabilities
- ✅ No regressions in existing tests

## Known Limitations

1. **Module-level function visibility in closures (TASK-1580):** Deferred to future phase. Workaround: inline the function body or pass functions as arguments.
2. **Power tower lifting:** The parser currently doesn't distinguish between tower levels. A future phase needs to implement proper lifting of pure expressions into workflow contexts.

## Closeout Criteria

- ✅ TASK-1581 implemented and tested
- ✅ TASK-1582 implemented and tested
- ✅ TASK-1580 documented and deferred
- ✅ Documentation updated
- ✅ No regressions in existing tests
