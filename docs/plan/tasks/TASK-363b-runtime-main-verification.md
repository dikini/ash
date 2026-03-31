# TASK-363b: Runtime Entry Workflow Verification

## Status: ⛔ Blocked

## Description

Verify entry file contains `main` workflow with correct signature using type information from Engine.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-6 (entry typing)**: ✅ Complete - confirms `main` contract
2. **Verify S57-5 (capability params)**: ✅ Complete - confirms capability-only params
3. **If SPEC differs**: Update verification criteria

## Verification Contract (per S57-6)

Check entry workflow:

- **Name**: `main`
- **Return type**: `Result<(), RuntimeError>` (exact)
- **Parameters**: Only usage-site capability types (`cap X`)

## Implementation Sketch

```rust
pub fn verify_entry_workflow(
    engine: &Engine,
    entry_module: &Module
) -> Result<(), VerificationError> {
    // 1. Find workflow named "main"
    let main = entry_module.find_workflow("main")
        .ok_or(VerificationError::NoMain)?;
    
    // 2. Check return type
    let expected_return = engine.resolve_type("Result<(), RuntimeError>")?;
    if main.return_type != expected_return {
        return Err(VerificationError::WrongReturnType { ... });
    }
    
    // 3. Check parameters are capabilities only
    for param in &main.params {
        if !is_capability_type(&param.typ) {
            return Err(VerificationError::NonCapabilityParam { ... });
        }
    }
    
    Ok(())
}
```

## Error Cases

| Error | Message |
|-------|---------|
| NoMain | "entry file has no 'main' workflow" |
| WrongReturnType | "expected Result<(), RuntimeError>, found {X}" |
| NonCapabilityParam | "parameter '{name}' must be capability type" |

## TDD Steps

### Test 1: Accepts Valid Main

```rust
let engine = Engine::with_stdlib();
let entry = r#"
    use result::Result
    use result::Ok
    use runtime::RuntimeError

    workflow main() -> Result<(), RuntimeError> { Ok(()) }
"#;
let module = engine.load_module(entry)?;
assert!(verify_entry_workflow(&engine, &module).is_ok());
```

### Test 2: Rejects Missing Main

```rust
let entry = "workflow other() {}";
let result = verify_entry_workflow(&engine, &module);
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("no 'main'"));
```

### Test 3: Rejects Non-Capability Param

```rust
let entry = r#"
    use result::Result
    use runtime::RuntimeError

    workflow main(n: Int) -> Result<(), RuntimeError> { done; }
"#;
let result = verify_entry_workflow(&engine, &module);
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("capability"));
```

## Dependencies

- TASK-363a: Engine with stdlib loaded
- S57-6: Entry typing contract
- S57-5: Capability type distinction

## Blocks

- TASK-363c: Bootstrap calls verification

## Spec Citations

| Aspect | Spec |
|--------|------|
| Entry contract | SPEC-003/022 after S57-6 |
| Capabilities | SPEC-017 after S57-5 |
| Error format | SPEC-005 after S57-2 |

## Acceptance Criteria

- [ ] S57-5, S57-6 show ✅ Complete (VALIDATION GATE)
- [ ] Detects missing `main`
- [ ] Verifies return type
- [ ] Verifies capability-only params
- [ ] Clear error messages
- [ ] Tests pass

## Est. Hours: 2-3
