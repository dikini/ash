# TASK-359: Extend Existing ash-std with Entry Point Support

## Status: ⬜ Pending

## Description

Extend the existing `ash-std` package (source rooted at `std/`) with entry point support:
`RuntimeError` type, `Args` capability interface, and system supervisor workflow.

**VALIDATION GATE:** Before implementing, verify:

- [ ] S57-4 (stdlib imports) shows ✅ Complete - confirms `use` syntax
- [ ] S57-5 (capability syntax) shows ✅ Complete - confirms usage-site `cap X` syntax and
  explicit effect-form capability use
- [ ] This task content aligns with updated SPEC - if SPEC uses different syntax than assumed here, update this task first

## Background

Workspace already has `ash-std` crate per `Cargo.toml`. This task extends it rather than creating a new crate.

## Scope

Add to the existing standard-library package under `std/`:

1. **`runtime/error.ash`**: `RuntimeError` record type
2. **`runtime/args.ash`**: `Args` capability interface
3. **`runtime/supervisor.ash`**: System supervisor workflow
4. **Module exports**: Update `lib.ash` to expose new modules

## Structure

```
std/src/
  lib.ash              # Existing - update exports
  result.ash           # Existing
  option.ash           # Existing
  runtime/             # NEW
    mod.ash            # runtime module exports
    error.ash          # RuntimeError type
    args.ash           # Args capability
    supervisor.ash     # System supervisor
```

## Deliverables

1. **RuntimeError type** in `runtime/error.ash`
2. **Args capability interface** in `runtime/args.ash`
3. **System supervisor** in `runtime/supervisor.ash`
4. **Module exports** updated in `lib.ash`
5. **Tests**: Module loads, types resolve

## TDD Steps

### Test 1: ash-std Builds with New Modules

```rust
let status = Command::new("cargo")
    .args(["build", "-p", "ash-std"])
    .status()
    .expect("ash-std builds");
assert!(status.success());
```

### Test 2: RuntimeError Available

```rust
// Use normative syntax per updated SPEC
let source = r#"
  use runtime::RuntimeError
  
  fn make_error() -> RuntimeError {
    RuntimeError { exit_code: 1, message: "error" }
  }
"#;
let result = compile_with_stdlib(source);
assert!(result.is_ok());
```

### Test 3: Args Capability Available

```rust
let source = r#"
  use result::Result
  use runtime::RuntimeError
  use runtime::Args
  
  workflow main(args: cap Args) -> Result<(), RuntimeError> {
    let first = observe Args 0;
    done;
  }
"#;
let result = compile_with_stdlib(source);
assert!(result.is_ok());
```

## Implementation Notes

**Uses existing architecture:**

- Extend `std/` (package name `ash-std`, already in workspace)
- Follow existing module patterns
- Use normative syntax from S57-4, S57-5

**Engine integration:**

- Uses existing `Engine` API (SPEC-010)
- No fictional `Runtime::new()`

## Dependencies

- S57-4: Stdlib import syntax (must be complete)
- S57-5: Capability syntax (must be complete)
- Existing `ash-std` crate structure

## Blocks

- TASK-360: RuntimeError type
- TASK-361: Args capability
- TASK-362: System supervisor

## Acceptance Criteria

- [ ] `cargo build -p ash-std` succeeds
- [ ] New modules (`runtime::*`) available
- [ ] RuntimeError type usable
- [ ] Args capability interface defined
- [ ] System supervisor workflow present
- [ ] Tests pass

## Related

- MCE-001: Entry point design (guidance only - verify against SPEC)
- Existing `ash-std` crate
- SPEC-010: Engine embedding (for runtime integration)

## Est. Hours: 4-6
