# TASK-368a: Minimum Entry Point Tests

## Status: ⛔ Blocked

## Description

Minimum integration tests for entry point execution: success path, error code, missing main.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify all 57A complete**: S57-1 through S57-7 show ✅ Complete
2. **Verify S57-3 (observable)**: Confirms what's testable
3. **This is MINIMUM core**: Does NOT test stdlib features (stdout, etc.)

## Test Coverage (Minimum)

### Test 1: Success Path

```rust
#[test]
fn entry_point_success_exits_zero() {
    let entry = r#"
        use result::Result
        use result::Ok
        use runtime::RuntimeError
        
        workflow main() -> Result<(), RuntimeError> {
            Ok(())
        }
    "#;
    
    let exit_code = run_program(entry);
    assert_eq!(exit_code, 0);
}
```

### Test 2: Error Code Propagation

```rust
#[test]
fn runtime_error_returns_exit_code() {
    let entry = r#"
        use result::Result
        use result::Err
        use runtime::RuntimeError
        
        workflow main() -> Result<(), RuntimeError> {
            Err(RuntimeError { exit_code: 42, message: "test" })
        }
    "#;
    
    let exit_code = run_program(entry);
    assert_eq!(exit_code, 42);
}
```

### Test 3: Missing Main Detected

```rust
#[test]
fn missing_main_reports_error() {
    let entry = "workflow other() {}";
    
    let (exit_code, output) = run_program_with_output(entry);
    assert_eq!(exit_code, 1);
    assert!(output.stderr.contains("no 'main' workflow"));
}
```

### Test 4: Args Capability Is Available at Entry

```rust
#[test]
fn args_capability_receives_args() {
    let entry = r#"
        use result::Result
        use result::Ok
        use runtime::RuntimeError
        use runtime::Args
        
        workflow main(args: cap Args) -> Result<(), RuntimeError> {
            let first = observe Args 0;
            let second = observe Args 1;
            Ok(())
        }
    "#;
    
    let exit_code = run_program_with_args(entry, &["hello", "world"]);
    assert_eq!(exit_code, 0);
}
```

## Test Infrastructure

```rust
fn run_program(source: &str) -> i32 {
    let temp = TempDir::new().unwrap();
    let file = temp.child("main.ash");
    file.write_str(source).unwrap();
    
    Command::new("ash")
        .args(["run", file.path()])
        .status()
        .unwrap()
        .code()
        .unwrap()
}

fn run_program_with_args(source: &str, args: &[&str]) -> i32 {
    let temp = TempDir::new().unwrap();
    let file = temp.child("main.ash");
    file.write_str(source).unwrap();
    
    let mut cmd = Command::new("ash");
    cmd.args(["run", file.path(), "--"]);
    cmd.args(args);
    cmd.status().unwrap().code().unwrap()
}
```

## Dependencies

- All 57A tasks including S57-7: ✅ Complete (VALIDATION GATE)
- 57B prerequisites through TASK-367: complete in dependency order before these minimum tests begin
- Test infrastructure: `assert_cmd`, `tempfile`

## Acceptance Criteria

- [ ] All 57A including S57-7 show ✅ Complete (VALIDATION GATE)
- [ ] Success test passes
- [ ] Error code test passes
- [ ] Missing main test passes
- [ ] Args capability test passes
- [ ] Tests run in CI

## Est. Hours: 2-3

## Extended Tests

For stdout, assertions, and other extended features, see TASK-368b (deferred).
