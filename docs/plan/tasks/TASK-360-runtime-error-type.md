# TASK-360: Define RuntimeError Type in ash-std

## Status: ✅ Complete

## Description

Define the canonical single-variant `RuntimeError` ADT in `std/src/runtime/error.ash` for entry-point error reporting.

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

**Style:** Canonical single-variant ADT syntax with a record payload (consistent with the ash-std surface tests).

## Requirements

1. **Type definition** in `runtime/error.ash`
2. **Module export** in `runtime/mod.ash` and `lib.ash`
3. **Documentation** for entry point use

## TDD Steps

### Test 1: Type Definition Compiles

```rust
let source = r#"
  use runtime::RuntimeError
  
  fn make_error() -> RuntimeError {
    RuntimeError { exit_code: 1, message: "failed" }
  }
"#;
let result = parse_and_typecheck(source);
assert!(result.is_ok());
```

### Test 2: Used in Entry Point Signature

```rust
let source = r#"
  use result::Result
  use result::Ok
  use result::Err
  use runtime::RuntimeError
  
  workflow main() -> Result<(), RuntimeError> {
    Err(RuntimeError { exit_code: 1, message: "error" })
  }
"#;
let result = parse_and_typecheck(source);
assert!(result.is_ok());
```

## Implementation Notes

- **Location**: `std/src/runtime/error.ash`
- **Export**: Add to `runtime/mod.ash` and `lib.ash`
- **Style**: Record ADT (matches existing standard-library patterns)

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
- [x] Type definition compiles
- [x] Can construct RuntimeError values
- [x] Can use in workflow return types
- [x] Tests pass

## Est. Hours: 2-3
