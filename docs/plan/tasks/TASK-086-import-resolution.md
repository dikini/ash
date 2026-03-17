# TASK-086: Import Resolution Algorithm

## Status: 🟢 Complete

## Description

Implement the import resolution algorithm to resolve `use` statements to actual items.

## Specification Reference

- SPEC-012: Import System - Section 3 Import Resolution

## Requirements

### Functional Requirements

1. Resolve `use path::to::item` to actual definition
2. Resolve glob imports (`use path::*`)
3. Resolve nested imports (`use path::{a, b}`)
4. Handle aliases (`use path::item as alias`)
5. Verify visibility constraints
6. Detect import cycles

### Property Requirements

```rust
// Resolution succeeds for valid paths
resolve_import("crate::foo::bar", &module_graph).is_ok()

// Glob resolves all public items
resolve_glob("crate::foo").contains_all_public_items()

// Private items not accessible
resolve_import("crate::internal::secret").is_err()

// Cycles detected
// a.ash: use crate::b::x; b.ash: use crate::a::y;
// -> cycle error
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_resolve_simple_import() {
    let graph = build_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    let result = resolver.resolve("crate::foo::bar", "crate::main");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "bar");
}

#[test]
fn test_resolve_glob_import() {
    let graph = build_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    let items = resolver.resolve_glob("crate::utils", "crate::main");
    assert!(items.len() > 0);
}

#[test]
fn test_resolve_private_item_fails() {
    let graph = build_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // internal is not pub
    let result = resolver.resolve("crate::foo::internal", "crate::main");
    assert!(result.is_err());
}

#[test]
fn test_detect_import_cycle() {
    let graph = build_cyclic_graph();
    let resolver = ImportResolver::new(&graph);
    
    let result = resolver.resolve_all();
    assert!(matches!(result, Err(ImportError::Cycle(_))));
}
```

### Step 2: Implement Resolver (Green)

```rust
pub struct ImportResolver<'a> {
    graph: &'a ModuleGraph,
    bindings: HashMap<ModuleId, HashMap<Box<str>, Binding>>,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub target_module: ModuleId,
    pub item_name: Box<str>,
    pub visibility: Visibility,
    pub kind: BindingKind,
}

#[derive(Debug, Clone)]
pub enum BindingKind {
    Capability,
    Workflow,
    Policy,
    Role,
}

impl<'a> ImportResolver<'a> {
    pub fn new(graph: &'a ModuleGraph) -> Self {
        Self {
            graph,
            bindings: HashMap::new(),
        }
    }
    
    pub fn resolve_all(&mut self) -> Result<(), ImportError> {
        // Resolve imports for all modules
        for (module_id, module) in self.graph.all_modules() {
            for use_stmt in &module.imports {
                self.resolve_use(*module_id, use_stmt)?;
            }
        }
        Ok(())
    }
    
    fn resolve_use(&mut self, from: ModuleId, use_stmt: &Use) -> Result<(), ImportError> {
        match &use_stmt.path {
            UsePath::Simple(path) => {
                let binding = self.resolve_path(path, from)?;
                let name = use_stmt.alias.as_ref()
                    .unwrap_or(&binding.item_name)
                    .clone();
                self.add_binding(from, name, binding);
            }
            UsePath::Glob(path) => {
                let bindings = self.resolve_glob(path, from)?;
                for binding in bindings {
                    self.add_binding(from, binding.item_name.clone(), binding);
                }
            }
            UsePath::Nested(path, items) => {
                for item in items {
                    let full_path = join_path(path, &item.name);
                    let binding = self.resolve_path(&full_path, from)?;
                    let name = item.alias.as_ref()
                        .unwrap_or(&item.name)
                        .clone();
                    self.add_binding(from, name, binding);
                }
            }
        }
        Ok(())
    }
    
    fn resolve_path(&self, path: &SimplePath, from: ModuleId) -> Result<Binding, ImportError> {
        // Resolve path to actual item
        // Verify visibility
        todo!()
    }
    
    fn resolve_glob(&self, path: &SimplePath, from: ModuleId) -> Result<Vec<Binding>, ImportError> {
        // Return all public items in target module
        todo!()
    }
    
    fn add_binding(&mut self, module: ModuleId, name: Box<str>, binding: Binding) {
        self.bindings
            .entry(module)
            .or_default()
            .insert(name, binding);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("Cannot find `{0}`")]
    NotFound(String),
    #[error("`{0}` is private")]
    Private(String),
    #[error("Import cycle detected: {0}")]
    Cycle(String),
}
```

## Completion Checklist

- [ ] Simple imports resolved
- [ ] Glob imports resolved
- [ ] Nested imports resolved
- [ ] Visibility checked
- [ ] Cycles detected
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

6 hours

## Dependencies

- TASK-084 (Use AST)
- TASK-085 (Parse use)
- TASK-069 (Module resolver)

## Blocked By

- TASK-085
- TASK-069

## Blocks

- TASK-087 (Name binding)
