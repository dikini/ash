# PLAN-158: Language Surface Fixes

**Status:** 📝 Planned
**Spec:** [SPEC-094: Language Surface Fix Specification](../spec/SPEC-094-LANGUAGE-SURFACE-FIX.md)
**Builds on:** [PLAN-157](PLAN-157-LIST-MIGRATION-HARDENING.md)
**Task range:** TASK-1580 through TASK-1584

## Goal

Fix three language surface issues that prevent idiomatic usage of pure algebraic data types and higher-order functions in Ash.

## Background

During Phase 157 (List Migration Hardening), three language limitations were identified that block full utilization of the pure ADT list implementation:

1. **Module-level functions not visible in closures**: When a closure defined in a workflow calls a module-level function, the interpreter fails with `UndefinedVariable`.
2. **Function vs capability name collision**: The lowerer treats function calls (like `reverse(list)`) as capability lookups, causing "unresolved symbolic capability" errors.
3. **Closure expression parsing limitations**: `fn(x) { x }` cannot be parsed in general expression positions like function arguments.

## Task Table

| Task | Description | Status |
|---|---|---|
| [TASK-1580](tasks/TASK-1580-closure-module-function-visibility.md) | Fix module-level function visibility inside closures | 📝 Planned |
| [TASK-1581](tasks/TASK-1581-function-vs-capability-resolution.md) | Distinguish function calls from capability calls in lowerer | 📝 Planned |
| [TASK-1582](tasks/TASK-1582-closure-expression-parsing.md) | Enable `fn` expression parsing in all expression contexts | 📝 Planned |
| [TASK-1583](tasks/TASK-1583-verification-and-regression-tests.md) | Add verification tests and ensure no regressions | 📝 Planned |
| [TASK-1584](tasks/TASK-1584-phase-158-closeout.md) | Phase closeout with documentation and changelog | 📝 Planned |

## Implementation Order

1. TASK-1581 (function vs capability) - Unblocks basic function calls
2. TASK-1582 (closure parsing) - Enables inline closures
3. TASK-1580 (module visibility) - Enables composition patterns
4. TASK-1583 (verification) - Ensures correctness
5. TASK-1584 (closeout) - Documentation

## Verification Strategy

- All existing tests must pass
- New tests must verify the three fixed patterns
- `cargo clippy --workspace` must pass
- `cargo fmt --check` must pass

## Closeout Criteria

- All three language limitations are resolved
- Tests demonstrate idiomatic ADT usage
- Documentation is updated
- CHANGELOG.md records the fixes
