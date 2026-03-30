# TASK-363a: Runtime Stdlib Loading Integration

## Status: ⛔ Blocked

## Description

Integrate stdlib loading into runtime using existing `Engine` API (SPEC-010), not fictional `Runtime::new()`.

**VALIDATION GATE - REQUIRED BEFORE IMPLEMENTATION:**

1. **Verify S57-4 (stdlib loading)**: ✅ Complete - confirms how Engine loads ash-std
2. **Verify existing Engine API**: Review SPEC-010 for `Engine::load_module` or equivalent
3. **If SPEC differs**: Update this task description

## Background

Current architecture (SPEC-010) has `Engine` for embedding:
```rust
// SPEC-010-EMBEDDING.md:14-36
pub struct Engine { ... }
impl Engine {
    pub fn new() -> Self;
    pub fn load_module(&mut self, source: &str) -> Result<Module, Error>;
    // ...
}
```

This task uses **existing** `Engine`, not a new `Runtime` type.

## Requirements

1. **Engine loads ash-std** at initialization
2. **Stdlib modules available** for `use` resolution
3. **No fictional APIs** - use SPEC-010 `Engine`

## Implementation Sketch

```rust
// In runtime/bootstrap (future TASK-363c)
let mut engine = Engine::new();

// Load ash-std modules
let stdlib_modules = load_ash_std_modules()?;  // This task
for module in stdlib_modules {
    engine.load_module(&module)?;
}

// Now entry file can `use runtime::Args`
```

## TDD Steps

### Test 1: Engine Loads ash-std
```rust
let mut engine = Engine::new();
let stdlib = load_ash_std().expect("ash-std loads");

for module_src in stdlib.modules() {
    engine.load_module(module_src)?;
}

// Verify result module available
assert!(engine.has_module("result"));
```

### Test 2: Entry File Can Import Stdlib
```rust
let mut engine = Engine::new();
load_ash_std_into(&mut engine)?;

let entry_src = r#"use runtime::Args; workflow main(args: capability Args) { ... }"#;
let result = engine.load_module(entry_src);
assert!(result.is_ok());  // Args resolves from ash-std
```

## Implementation Notes

- **Use existing**: `Engine` from `ash-engine` crate
- **No new types**: No `Runtime::new()`
- **Source loading**: Load ash-std source files, compile via Engine

## Dependencies

- TASK-359: ash-std structure exists
- SPEC-010: Engine API
- S57-4: Module loading semantics

## Blocks

- TASK-363c: Bootstrap uses stdlib loading

## Spec Citations

| Aspect | Spec |
|--------|------|
| Engine API | SPEC-010 |
| Module loading | SPEC-009 after S57-4 |

## Acceptance Criteria

- [ ] S57-4, SPEC-010 verified (VALIDATION GATE)
- [ ] Engine loads ash-std modules
- [ ] Stdlib types resolve in entry files
- [ ] Uses existing Engine, no fictional APIs
- [ ] Tests pass

## Est. Hours: 2-3
