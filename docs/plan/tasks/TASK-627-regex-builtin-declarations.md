# TASK-627: Rewrite `std/src/regex.ash` with `builtin fn` Declarations

## Status: ✅ Complete

## Description
Replace the current regex stdlib wrappers that use `act execute ...` through the capability-provider path with pure `pub builtin fn` declarations. This converts `std/src/regex.ash` into the canonical declaration surface for regex builtins and unblocks the runtime migration to evaluator dispatch.

## Specification Reference
- SPEC-BUILTIN-FN: Sections 2, 4, 5, 8, 9.5
- PLAN-BUILTIN-FN: Track E / TASK-627
- DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN

## Dependencies
- ✅ TASK-617: Module-level snippet extraction for `builtin fn`
- ✅ TASK-621: Runtime builtin dispatch table

## Requirements

### Functional Requirements
1. Rewrite `std/src/regex.ash` to use only `pub builtin fn` declarations.
2. Declare exactly these regex builtins:
   - `find(pattern: String, text: String) -> Option<String>`
   - `matches(pattern: String, text: String) -> Bool`
   - `replace(pattern: String, replacement: String, text: String) -> String`
3. Remove the old `pub fn` bodies that used `act execute Regex.*`.
4. Ensure the file parses and the module loader exports the regex callables.
5. Ensure `use regex::{find}` resolves at module-load time.

## TDD Steps

### Step 1: Write Tests (Red)
- Confirm the current limitation/boundary around `regex.ash` and import resolution.
- Add or adapt tests to prove `std/src/regex.ash` is exported as builtin callables after the rewrite.

### Step 2: Implement (Green)
- Rewrite `std/src/regex.ash` with `pub builtin fn` declarations only.
- Preserve the existing module surface and names.

### Step 3: Verify
- `cargo test -p ash-engine --test builtin_fn_e2e_import`
- Regex module import resolution tests pass.

## Verification Steps
- [x] `std/src/regex.ash` contains only `pub builtin fn` declarations for regex ops
- [x] module loader recognizes the regex builtin declarations
- [x] `use regex::{find}` resolves successfully
- [x] targeted engine tests pass

## Notes
This task changes declaration surface only. Actual runtime dispatch moves in TASK-628.
