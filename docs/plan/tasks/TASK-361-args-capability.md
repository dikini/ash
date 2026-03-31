# TASK-361: Define Args Capability Interface

## Status: ⛔ Blocked

## Description

Define the `Args` capability interface in `std/src/runtime/args.ash` for accessing command-line arguments.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-4 (import syntax)**: ✅ Complete - confirms `use runtime::Args`
2. **Verify S57-5 (capability syntax)**: ✅ Complete - confirms usage-site `cap Args` typing and explicit effect-form capability use
3. **If SPEC uses different syntax**: Update this task description to match

## Design (per updated SPEC)

```ash
-- std/src/runtime/args.ash
pub capability Args : observe (index: Int) returns Option<String>
```

**Note on CLI args**: Do not assume method-style access or a C-style `argv[0]` slot in the normative interface. Align the injected capability with the `ash run <file> [-- <args>...]` contract and the explicit observation form `observe Args <index>`.

## Requirements

1. **Capability interface** in `runtime/args.ash`
2. **Module export** in `runtime/mod.ash`
3. **Documentation** for entry point usage

## TDD Steps

### Test 1: Capability Interface Compiles

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

### Test 2: Repeated Args Observation Works

```rust
let source = r#"
  use result::Result
  use runtime::RuntimeError
  use runtime::Args
  
  workflow main(args: cap Args) -> Result<(), RuntimeError> {
    let first = observe Args 0;
    let second = observe Args 1;
    done;
  }
"#;
let result = compile_and_typecheck(source);
assert!(result.is_ok());
```

## Implementation Notes

- **Location**: `std/src/runtime/args.ash`
- **Export**: Add to `runtime/mod.ash`
- **Style**: ordinary `capability` declaration plus explicit `observe` usage per SPEC-017

## Runtime Integration (Future)

The capability is **injected** by runtime, not constructed:

```rust
// Runtime side (Rust) - future implementation
let args_cap = ArgsCapability::from_env();
engine.inject_capability("Args", args_cap);
```

This is **not part of this task** - belongs to TASK-363.

## Dependencies

- TASK-359: ash-std structure
- S57-4: Import syntax (`use runtime::Args`)
- S57-5: Capability syntax (`cap Args` at usage sites, explicit effect-form invocation)

## Blocks

- TASK-362: System supervisor receives Args from runtime
- TASK-366: CLI passes args to runtime

## Spec Citations

| Aspect | Spec |
|--------|------|
| Capability syntax | SPEC-017 after S57-5 |
| Import syntax | SPEC-012 after S57-4 |
| CLI arg passing | SPEC-005 after S57-2 |

## Acceptance Criteria

- [ ] S57-4, S57-5 show ✅ Complete (VALIDATION GATE)
- [ ] Capability interface defined
- [ ] Can use as workflow parameter
- [ ] Explicit observation interface typechecks
- [ ] Tests pass

## Est. Hours: 3-4
