# TASK-631A: Remove hardcoded builtin type entries covered by D1

## Status: ✅ Complete

## Description
Remove type-environment hardcoded builtin registrations that are now covered by stdlib declaration files from Track D1. In the landed Phase 92 state this meant deleting the remaining string entries and confirming there were no lingering record entries to remove.

## Specification Reference
- SPEC-BUILTIN-FN: Sections 6, 9.5
- PLAN-BUILTIN-FN: Track F / TASK-631A

## Dependencies
- ✅ TASK-623: Create `std/src/string.ash`
- ✅ TASK-626: Declare record operation builtins

## Requirements
1. Remove hardcoded type entries for string builtins now covered by declarations.
2. Remove any hardcoded type entries for record builtins now covered by declarations, or confirm they are already absent.
3. Preserve remaining hardcoded entries that are still legitimately blocked/deferred (e.g. polymorphic list ops, type predicates if still needed).
4. Verify type resolution still succeeds through module declarations rather than hardcoded registration.

## TDD Steps
### Step 1: Baseline
- Confirm current type-env tests pass before removal.

### Step 2: Implement
- Remove only the D1-covered entries from `add_builtin_functions()` or equivalent.

### Step 3: Verify
- Run ash-typeck tests and targeted engine tests covering string/record builtins.

## Verification Steps
- [x] D1-covered string entries removed
- [x] D1-covered record entries confirmed absent (no extra removal needed)
- [x] blocked/deferred builtin registrations not incorrectly removed
- [x] typechecker/tests still pass

## Notes
TASK-631B remains blocked on deferred polymorphic builtin work. In the landed
Phase 92 state, the hardcoded string entries were removed while record entries
required no deletion because they were already absent from the type environment.
