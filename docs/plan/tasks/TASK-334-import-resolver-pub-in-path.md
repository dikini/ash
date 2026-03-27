# TASK-334: Implement pub(in path) Enforcement in Import Resolver

## Status: 🔴 Critical

## Problem

The import resolver treats `pub(in path)` as universally visible, ignoring the specified restriction path.

**Current (Non-Compliant):**
```rust
Visibility::Restricted { .. } => true,  // Always allows - WRONG
```

**Required (SPEC-009 Compliant):**
```rust
Visibility::Restricted { path } => {
    match self.resolve_restricted_path(path) {
        Some(restricted_module) => {
            self.module_graph.is_descendant_or_same(importing_module, restricted_module)
        }
        None => false,
    }
}
```

## Files to Modify

- `crates/ash-parser/src/import_resolver.rs` - Update `is_visible()` method
- `crates/ash-core/src/module_graph.rs` - Add descendant/path helpers (if needed)

## Implementation (TDD)

### Step 1: Write Failing Tests

```rust
#[test]
fn test_pub_in_path_exact_match_allowed() {
    // Arrange: Module restricting visibility to itself
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Same module importing pub(in crate::module) item
    // Use the existing add_module_exports/add_module_uses/resolve_all flow and
    // construct Visibility::Restricted { path: "crate::module".into() } directly.
    
    // Assert: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_pub_in_path_descendant_allowed() {
    // Arrange: Parent restricting to path, child importing
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Child importing pub(in crate::parent) item
    // Use the existing add_module_exports/add_module_uses/resolve_all flow and
    // construct the restriction path with SimplePath-compatible segments
    // (for example via the local simple_path helper or an equivalent value).
    
    // Assert: Should succeed (child is descendant of parent)
    assert!(result.is_ok());
}

#[test]
fn test_pub_in_path_sibling_rejected() {
    // Arrange: Two siblings
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: sibling_b trying to import from sibling_a's restricted item
    // Extend the graph with explicit sibling modules if needed, export an item
    // with Visibility::Restricted { path: "crate::root::sibling_a".into() },
    // import from sibling_b, and assert resolve_all() rejects it.
    
    // Assert: Should fail
    assert!(result.is_err());
}

#[test]
fn test_pub_in_path_parent_rejected() {
    // Arrange: Parent and child
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Parent trying to import child's restricted item
    // Use the existing resolver test harness:
    // export from the child with a restriction path naming the child module,
    // import from the parent, and assert resolve_all() rejects it.
    
    // Assert: Should fail (parent is not descendant of child)
    assert!(result.is_err());
}

#[test]
fn test_pub_in_path_crate_root() {
    // Arrange: Single crate
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Importing pub(in crate) item
    // Restrict an export with path "crate" and verify imports from another
    // module in the same rooted graph still succeed.
    
    // Assert: Should succeed (crate root includes all)
    assert!(result.is_ok());
}

#[test]
fn test_pub_in_path_nonexistent_rejected() {
    // Arrange: Module graph
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Importing with non-existent path
    // Use Visibility::Restricted { path: "crate::nonexistent".into() } and
    // assert resolve_all() rejects the import because the restricted module
    // path cannot be resolved.
    
    // Assert: Should fail (non-existent path = not visible)
    assert!(result.is_err());
}
```

### Step 2: Add Descendant Helper to ModuleGraph

```rust
impl ModuleGraph {
    /// Check if `module` is `ancestor` or a descendant of `ancestor`
    pub fn is_descendant_or_same(&self, module: ModuleId, ancestor: ModuleId) -> bool {
        // Walk parent links / derived ancestry in the existing graph
    }
}
```

### Step 3: Implement Restricted Check in is_visible()

```rust
fn is_visible(
    &self,
    visibility: &Visibility,
    importing_module: ModuleId,
    target_module: ModuleId,
) -> bool {
    match visibility {
        // ... other variants
        Visibility::Restricted { path } => {
            // Resolve the restricted path to a module
            match self.resolve_restricted_path(path) {
                Some(restricted_module) => {
                    // Importing module must be the restricted module or its descendant
                    self.module_graph.is_descendant_or_same(importing_module, restricted_module)
                }
                None => {
                    // Non-existent path = not visible
                    false
                }
            }
        }
        // ... other variants
    }
}
```

### Step 4: Run Tests

```bash
cargo test --package ash-parser test_pub_in_path --quiet
```

## Edge Cases

1. **Path = `crate`:** Should behave like `pub(crate)` under the current resolver model
2. **Path to non-module item:** Should be rejected
3. **Path with super/self:** May need special handling
4. **Circular paths:** Shouldn't occur but handle gracefully

## Verification

```bash
cargo test --package ash-parser import_resolver --quiet
cargo clippy --package ash-parser -- -D warnings
```

## Completion Checklist

- [ ] Tests written for exact match, descendant, sibling, parent scenarios
- [ ] Restricted-path resolution implemented using the resolver's current path model
- [ ] `is_descendant_or_same()` method added to ModuleGraph
- [ ] `is_visible()` updated for Restricted visibility
- [ ] Edge cases handled (non-existent path, crate root)
- [ ] All tests pass
- [ ] Clippy clean
- [ ] CHANGELOG.md updated

**Estimated Hours:** 3-4
**Priority:** Critical (fine-grained visibility control)
**Dependencies:** TASK-332, TASK-333 (shared graph helper groundwork)
**Related:** TASK-329 verification report
