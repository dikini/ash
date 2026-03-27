# TASK-333: Implement pub(super) Enforcement in Import Resolver

## Status: 🔴 Critical

## Problem

The import resolver treats `pub(super)` as universally visible, ignoring parent module hierarchy.

**Current (Non-Compliant):**
```rust
Visibility::Super { .. } => true,  // Always allows - WRONG
```

**Required (SPEC-009 Compliant):**
```rust
Visibility::Super { levels } => {
    let target_ancestors = self.module_graph.ancestors(target_module);
    let importing_is_ancestor = target_ancestors
        .take(*levels as usize)
        .any(|module| module == importing_module);
    importing_is_ancestor
}
```

## Files to Modify

- `crates/ash-parser/src/import_resolver.rs` - Update `is_visible()` method
- `crates/ash-core/src/module_graph.rs` - Add ancestor/parent helpers (if needed)

## Implementation (TDD)

### Step 1: Write Failing Tests

```rust
#[test]
fn test_pub_super_parent_allowed() {
    // Arrange: parent -> child module hierarchy
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Parent importing from child with pub(super)
    // Use the existing resolver API and helpers in import_resolver.rs tests:
    // add a pub(super) export in a child module, import from its parent,
    // and assert resolve_all() succeeds.
    
    // Assert: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_pub_super_grandparent_allowed() {
    // Arrange: grandparent -> parent -> child
    let graph = create_deep_module_graph(3);
    let resolver = ImportResolver::new(graph);
    
    // Act: Grandparent importing from child with Visibility::Super { levels: 2 }
    // Construct the enum directly in the test if needed. Do not assume a new
    // source syntax like `pub(super, super)` exists.
    
    // Assert: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_pub_super_sibling_rejected() {
    // Arrange: Two sibling modules
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Sibling trying to import pub(super) item
    // Use the existing add_module_exports/add_module_uses/resolve_all flow.
    // Extend the test graph with a second child under the same parent if needed,
    // export an item from one sibling with Visibility::Super { levels: 1 },
    // import it from the other sibling, and assert resolve_all() rejects it.
    
    // Assert: Should fail
    assert!(result.is_err());
}

#[test]
fn test_pub_super_child_rejected() {
    // Arrange: Parent with child
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Child trying to import parent's pub(super) item
    // Use the existing resolver test harness:
    // place the export on the parent module, import from its child,
    // and assert resolve_all() rejects it.
    
    // Assert: Should fail (child is not parent's ancestor)
    assert!(result.is_err());
}
```

### Step 2: Add Ancestor Helper to ModuleGraph (if needed)

```rust
impl ModuleGraph {
    /// Return the direct parent of a module, if any
    pub fn parent_of(&self, module: ModuleId) -> Option<ModuleId> {
        self.nodes.iter().find_map(|(&candidate, node)| {
            node.children.contains(&module).then_some(candidate)
        })
    }

    /// Returns iterator from module toward root
    pub fn ancestors(&self, module: ModuleId) -> impl Iterator<Item = ModuleId> {
        std::iter::successors(Some(module), move |current| self.parent_of(*current))
    }
}
```

### Step 3: Implement Super Check in is_visible()

```rust
fn is_visible(
    &self,
    visibility: &Visibility,
    importing_module: ModuleId,
    target_module: ModuleId,
) -> bool {
    match visibility {
        // ... other variants
        Visibility::Super { levels } => {
            // Check if importing_module is an ancestor of target_module
            // within 'levels' steps up from target
            let target_ancestors = self.module_graph.ancestors(target_module);
            let importing_is_ancestor = target_ancestors
                .take(*levels as usize)
                .any(|module| module == importing_module);
            importing_is_ancestor
        }
        // ... other variants
    }
}
```

### Step 4: Run Tests

```bash
cargo test --package ash-parser test_pub_super --quiet
```

## Edge Cases

1. **Root module with pub(super):** Verify and document actual SPEC-009 behavior instead of guessing
2. **levels > ancestor depth:** Should allow up to root, not beyond
3. **levels = 0:** Edge case - should behave like pub(self)

## Verification

```bash
cargo test --package ash-parser import_resolver --quiet
cargo clippy --package ash-parser -- -D warnings
```

## Completion Checklist

- [ ] Tests written for parent, grandparent, sibling, child scenarios
- [ ] Ancestor/parent helper added to ModuleGraph (if needed)
- [ ] `is_visible()` updated for Super visibility
- [ ] Edge cases handled (root, deep levels)
- [ ] All tests pass
- [ ] Clippy clean
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2-3
**Priority:** Critical (encapsulation boundary)
**Dependencies:** TASK-332 (shared graph helper groundwork, if any)
**Related:** TASK-329 verification report
