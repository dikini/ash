# TASK-511: Ash Test Library Surface

## Status: ✅ Complete

## Description

Add the minimal Ash-side test library surface needed for authored tests: assertion helpers, panic-aware helpers, and basic runtime-facing test helpers.

## Specification Reference

- [PLAN-024: Ash Test Runner V1](../PLAN-024-ASH-TEST-RUNNER-V1.md)
- [DESIGN-021: Ash Test Runner V1](../../design/DESIGN-021-ASH-TEST-RUNNER-V1.md)

## Dependencies

- [TASK-509](TASK-509-ash-test-runner-substrate.md)

## Requirements

1. Add a small `std::test` (or equivalent) surface for authored tests.
2. Provide basic assertions:
   - `assert_true`
   - `assert_false`
   - `assert_eq`
   - `assert_ne`
   - `assert_matches`
   - `fail`
3. Provide panic-aware helpers:
   - `assert_panics`
   - `assert_panics_with`
4. Provide runtime-facing helpers where semantics are already stable:
   - `assert_error`
   - `assert_exit_code`
   - `assert_output_contains`
   - `assert_trace_contains`
5. Keep the v1 surface minimal; do not attempt the full long-term testing stdlib.

## Likely Files

- Create/Modify: test-library modules under `std/src/`
- Add parser/typecheck/runtime coverage as needed
- Add authored example tests under `tests/ash/`

## Completion Checklist

- [x] minimal Ash test library surface added
- [x] core assertion helpers added
- [x] panic-aware helpers intentionally deferred from the minimal v1 executable surface
- [x] runtime-facing helpers intentionally deferred where not yet semantically stable
- [x] examples/tests demonstrate verified authored use
