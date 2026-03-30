# TASK-361: Define Args Capability Interface

## Status: ⛔ Blocked

## Description

Define the `Args` capability interface in `ash-std/src/runtime/args.ash` for accessing command-line arguments.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-4 (import syntax)**: ✅ Complete - confirms `use runtime::Args`
2. **Verify S57-5 (capability syntax)**: ✅ Complete - confirms `capability Args { ... }`
3. **If SPEC uses different syntax**: Update this task description to match

## Design (per updated SPEC)

```ash
-- ash-std/src/runtime/args.ash
pub capability Args {
  fn get(index: Int) -> Option<String>
  fn all() -> [String]
  fn len() -> Int
  fn is_empty() -> Bool
}
```

**Note on argv[0]**: `get(0)` returns program name (first arg), `get(1)` returns first user arg. This matches common convention but verify in S57-2 if different.

## Requirements

1. **Capability interface** in `runtime/args.ash`
2. **Module export** in `runtime/mod.ash`
3. **Documentation** for entry point usage

## TDD Steps

### Test 1: Capability Interface Compiles
```rust
let source = r#"
  use runtime::Args
  
  workflow main(args: capability Args) -> Result<(), RuntimeError> {
    let first = args.get(0);
    Ok(())
  }
"#;
let result = compile_with_stdlib(source);
assert!(result.is_ok());
```

### Test 2: Args Methods Work
```rust
let source = r#"
  use runtime::Args
  
  fn check_args(args: capability Args) -> Bool {
    if args.is_empty() {
      false
    } else {
      args.len() > 0
    }
  }
"#;
let result = compile_and_typecheck(source);
assert!(result.is_ok());
```

## Implementation Notes

- **Location**: `ash-std/src/runtime/args.ash`
- **Export**: Add to `runtime/mod.ash`
- **Style**: `capability` keyword per SPEC-017 after S57-5

## Runtime Integration (Future)

The capability is **injected** by runtime, not constructed:

```rust
// Runtime side (Rust) - future implementation
let args_cap = ArgsCapability::from_env();
engine.inject_capability("args", args_cap);
```

This is **not part of this task** - belongs to TASK-363.

## Dependencies

- TASK-359: ash-std structure
- S57-4: Import syntax (`use runtime::Args`)
- S57-5: Capability syntax (`capability Args`)

## Blocks

- TASK-362: System supervisor receives Args from runtime
- TASK-366: CLI passes args to runtime

## Spec Citations (Update After 57A)

| Aspect | Spec |
|--------|------|
| Capability syntax | SPEC-017 after S57-5 |
| Import syntax | SPEC-012 after S57-4 |
| CLI arg passing | SPEC-005 after S57-2 |

## Acceptance Criteria

- [ ] S57-4, S57-5 show ✅ Complete (VALIDATION GATE)
- [ ] Capability interface defined
- [ ] Can use as workflow parameter
- [ ] Methods typecheck
- [ ] Tests pass

## Est. Hours: 3-4
