# TASK-363c: Runtime Bootstrap and Supervisor Execution

## Status: ✅ Complete

## Description

Complete bootstrap: load stdlib, compile entry, verify `main`, spawn supervisor, execute, return exit code.

Implemented as the approved narrow Phase 57 bootstrap slice in `ash-engine` and `ash-cli`:
the engine now preflights runtime stdlib sources, parses/checks/verifies entry source, executes
verified `main`, and derives the observable process exit code from `Result<(), RuntimeError>`
terminal values. The surfaced `runtime::system_supervisor` contract remains in stdlib, but the
current runtime path executes `main` directly rather than materializing a full supervisor/module
graph while parser and module-loading support stay intentionally narrow. The current `ash run`
integration is only this prerequisite bootstrap slice for obvious runtime entry sources; it does
not claim the broader TASK-366 CLI semantics work is complete.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify all blocking tasks complete**:

- TASK-363a (stdlib loading)
- TASK-363b (main verification)
- TASK-362 (system supervisor)

1. **Verify completed spec prerequisites remain satisfied**:

- S57-1 (control authority) ✅
- S57-2 (exit policy) ✅
- S57-3 (observable behavior) ✅
- S57-7 (post-spec review) ✅

## Bootstrap Flow

```rust
pub fn bootstrap(entry_path: &Path) -> Result<i32, BootstrapError> {
    // 1. Create Engine with stdlib
    let mut engine = Engine::new();
    load_ash_std_into(&mut engine)?;  // TASK-363a
    
    // 2. Load and compile entry file
    let entry_src = fs::read_to_string(entry_path)?;
    let entry_module = engine.load_module(&entry_src)?;
    
    // 3. Verify main workflow
    verify_entry_workflow(&engine, &entry_module)?;  // TASK-363b
    
    // 4. Create Args capability from CLI
    let args = create_args_capability();  // Injected, not constructed
    
    // 5. Spawn system supervisor
    // (runtime-internal spawn, not user-visible)
    let supervisor_wf = engine.get_workflow("runtime", "system_supervisor")?;
    let (handle, control_auth) = runtime_spawn(supervisor_wf, args)?;
    
    // 6. Observe supervisor completion
    // (per S57-1 control authority semantics)
    let completion = control_auth.observe_terminal_completion();
    
    // 7. Extract exit code
    let exit_code = match completion.result {
        Ok(exit_code) => exit_code,  // Int from supervisor
        Err(_) => 1,  // Supervisor itself failed
    };
    
    Ok(exit_code)
}
```

## Key Points

- **Uses Engine** (SPEC-010), not fictional Runtime
- **Uses existing spawn** from runtime internals
- **No `await`** - uses control authority observation (S57-1)
- **Exit immediately** on main completion (S57-2)

## TDD Steps

### Test 1: Full Bootstrap Success

```rust
let entry = temp_file(r#"
  use result::Result
  use result::Ok
  use runtime::RuntimeError
  
  workflow main() -> Result<(), RuntimeError> {
    Ok(())
  }
"#);

let exit_code = bootstrap(entry.path())?;
assert_eq!(exit_code, 0);
```

### Test 2: Bootstrap Returns Error Code

```rust
let entry = temp_file(r#"
  use result::Result
  use result::Err
  use runtime::RuntimeError

  workflow main() -> Result<(), RuntimeError> {
    Err(RuntimeError { exit_code: 42, message: "test" })
  }
"#);

let exit_code = bootstrap(entry.path())?;
assert_eq!(exit_code, 42);
```

### Test 3: Missing Main Fails

```rust
let entry = temp_file("workflow other() {}");
let result = bootstrap(entry.path());
assert!(result.is_err());
```

## Integration

This is the **entry point** called by CLI:

```rust
// main.rs
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let entry_file = &args[1];
    
    match bootstrap(Path::new(entry_file)) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Dependencies

- TASK-363a: Stdlib loading
- TASK-363b: Main verification
- TASK-362: System supervisor exists in stdlib
- S57-1, S57-2, S57-3: All SPEC semantics

## Blocks

- TASK-365: Exit code propagation (uses this)
- TASK-368: Integration tests (test this)

## Spec Citations

| Aspect | Spec |
|--------|------|
| Engine | SPEC-010 |
| Control authority | SPEC-004 after S57-1 |
| Exit policy | SPEC-005 after S57-2 |
| Observable behavior | SPEC-021 after S57-3 |

## Acceptance Criteria

- [x] Approved narrow bootstrap slice implemented against completed spec prerequisites
- [x] Full current bootstrap flow works
- [x] Success returns 0
- [x] RuntimeError returns correct code
- [x] Missing main fails with error
- [x] Uses Engine, no fictional APIs
- [x] No `await` syntax
- [x] Tests pass

## Est. Hours: 3-4
