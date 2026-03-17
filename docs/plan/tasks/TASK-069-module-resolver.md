# TASK-069: Module Resolution Algorithm

## Status: 🟢 Complete

## Description

Implement module resolution algorithm to discover and load modules from files.

## Specification Reference

- SPEC-009: Module System - Section 2.1 File-Based Modules

## Requirements

### Functional Requirements

1. Resolve `mod foo;` to `foo.ash` or `foo/mod.ash`
2. Build complete module graph from root
3. Detect circular dependencies
4. Handle parse errors in module files

### Property Requirements

```rust
// File resolution order
resolve_module("mod foo;", "src/") -> tries "src/foo.ash" then "src/foo/mod.ash"

// Graph construction
resolver.resolve_crate("main.ash") -> ModuleGraph with all modules

// Cycle detection
mod_a -> mod_b -> mod_a -> error
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_resolve_simple_module() {
    let fs = MockFs::new()
        .with_file("main.ash", "mod foo;")
        .with_file("foo.ash", "capability c: observe();");
    
    let resolver = ModuleResolver::with_fs(Box::new(fs));
    let graph = resolver.resolve_crate("main.ash").unwrap();
    
    assert_eq!(graph.node_count(), 2);
}

#[test]
fn test_resolve_mod_dir() {
    let fs = MockFs::new()
        .with_file("main.ash", "mod foo;")
        .with_file("foo/mod.ash", "capability c: observe();");
    
    let resolver = ModuleResolver::with_fs(Box::new(fs));
    let graph = resolver.resolve_crate("main.ash").unwrap();
    
    assert_eq!(graph.node_count(), 2);
}

#[test]
fn test_detect_cycle() {
    let fs = MockFs::new()
        .with_file("a.ash", "mod b;")
        .with_file("b.ash", "mod a;");
    
    let resolver = ModuleResolver::with_fs(Box::new(fs));
    let result = resolver.resolve_crate("a.ash");
    
    assert!(result.is_err());
}
```

### Step 2: Implement Resolver (Green)

```rust
pub struct ModuleResolver {
    fs: Box<dyn Fs>,
}

impl ModuleResolver {
    pub fn new() -> Self { /* real fs */ }
    pub fn with_fs(fs: Box<dyn Fs>) -> Self { Self { fs } }
    
    pub fn resolve_crate(&self, root: &str) -> Result<ModuleGraph, ResolveError> {
        let mut graph = ModuleGraph::new();
        let root_id = self.resolve_recursive(root, None, &mut graph, &mut HashSet::new())?;
        graph.set_root(root_id);
        Ok(graph)
    }
    
    fn resolve_recursive(&self, path: &str, parent: Option<ModuleId>, 
                         graph: &mut ModuleGraph, visiting: &mut HashSet<String>) 
                         -> Result<ModuleId, ResolveError> {
        // Implementation: parse file, find mod declarations, recurse
        todo!()
    }
}
```

## Completion Checklist

- [ ] ModuleResolver struct
- [ ] File resolution (foo.ash then foo/mod.ash)
- [ ] Recursive module loading
- [ ] Cycle detection
- [ ] Error handling
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

8 hours

## Dependencies

- TASK-064 (Module AST)
- TASK-067 (Parse module declarations)
- TASK-068 (Module graph)

## Blocked By

- TASK-068

## Blocks

- TASK-070 (Visibility checking)
