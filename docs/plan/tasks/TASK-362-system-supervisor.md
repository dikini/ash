# TASK-362: Implement System Supervisor in ash-std

## Status: ✅ Complete

## Description

Complete the stdlib-visible `system_supervisor` contract in `std/src/runtime/supervisor.ash`. TASK-362 now covers the canonical `system_supervisor(args: cap Args) -> Int` surface, the documented terminal `Result<(), RuntimeError>` completion shape, and focused parser regressions for the workflow definition.

The normative runtime semantics are unchanged: the runtime still owns spawning `main(args)` and observing its terminal completion via control authority. That bootstrap/execution behavior remains explicitly deferred to [TASK-363c](TASK-363c-runtime-bootstrap-execution.md), so this task does **not** claim the runtime path is already wired.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-1 (control authority)**: ✅ Complete - confirms how supervisor observes completion
2. **Verify S57-4 (imports)**: ✅ Complete - confirms `use runtime::{Args, RuntimeError}`
3. **Verify S57-5 (capabilities)**: ✅ Complete - confirms `cap Args` usage-site typing and explicit capability invocation
4. **Verify S57-6 (entry typing)**: ✅ Complete - confirms `main` signature
5. **If SPEC differs**: Update this task description to match

## Current Completion Scope

- ✅ `std/src/runtime/supervisor.ash` exposes the canonical `system_supervisor(args: cap Args) -> Int` contract.
- ✅ The stdlib-visible body documents the runtime-provided `Result<(), RuntimeError>` completion payload and the parser-feasible exit-code shaping intent:

```ash
if let Err { error: RuntimeError { exit_code: code, message: _ } } = completion then code else 0
```

- ✅ Spawn/completion observation stays runtime-internal and is deferred to TASK-363c.
- ✅ Focused parser regressions cover both the stdlib surface and the workflow-body parse.

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
    Ok { value: _ } => {
      -- Check obligations discharged (per SPEC)
      0
    }
    Err { error: RuntimeError { exit_code: code, message: _ } } => {
      code  -- Nested variant destructuring, not field access
    }
  }
}
```

**Note**: Uses nested variant destructuring because `RuntimeError` is a single-variant ADT and the current expression pipeline does not support direct field access.

## Requirements

1. **Workflow definition** in `runtime/supervisor.ash`
2. **Exposes the canonical contract** `system_supervisor(args: cap Args) -> Int`
3. **Documents** the normative spawn/completion semantics as runtime-internal behavior reserved for TASK-363c
4. **Returns Int** via parser-feasible exit-code shaping over the terminal `Result<(), RuntimeError>` contract

## Implementation Sketch (Runtime-Internal)

The supervisor's observation of completion is **runtime-internal**, not user-visible syntax:

```rust
// Pseudocode - runtime implements this, not Ash surface
let (main_instance, control_auth) = spawn(main_workflow);
let completion_payload = control_auth.observe_terminal_completion();
let exit_code = supervisor_form_exit_code(completion_payload);
```

## TDD Steps

### Test 1: Stdlib Surface Regression

```rust
cargo test -p ash-parser --quiet --test stdlib_surface runtime_stdlib_surface_is_exposed -- --exact
```

Verifies the stdlib-visible supervisor contract, imports, `Result<(), RuntimeError>` documentation, and exit-code shaping markers.

### Test 2: Workflow Body Parse Regression

```rust
cargo test -p ash-parser --quiet --test stdlib_parsing test_runtime_supervisor_workflow_definition_parses -- --exact
```

Verifies the workflow body remains parser-feasible without introducing fake runtime bootstrap bindings or `await` syntax.

### Downstream Runtime Verification

End-to-end bootstrap execution remains owned by TASK-363c once the runtime wires stdlib loading, `main` verification, spawning, and terminal completion observation.

## Implementation Notes

- **Location**: `std/src/runtime/supervisor.ash`
- **Current task output**: Canonical stdlib-visible contract and exit-code shaping surface
- **Normative runtime semantics**: Spawn/observation remain the runtime's responsibility (per S57-1) and are not surfaced as new syntax here
- **No `await`**: Not in surface language
- **Pattern extraction**: destructure `Err { error: RuntimeError { exit_code: code, message: _ } }` to obtain the exit code from the `RuntimeError` payload

## Dependencies

- TASK-359: ash-std structure
- TASK-360: RuntimeError type
- TASK-361: Args capability
- S57-1: Control authority semantics (critical)
- S57-4, S57-5, S57-6: Syntax

## Downstream Follow-up

- TASK-363c: Bootstrap spawns `system_supervisor`, provides the runtime completion payload, and observes terminal completion at runtime

## Spec Citations

| Aspect | Spec |
|--------|------|
| Spawn semantics | SPEC-004 after S57-1 |
| Control authority | SPEC-004 after S57-1 |
| Entry point contract | SPEC-003/022 after S57-6 |
| Capability params | SPEC-017 after S57-5 |

## Acceptance Criteria

- [x] S57-1, S57-4, S57-5, S57-6 show ✅ Complete (VALIDATION GATE)
- [x] `std/src/runtime/supervisor.ash` exposes the canonical `system_supervisor(args: cap Args) -> Int` contract
- [x] The stdlib-visible supervisor surface documents the terminal `Result<(), RuntimeError>` shape without claiming runtime bootstrap wiring is complete
- [x] Normative spawn/completion semantics remain documented as runtime-internal behavior reserved for TASK-363c
- [x] The workflow body preserves nested `RuntimeError` exit-code destructuring intent and avoids `await` syntax
- [x] Focused parser regressions pass for the stdlib surface and workflow-body parse

## Est. Hours: 4-6
