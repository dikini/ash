# TASK-368b: Extended Entry Point Tests (Deferred)

## Status: ⛔ Deferred / Future

## Description

Extended integration tests for entry point: stdout capture, assertions, complex scenarios.

**Deferred to future phase** - not required for minimum core.

## Test Coverage (Extended - Not Minimum)

### Test: Stdout Capture

```rust
#[test]
fn program_writes_stdout() {
    let entry = r#"
        use result::Result
        use result::Ok
        use runtime::RuntimeError
        use io::Stdout  -- Requires io module in stdlib
        
        workflow main(stdout: cap Stdout) -> Result<(), RuntimeError> {
            send Stdout "Hello, World!";
            Ok(())
        }
    "#;
    
    let (exit_code, output) = run_program_with_output(entry);
    assert_eq!(exit_code, 0);
    assert!(output.stdout.contains("Hello, World!"));
}
```

### Test: Assertions in Ash

```rust
#[test]
fn ash_assertions_work() {
    let entry = r#"
        use result::Result
        use result::Ok
        use runtime::RuntimeError
        use test::Assert  -- Requires test module
        
        workflow main() -> Result<(), RuntimeError> {
            assert_eq(1 + 1, 2);
            Ok(())
        }
    "#;
    
    let exit_code = run_program(entry);
    assert_eq!(exit_code, 0);
}
```

### Test: Large Exit Code

```rust
#[test]
fn large_exit_code() {
    let entry = r#"
        use runtime::RuntimeError

        workflow main() -> Result<(), RuntimeError> {
            Err(RuntimeError { exit_code: 255, message: "max" })
        }
    "#;
    
    let exit_code = run_program(entry);
    assert_eq!(exit_code, 255);
}
```

## Why Deferred

These tests require:

- `io::Stdout` capability (not in minimum core)
- Test assertion helpers (not in minimum core)
- A finalized stdlib `io` capability surface for concrete send/set examples
- More stdlib surface area

Minimum core (TASK-368a) tests the entry point mechanism without requiring full stdlib.

## When to Revisit

- After minimum core complete
- After io module added to stdlib
- After test helpers available

## Est. Hours: 2-3 (when revisited)
