# TASK-365: Exit Code Propagation to OS

## Status: ⛔ Blocked

## Description

Propagate exit code from supervisor completion to OS process exit.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-2 (CLI exit policy)**: ✅ Complete - confirms exit-immediately
2. **Verify S57-3 (observable behavior)**: ✅ Complete - confirms what's observable
3. **If SPEC differs**: Update propagation logic

## Exit Code Mapping (per S57-2/S57-3)

| Condition | Exit Code |
|-----------|-----------|
| `main` returns `Ok(())` | 0 |
| `main` returns `Err(RuntimeError { exit_code: N, .. })` | N |
| Bootstrap/verification error | 1 |
| Supervisor crash (implementation error) | 1 |

## Implementation

```rust
// In main.rs or CLI layer
fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("usage: ash run <file>");
        std::process::exit(1);
    }
    
    let entry_file = &args[1];
    
    // Bootstrap (TASK-363c)
    let exit_code = match bootstrap(Path::new(entry_file)) {
        Ok(code) => code,
        Err(BootstrapError::NoMainWorkflow) => {
            eprintln!("error: entry file has no 'main' workflow");
            1
        }
        Err(BootstrapError::WrongReturnType { expected, found }) => {
            eprintln!("error: 'main' has wrong return type");
            eprintln!("  expected: {}", expected);
            eprintln!("  found: {}", found);
            1
        }
        Err(e) => {
            eprintln!("error: {}", e);
            1
        }
    };
    
    std::process::exit(exit_code);
}
```

## Key Behaviors

- **Exit immediately** on main completion (S57-2)
- **Descendants** not waited for (S57-3)
- **Error code 1** for all bootstrap failures
- **User code** controls exit via `RuntimeError.exit_code`

## TDD Steps

### Test 1: Success Returns 0

```rust
let entry = r#"
    use result::Result
    use result::Ok
    use runtime::RuntimeError

    workflow main() -> Result<(), RuntimeError> { Ok(()) }
"#;
let code = run_and_get_exit_code(entry);
assert_eq!(code, 0);
```

### Test 2: RuntimeError Returns Code

```rust
let entry = r#"
    use result::Result
    use result::Err
    use runtime::RuntimeError

  workflow main() -> Result<(), RuntimeError> {
    Err(RuntimeError { exit_code: 42, message: "test" })
  }
"#;
let code = run_and_get_exit_code(entry);
assert_eq!(code, 42);
```

### Test 3: Bootstrap Error Returns 1

```rust
let entry = "workflow other() {}";  // No main
let code = run_and_get_exit_code(entry);
assert_eq!(code, 1);
```

## Dependencies

- S57-2: Exit policy
- S57-3: Observable behavior
- TASK-363c: Bootstrap returns exit code

## Spec Citations

| Aspect | Spec |
|--------|------|
| Exit policy | SPEC-005 after S57-2 |
| Observable | SPEC-021 after S57-3 |
| Error format | SPEC-005 |

## Acceptance Criteria

- [ ] S57-2, S57-3 show ✅ Complete (VALIDATION GATE)
- [ ] Success → exit 0
- [ ] RuntimeError → exit code from error
- [ ] Bootstrap errors → exit 1
- [ ] Error messages to stderr
- [ ] Tests pass

## Est. Hours: 1-2
