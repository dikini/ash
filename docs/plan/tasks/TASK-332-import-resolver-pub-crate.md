# TASK-332: Implement pub(crate) Enforcement in Import Resolver

## Status: 🔴 Critical

## Problem

The import resolver treats `pub(crate)` as universally visible. Under the current resolver this is an underspecified shortcut: the code should express the crate-root visibility rule explicitly instead of relying on unconditional allow.

**Current (Non-Compliant):**
```rust
Visibility::Crate => true,  // Always allows - WRONG
```

**Required (SPEC-009 Compliant):**
```rust
Visibility::Crate => {
    self.module_graph.in_same_rooted_crate(importing_module, target_module)
}
```

Note: the current resolver only accepts `crate::...` paths from a single graph root. Do **not** invent `CrateId` or multi-crate parsing unless the implementation proves it is necessary.

## Files to Modify

- `crates/ash-parser/src/import_resolver.rs` - Update `is_visible()` method
- `crates/ash-core/src/module_graph.rs` - Add minimal graph helpers (if needed)

## Implementation (TDD)

### Step 1: Write Failing Tests

Add to `crates/ash-parser/src/import_resolver.rs` test module:

```rust
#[test]
fn test_pub_crate_same_crate_allowed() {
    // Arrange: Two modules in same crate
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Import pub(crate) item within same crate
    // Use the existing add_module_exports/add_module_uses/resolve_all flow
    // and assert that a pub(crate) export is importable from another module
    // within the same rooted graph.
    
    // Assert: Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_pub_crate_cross_crate_rejected() {
    // Arrange: Two modules in different crates
    let graph = create_test_graph();
    let resolver = ImportResolver::new(&graph);
    
    // Act: Import pub(crate) item from different crate
    // If a true cross-crate case cannot be expressed in today's resolver,
    // replace this test with one that documents the current single-root
    // assumption and add a follow-up task rather than inventing fake APIs.
}
```

### Step 2: Add Minimal Graph Helper (if needed)

```rust
// In module_graph.rs
impl ModuleGraph {
    pub fn in_same_rooted_crate(&self, a: ModuleId, b: ModuleId) -> bool {
        // Express the current same-root graph semantics explicitly
    }
}
```

### Step 3: Implement Crate Check in is_visible()

```rust
fn is_visible(
    &self,
    visibility: &Visibility,
    importing_module: ModuleId,
    target_module: ModuleId,
) -> bool {
    match visibility {
        Visibility::Public => true,
        Visibility::Crate => {
            self.module_graph.in_same_rooted_crate(importing_module, target_module)
        }
        // ... other variants
    }
}
```

### Step 4: Run Tests

```bash
cargo test --package ash-parser import_resolver --quiet
# Expected: New tests pass, existing tests still pass
```

### Step 5: Clippy Check

```bash
cargo clippy --package ash-parser -- -D warnings
```

## Verification

```bash
# Run specific tests
cargo test --package ash-parser test_pub_crate --quiet

# Full parser test suite
cargo test --package ash-parser --quiet

# Full workspace
cargo test --workspace --quiet
```

## Completion Checklist

- [ ] Tests written (TDD - red phase)
- [ ] Minimal graph helper added to ModuleGraph (if needed)
- [ ] `is_visible()` updated for Crate visibility
- [ ] All tests pass (green phase)
- [ ] Clippy clean
- [ ] Documentation comments added
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2-3
**Priority:** Critical (security boundary)
**Dependencies:** None
**Related:** TASK-329 verification report
