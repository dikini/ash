# TASK-360: Define RuntimeError Type in ash-std

## Status: ✅ Complete

## Description

Define the canonical single-variant `RuntimeError` ADT in `std/src/runtime/error.ash` for entry-point error reporting.

Current verification scope for this task is the stdlib ADT surface itself in the parser, direct constructor compatibility in the current typechecker pipeline, and nested variant-pattern extraction in the interpreter pipeline. Full designated-entry-workflow verification remains downstream work for [TASK-364](TASK-364-main-verification.md) and [TASK-368a](TASK-368a-entry-point-tests-minimum.md).

**Validation status:**

1. **Verify S57-4 (import syntax)**: ✅ Complete - use `use runtime::RuntimeError`
2. **Verify TYPES-001 or SPEC**: ✅ Complete - canonical ADT form is a single-variant record payload
3. **Verify task description matches the normative syntax**: ✅ Complete
4. **Verify no syntax mismatch**: ✅ Complete - `RuntimeError { exit_code, message }` matches the exported stdlib surface

## Design (per updated SPEC)

```ash
-- std/src/runtime/error.ash
pub type RuntimeError = RuntimeError {
  exit_code: Int,
  message: String
};
```

**Style:** Canonical single-variant ADT syntax with a record payload, consumed downstream through constructor expressions and variant-pattern destructuring rather than direct field access.

## Requirements

1. **Type definition** in `runtime/error.ash`
2. **Module export** in `runtime/mod.ash` and `lib.ash`
3. **Documentation** for entry point use

## TDD Steps

### Test 1: Constructor Form Typechecks

```rust
let result = check_runtime_error_constructor();
assert!(result.is_ok());
```

### Test 2: Composes with Result and Runtime Pattern Extraction

```rust
assert!(check_runtime_error_inside_err_constructor().is_ok());
assert!(match_runtime_error_exit_code_pattern().is_ok());
```

## Implementation Notes

- **Location**: `std/src/runtime/error.ash`
- **Export**: Add to `runtime/mod.ash` and `lib.ash`
- **Style**: Record-payload ADT consumed via constructor expressions and nested variant patterns

## Dependencies

- TASK-359: ash-std structure extended
- S57-4: Import syntax (`use runtime::RuntimeError`)

## Blocks

- TASK-362: System supervisor returns RuntimeError

## Spec Citations

| Aspect | Spec |
|--------|------|
| Record ADT syntax | SPEC-020 or TYPES-001 |
| Import syntax | SPEC-012 after S57-4 |
| Entry point return | SPEC-003/022 after S57-6 |

## Acceptance Criteria

- [x] S57-4, TYPES-001 show ✅ Complete (validation gate)
- [x] Type definition parses and exports from the stdlib surface
- [x] `RuntimeError { exit_code, message }` constructor expressions typecheck in the ADT pipeline
- [x] `RuntimeError` values compose inside `Err { error: ... }` constructors, and runtime pattern matching can extract `exit_code` from the nested ADT payload
- [x] Tests pass

## Est. Hours: 2-3
