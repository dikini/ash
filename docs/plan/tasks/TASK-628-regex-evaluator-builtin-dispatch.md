# TASK-628: Move regex dispatch to evaluator builtin table

## Status: ✅ Complete

## Description
Move regex execution from the legacy capability dispatch path to the evaluator builtin dispatch path. After this task, `regex::find`, `regex::matches`, and `regex::replace` should execute as pure builtins without going through capability-provider wiring.

## Specification Reference
- SPEC-BUILTIN-FN: Sections 7, 8, 9.5
- PLAN-BUILTIN-FN: Track E / TASK-628
- DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN

## Dependencies
- ✅ TASK-621: Runtime builtin dispatch table
- ✅ TASK-627: Rewrite `std/src/regex.ash` with `builtin fn` declarations

## Requirements
1. Add regex builtin dispatch entries in the evaluator/runtime builtin table.
2. Route `regex::find`, `regex::matches`, and `regex::replace` through pure evaluator dispatch.
3. Use the existing `regex` crate behavior equivalently to the prior provider implementation.
4. Preserve clear errors for invalid patterns.
5. Remove any remaining regex dependence on capability dispatch in the builtin call path.

## TDD Steps
### Step 1: Write Tests (Red)
- Add tests that call `regex::find`, `regex::matches`, and `regex::replace` through builtin dispatch.
- Verify failure before wiring if the path is not yet implemented.

### Step 2: Implement (Green)
- Add regex match arms / dispatch entries in the evaluator builtin path.
- Port logic from the provider implementation as needed.

### Step 3: Verify
- Regex builtin tests pass through evaluator dispatch without capability-provider dependence.

## Verification Steps
- [x] evaluator dispatch supports `regex::find`
- [x] evaluator dispatch supports `regex::matches`
- [x] evaluator dispatch supports `regex::replace`
- [x] invalid-pattern behavior is tested
- [x] targeted interp/engine tests pass

## Notes
This task is the runtime half of the regex migration. Provider deletion is deferred to TASK-629 after positive end-to-end proof in TASK-630.
