# TASK-362: Implement System Supervisor in ash-std

## Status: ⛔ Blocked

## Description

Implement the system supervisor workflow in `std/src/runtime/supervisor.ash`. This workflow spawns `main` and observes its terminal completion via control authority.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-1 (control authority)**: ✅ Complete - confirms how supervisor observes completion
2. **Verify S57-4 (imports)**: ✅ Complete - confirms `use runtime::{Args, RuntimeError}`
3. **Verify S57-5 (capabilities)**: ✅ Complete - confirms `cap Args` usage-site typing and explicit capability invocation
4. **Verify S57-6 (entry typing)**: ✅ Complete - confirms `main` signature
5. **If SPEC differs**: Update this task description to match

## Critical: No `await` Syntax

Current AST/surface has **no** `await` construct. Use "observe terminal completion via control authority" semantics, not `await handle`.

## Design (per updated SPEC)

```ash
-- std/src/runtime/supervisor.ash
use result::Result
use result::Ok
use result::Err
use runtime::RuntimeError
use runtime::Args

workflow system_supervisor(args: cap Args) -> Int {
  -- Spawn main, receive control authority (runtime-internal)
  -- (spawn semantics per SPEC-004 after S57-1)
  
  -- Observe terminal completion via control authority
  -- (observation semantics per SPEC-004 after S57-1)
  
  -- Extract result from completion payload
  -- result : Result<(), RuntimeError>
  
  -- Form exit code
  match result {
    Ok(()) => {
      -- Check obligations discharged (per SPEC)
      0
    }
    Err(runtime_error) => {
      runtime_error.exit_code  -- Record access, not tuple destructuring
    }
  }
}
```

**Note**: Uses **record access** (`runtime_error.exit_code`) matching TASK-360's record-style RuntimeError.

## Requirements

1. **Workflow definition** in `runtime/supervisor.ash`
2. **Spawns `main`** using normative spawn semantics
3. **Observes completion** via control authority (not `await`)
4. **Returns Int** (exit code to runtime)

## Implementation Sketch (Runtime-Internal)

The supervisor's observation of completion is **runtime-internal**, not user-visible syntax:

```rust
// Pseudocode - runtime implements this, not Ash surface
let (main_instance, control_auth) = spawn(main_workflow);
let completion_payload = control_auth.observe_terminal_completion();
let exit_code = supervisor_form_exit_code(completion_payload);
```

## TDD Steps

### Test 1: Supervisor Compiles

```rust
let source = r#"
  use runtime::supervisor
  
  -- Verify type: system_supervisor : (cap Args) -> Int
  fn check() {}
"#;
let result = compile_with_stdlib(source);
assert!(result.is_ok());
```

### Test 2: Integration (Rust side)

```rust
// Test via Engine API (SPEC-010), not fictional Runtime::new()
let engine = Engine::new();
let args = vec!["test".to_string()];
let exit_code = engine.run_supervisor(args);
assert_eq!(exit_code, 0);  // When main returns Ok(())
```

## Implementation Notes

- **Location**: `std/src/runtime/supervisor.ash`
- **Uses**: Normative spawn, normative observation (per S57-1)
- **No `await`**: Not in surface language
- **Record access**: `runtime_error.exit_code` (matches TASK-360)

## Dependencies

- TASK-359: ash-std structure
- TASK-360: RuntimeError type
- TASK-361: Args capability
- S57-1: Control authority semantics (critical)
- S57-4, S57-5, S57-6: Syntax

## Blocks

- TASK-363c: Bootstrap spawns supervisor

## Spec Citations

| Aspect | Spec |
|--------|------|
| Spawn semantics | SPEC-004 after S57-1 |
| Control authority | SPEC-004 after S57-1 |
| Entry point contract | SPEC-003/022 after S57-6 |
| Capability params | SPEC-017 after S57-5 |

## Acceptance Criteria

- [ ] S57-1, S57-4, S57-5, S57-6 show ✅ Complete (VALIDATION GATE)
- [ ] Supervisor workflow in stdlib
- [ ] Spawns `main` per normative spawn
- [ ] Observes completion (no `await` syntax)
- [ ] Returns Int exit code
- [ ] Record-style RuntimeError access (not tuple)
- [ ] Tests pass

## Est. Hours: 4-6
