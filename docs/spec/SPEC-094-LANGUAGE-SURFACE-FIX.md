# SPEC-094: Language Surface Fix Specification

**Status:** Implemented MVP (Phase 158; Phase 176 reconciled deferred list/surface tail)
**Scope:** Fix three language surface issues blocking idiomatic pure ADT usage
**Related:** PLAN-153 (List Builtin to Stdlib), PLAN-157 (List Migration Hardening)

## Problem Statement

Three language limitations prevent idiomatic usage of pure algebraic data types and higher-order functions:

1. **Module-level function visibility in closures**: Module-level functions are not accessible from within closures defined in workflows.
2. **Function vs capability name resolution**: The lowerer conflates function calls with capability calls, treating imported functions as symbolic capabilities.
3. **Closure expression parsing**: `fn` literals cannot be parsed in general expression positions (e.g., as function arguments).

## Goals

1. Enable module-level functions to be called from within closures
2. Distinguish function calls from capability calls in the lowerer
3. Allow `fn` expressions anywhere an expression is expected

## Non-Goals

- No changes to capability semantics
- No changes to the ADT representation
- No performance optimizations

## Acceptance Criteria

- `map(list, fn(x) { x })` parses and executes correctly
- `compose(x) { add_one(mul_two(x)) }` works when `add_one` and `mul_two` are module-level functions
- `reverse(list)` works when `reverse` is imported from `list`
- All existing tests continue to pass
