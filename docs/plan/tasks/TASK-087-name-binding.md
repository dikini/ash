# TASK-087: Name Binding with Imports

## Status: 🟢 Complete

## Description

Integrate import resolution with name resolution to enable using imported names.

## Specification Reference

- SPEC-012: Import System - Section 7.3 Name Resolution Integration

## Requirements

### Functional Requirements

1. Look up names in import bindings
2. Handle shadowing (local > import > parent)
3. Support glob import priority
4. Detect name conflicts

### Property Requirements

```rust
// Import is usable
import("use crate::foo::bar", "crate::main");
resolve_name("bar", "crate::main").is_ok()

// Local shadows import
// use crate::foo::bar;
// let bar = 1;
// bar -> refers to local, not import

// Later import shadows earlier
// use crate::a::x;
// use crate::b::x;  -- this x is used
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_resolve_imported_name() {
    let resolver = build_resolver(r#"
        mod foo {
            pub capability bar: observe();
        }
        mod main {
            use crate::foo::bar;
        }
    "#);
    
    let result = resolver.resolve("bar", "crate::main");
    assert!(result.is_ok());
}

#[test]
fn test_local_shadows_import() {
    let resolver = build_resolver(r#"
        use crate::foo::bar;
        workflow w {
            let bar = 1;  -- shadows import
            action a {
                body: || -> bar;  -- refers to local
            }
        }
    "#);
    
    let binding = resolver.resolve("bar", "crate").unwrap();
    assert!(matches!(binding.kind, BindingKind::Local));
}

#[test]
fn test_glob_import_priority() {
    let resolver = build_resolver(r#"
        mod utils {
            pub capability a: observe();
        }
        mod main {
            use crate::utils::*;  -- brings in `a`
            -- explicit import would shadow glob
        }
    "#);
    
    let result = resolver.resolve("a", "crate::main");
    assert!(result.is_ok());
}
```

### Step 2: Integrate (Green)

Update name resolver to check imports:

```rust
impl NameResolver {
    pub fn resolve(&self, name: &str, module: ModuleId) -> Result<Binding, ResolveError> {
        // 1. Check local scope (let bindings, parameters)
        if let Some(binding) = self.local_scope.get(name) {
            return Ok(binding.clone());
        }
        
        // 2. Check module definitions
        if let Some(binding) = self.module_definitions.get(&(module, name.into())) {
            return Ok(binding.clone());
        }
        
        // 3. Check import bindings
        if let Some(binding) = self.import_bindings.get(&(module, name.into())) {
            return Ok(binding.clone());
        }
        
        // 4. Check parent module
        if let Some(parent) = self.graph.parent(module) {
            return self.resolve(name, parent);
        }
        
        Err(ResolveError::NotFound(name.into()))
    }
}
```

## Completion Checklist

- [ ] Name resolver checks imports
- [ ] Shadowing works correctly
- [ ] Glob imports have correct priority
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

5 hours

## Dependencies

- TASK-086 (Import resolution)
- TASK-022 (Name resolution)

## Blocked By

- TASK-086

## Blocks

None (completes import system)
