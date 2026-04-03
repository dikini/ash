# TASK-363a: Runtime Stdlib Loading Integration

## Status: ✅ Complete

## Description

Completed the narrow runtime stdlib-loading slice needed for entry verification and bootstrap using
the existing `Engine` API (SPEC-010), without claiming a full general module system.

`Engine` now owns a narrow runtime stdlib registry keyed by canonical module path. It exposes
`load_runtime_stdlib()` to register the canonical entry/runtime modules and
`has_registered_runtime_module()` for narrow registry checks. `parse_entry_source()` now validates
leading runtime `use` imports against that registry before stripping the entry prelude, and
`bootstrap_entry_source()` loads the runtime stdlib through the engine registry path instead of
depending only on raw preflight source loading.

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

This task uses **existing** `Engine`, not a new `Runtime` type. The implemented scope is limited to
the entry/runtime stdlib slice and does **not** introduce a general module graph or arbitrary module
loading pipeline.

## Requirements

1. **Engine owns a narrow runtime stdlib registry** keyed by canonical module path
2. **Leading entry runtime imports validate honestly** against the engine registry before the entry prelude is stripped
3. **Bootstrap uses the same registry-backed load path** via `load_runtime_stdlib()`
4. **No fictional APIs** - use SPEC-010 `Engine`
5. **Scope remains narrow** to entry/runtime stdlib support, not a full general module system

## Implementation Sketch

```rust
// In runtime/bootstrap
let mut engine = Engine::new();

// Register the narrow runtime stdlib slice needed for entry bootstrap.
engine.load_runtime_stdlib()?;

// Entry parsing now validates leading runtime imports against the registry
// before stripping the runtime prelude.
let parsed = engine.parse_entry_source(entry_src)?;
```

## TDD Steps

### Test 1: Engine Registers Runtime Stdlib Modules

```rust
let mut engine = Engine::new();
engine.load_runtime_stdlib()?;

assert!(engine.has_registered_runtime_module("runtime"));
```

### Test 2: Entry File Validates Leading Runtime Imports

```rust
let mut engine = Engine::new();
engine.load_runtime_stdlib()?;

let entry_src = r#"
    use runtime::RuntimeError
    use runtime::Args

    workflow main(args: cap Args) -> Result<(), RuntimeError> { done; }
"#;
let result = engine.parse_entry_source(entry_src);
assert!(result.is_ok());
```

## Implementation Notes

- **Use existing**: `Engine` from `ash-engine` crate
- **No new types**: No `Runtime::new()`
- **Registry ownership**: Runtime stdlib registration lives on `Engine`
- **Honest validation**: `parse_entry_source()` checks leading runtime imports against the registry before prelude stripping
- **Narrow scope**: This is entry/runtime stdlib support only, not a full general module loader

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

- [x] `Engine` owns a narrow runtime stdlib registry keyed by canonical module path
- [x] `Engine::load_runtime_stdlib()` registers the canonical runtime entry modules through the engine-owned path
- [x] `Engine::has_registered_runtime_module()` exposes narrow registry membership checks
- [x] `parse_entry_source()` validates leading runtime imports against the engine registry before stripping the prelude
- [x] `bootstrap_entry_source()` loads the runtime stdlib through the engine registry path
- [x] Uses existing `Engine`, with scope kept to the entry/runtime stdlib slice rather than a full general module graph

## Est. Hours: 2-3
