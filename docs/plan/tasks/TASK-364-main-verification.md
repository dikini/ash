# TASK-364: Verify Entry Workflow Type Signature

## Status: ⛔ Blocked

## Description

Type-level verification that entry workflow `main` conforms to entry contract: correct name, return type, and parameter types.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-6 (entry typing)**: ✅ Complete - confirms exact contract
2. **Confirm these decisions**:
   - Is `Result<(), RuntimeError>` **exact** or `Result<T, RuntimeError>` for any T?
   - Are **zero parameters** allowed?
   - Are **arbitrary capability parameters** allowed or only specific ones?

**If S57-6 differs from below, update this task.**

## Entry Contract (assumed - verify in S57-6)

| Aspect | Constraint |
|--------|------------|
| Name | `main` (exact) |
| Return | `Result<(), RuntimeError>` (exact - unit success payload) |
| Params | Zero or more, **capability types only** |

## Verification Logic

```rust
fn verify_main_signature(wf: &Workflow) -> Result<(), TypeError> {
    // 1. Name check
    if wf.name != "main" {
        return Err(TypeError::NoMainWorkflow);
    }
    
    // 2. Return type check (exact match)
    let expected = parse_type("Result<(), RuntimeError>");
    if wf.return_type != expected {
        return Err(TypeError::WrongReturnType {
            expected,
            found: wf.return_type.clone(),
        });
    }
    
    // 3. Parameter check - all capabilities
    for param in &wf.params {
        if !is_capability_type(&param.typ) {
            return Err(TypeError::NonCapabilityParam {
                name: param.name.clone(),
                typ: param.typ.clone(),
            });
        }
    }
    
    Ok(())
}
```

## TDD Steps

### Test 1: Valid Main (No Params)
```rust
let wf = parse_workflow("workflow main() -> Result<(), RuntimeError> { Ok(()) }");
assert!(verify_main_signature(&wf).is_ok());
```

### Test 2: Valid Main (With Args)
```rust
let wf = parse_workflow(r#"
  workflow main(args: capability Args) -> Result<(), RuntimeError> { Ok(()) }
"#);
assert!(verify_main_signature(&wf).is_ok());
```

### Test 3: Wrong Return Type
```rust
let wf = parse_workflow("workflow main() -> Int { 42 }");
let err = verify_main_signature(&wf).unwrap_err();
assert!(matches!(err, TypeError::WrongReturnType { .. }));
```

### Test 4: Non-Capability Param
```rust
let wf = parse_workflow(r#"
  workflow main(n: Int) -> Result<(), RuntimeError> { Ok(()) }
"#);
let err = verify_main_signature(&wf).unwrap_err();
assert!(matches!(err, TypeError::NonCapabilityParam { .. }));
```

## Precisions Needed from S57-6

Before implementing, confirm:

1. **Exact return**: `Result<(), RuntimeError>` only? Or generic success payload?
2. **Zero params allowed**: Is `main()` valid or must have at least `capability Args`?
3. **Multiple caps**: Is `main(a: A, b: B, c: C)` valid?
4. **Error messages**: Standard format per SPEC-005?

## Dependencies

- S57-6: Entry typing contract (critical)
- S57-5: Capability type distinction
- Type checker: For `is_capability_type()`

## Blocks

- TASK-363b: Calls this verification
- TASK-367: Error message format

## Spec Citations

| Aspect | Spec |
|--------|------|
| Entry contract | SPEC-003/022 after S57-6 |
| Capabilities | SPEC-017 after S57-5 |
| Error format | SPEC-005 after S57-2 |

## Acceptance Criteria

- [ ] S57-6 shows ✅ Complete with precise contract (VALIDATION GATE)
- [ ] Detects wrong name
- [ ] Detects wrong return type
- [ ] Detects non-capability params
- [ ] Error messages clear
- [ ] Tests pass

## Est. Hours: 2-3
