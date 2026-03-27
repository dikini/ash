# Phase 54: Import Resolver Visibility Enforcement

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Implement proper SPEC-009 visibility enforcement in the import resolver for restricted visibility variants (`pub(crate)`, `pub(super)`, `pub(in path)`), using the current single-root `crate::...` import model.

**Source:** TASK-329 verification report findings  
**Priority:** Critical (security/compliance)  
**Estimated Duration:** 10-14 hours

---

## Overview

The import resolver in `ash-parser` has placeholder implementations that treat all restricted visibility as `pub`. This phase implements proper enforcement against the current module graph and path model, without inventing unrelated multi-crate infrastructure unless a concrete blocker is discovered.

### Current State (Non-Compliant)

```rust
// crates/ash-parser/src/import_resolver.rs:474-488
fn is_visible(&self, visibility: &Visibility, ...) -> bool {
    match visibility {
        Visibility::Public => true,
        Visibility::Crate => true,              // ❌ Wrong: always allows
        Visibility::Inherited => false,
        Visibility::Super { .. } => true,       // ❌ Wrong: always allows
        Visibility::Self_ => false,
        Visibility::Restricted { .. } => true,  // ❌ Wrong: always allows
    }
}
```

### Target State (Compliant)

- `pub(crate)` - only visible within same crate
- `pub(super)` - only visible to parent module(s)
- `pub(in path)` - only visible within specified path

---

## Task Summary

| Task | Description | Est. Hours | Dependencies |
|------|-------------|------------|--------------|
| TASK-332 | Implement `pub(crate)` enforcement in import resolver | 2-3 | None |
| TASK-333 | Implement `pub(super)` enforcement in import resolver | 2-3 | TASK-332 |
| TASK-334 | Implement `pub(in path)` enforcement in import resolver | 3-4 | TASK-333 |
| TASK-335 | Add comprehensive visibility tests to import resolver | 2-3 | TASK-332-334 |
| TASK-336 | Phase 54 closeout and verification | 1 | All above |

---

## TASK-332: Implement `pub(crate)` Enforcement

**Objective:** Make `pub(crate)` items only importable within the same crate.

### Current Gap
```rust
Visibility::Crate => true,  // Always allows - WRONG
```

### Implementation Plan

#### Step 1: Confirm Current Crate Model

The `ImportResolver` only resolves `crate::...` paths from the graph root today. Before adding infrastructure, confirm whether `pub(crate)` enforcement can be expressed in terms of the current single-root graph.

If the resolver still only supports one crate root, do **not** introduce `CrateId` unless it is required by the implementation. Keep the task scoped to the current import model and document any future multi-crate work as a follow-up.

**Files to modify:**
- `crates/ash-parser/src/import_resolver.rs`

**Key structures:**
```rust
// ModuleGraph already tracks module relationships (root, parent, child edges)
// Prefer deriving visibility from the existing graph shape before adding new identity types
```

#### Step 2: Update `is_visible()` for Crate Visibility

```rust
Visibility::Crate => {
    self.module_graph.in_same_rooted_crate(importing_module, target_module)
}
```

#### Step 3: Update ModuleGraph Helpers (if needed)

If the existing graph helpers are insufficient, add only the minimal helper methods needed to answer same-root, parent, ancestor, and descendant questions.

#### Step 4: Write Tests (TDD)

**Test cases:**
- `test_pub_crate_same_crate_allowed` - import within same crate passes
- `test_pub_crate_cross_crate_rejected` - import from different crate fails
- `test_pub_crate_root_module` - pub(crate) at root behaves correctly

**Test file:** `crates/ash-parser/src/import_resolver.rs` (in `#[cfg(test)]` module using the existing resolver-style helpers and `resolve_all()`)

#### Step 5: Verify

```bash
cargo test --package ash-parser import_resolver --quiet
cargo clippy --package ash-parser -- -D warnings
```

#### Step 6: Commit

```bash
git add crates/ash-parser/
git commit -m "fix(visibility): TASK-332 - Implement pub(crate) enforcement in import resolver

- Add same-root visibility helper to ModuleGraph
- Update is_visible() to check crate membership for Crate visibility
- Add tests for same-root import visibility scenarios"
```

---

## TASK-333: Implement `pub(super)` Enforcement

**Objective:** Make `pub(super)` items only visible to parent modules.

### Current Gap
```rust
Visibility::Super { .. } => true,  // Always allows - WRONG
```

### Implementation Plan

#### Step 1: Understand Parent Module Hierarchy

`pub(super)` means visible to:
- The immediate parent module (levels=1)
- Ancestor modules up to `levels` (represented internally via `Visibility::Super { levels }`)

**Key insight:** Importing module must be an ancestor of the target module.

#### Step 2: Update `is_visible()` for Super Visibility

```rust
Visibility::Super { levels } => {
    let target_ancestors = self.module_graph.ancestors(target_module);
    let importing_is_ancestor = target_ancestors
        .take(*levels as usize)
        .any(|(module, _)| module == importing_module);
    importing_is_ancestor
}
```

#### Step 3: Ensure ModuleGraph Has `ancestors()` Method

If not present, implement minimal helper methods on the existing module graph in `ash-core` rather than inventing a new path type in `ash-parser`.

#### Step 4: Write Tests (TDD)

**Test cases:**
- `test_pub_super_parent_allowed` - direct parent can import
- `test_pub_super_grandparent_allowed` - ancestor within levels can import
- `test_pub_super_sibling_rejected` - sibling module cannot import
- `test_pub_super_child_rejected` - child module cannot import
- `test_pub_super_multi_level` - pub(super, super) works correctly

#### Step 5: Verify

```bash
cargo test --package ash-parser import_resolver --quiet
cargo clippy --package ash-parser -- -D warnings
```

#### Step 6: Commit

```bash
git add crates/ash-parser/
git commit -m "fix(visibility): TASK-333 - Implement pub(super) enforcement in import resolver

- Add ancestor chain checking for Super visibility
- Support multi-level pub(super) with levels parameter
- Add tests for parent, grandparent, sibling, and child scenarios"
```

---

## TASK-334: Implement `pub(in path)` Enforcement

**Objective:** Make `pub(in path)` items only visible within the specified module path.

### Current Gap
```rust
Visibility::Restricted { .. } => true,  // Always allows - WRONG
```

### Implementation Plan

#### Step 1: Understand Restricted Path Visibility

`pub(in crate::foo::bar)` means visible to:
- The specified module path `crate::foo::bar`
- Any descendant of that path

**Key insight:** Importing module must be the specified path OR a descendant of it.

#### Step 2: Update `is_visible()` for Restricted Visibility

```rust
Visibility::Restricted { path } => {
    // Convert the restricted path string into a module lookup using the existing resolver path model
    let restricted_module = self.resolve_restricted_path(path);
    match restricted_module {
        Some(restricted_module) => {
            self.module_graph.is_descendant_or_same(importing_module, restricted_module)
        }
        None => false,
    }
}
```

#### Step 3: Add Helper Methods to ModuleGraph

```rust
impl ModuleGraph {
    /// Check if `module` is `ancestor` or a descendant of `ancestor`
    fn is_descendant_or_same(&self, module: ModuleId, ancestor: ModuleId) -> bool;
}
```

If path resolution is needed, prefer reusing the existing `SimplePath`/root-walk logic in `ImportResolver` rather than moving path parsing into `ModuleGraph`.

#### Step 4: Handle Path Resolution Errors

Path may not resolve if:
- Module doesn't exist
- Path is malformed

Should return `false` (not visible) in these cases.

#### Step 5: Write Tests (TDD)

**Test cases:**
- `test_pub_in_path_exact_match_allowed` - exact path can import
- `test_pub_in_path_descendant_allowed` - descendant module can import
- `test_pub_in_path_sibling_rejected` - sibling of restricted path cannot import
- `test_pub_in_path_parent_rejected` - parent of restricted path cannot import
- `test_pub_in_path_crate_root` - pub(in crate) works like pub(crate)

#### Step 6: Verify

```bash
cargo test --package ash-parser import_resolver --quiet
cargo clippy --package ash-parser -- -D warnings
```

#### Step 7: Commit

```bash
git add crates/ash-parser/
git commit -m "fix(visibility): TASK-334 - Implement pub(in path) enforcement in import resolver

- Add path resolution for restricted visibility
- Implement descendant checking for restricted paths
- Handle edge cases: exact match, descendants, non-existent paths
- Add comprehensive tests for restricted visibility scenarios"
```

---

## TASK-335: Add Comprehensive Visibility Tests

**Objective:** Ensure import resolver has test parity with type checker visibility tests.

### Current State
- Type checker: 33 visibility tests
- Import resolver: 11 tests (only Public/Inherited)

### Implementation Plan

#### Step 1: Create Test Infrastructure

Add helper functions for building test module graphs:
```rust
#[cfg(test)]
mod tests {
    fn create_test_module_graph() -> ModuleGraph {
        // Create a complex hierarchy:
        // crate::root
        //   ├── crate::root::child_a
        //   │   └── crate::root::child_a::grandchild
        //   ├── crate::root::child_b
        //   └── crate::root::shared
    }
}
```

#### Step 2: Add Focused Deterministic Tests

Prefer targeted unit and integration tests that exercise real resolver behavior:
```rust
#[test]
fn test_import_visibility_nested_import_respects_visibility() { }

#[test]
fn test_import_visibility_alias_import_respects_visibility() { }
```

#### Step 3: Add Edge Case Tests

**Edge cases to cover:**
- Root module visibility (pub(super) at root)
- Deep nesting (5+ levels)
- Circular module dependencies (shouldn't affect visibility)

#### Step 4: Add Integration Tests

Create `crates/ash-parser/tests/visibility_integration_test.rs`:
- Parse actual `.ash` files with visibility annotations
- Verify import resolution respects visibility

#### Step 5: Verify Test Count

Target: materially expanded import-resolver visibility coverage

```bash
cargo test --package ash-parser import_resolver --quiet
cargo test --package ash-parser visibility --quiet
```

#### Step 6: Commit

```bash
git add crates/ash-parser/
git commit -m "test(visibility): TASK-335 - Add comprehensive import resolver visibility tests

- Add test infrastructure for module graph construction
- Add focused deterministic tests for visibility behavior
- Add edge case tests (root, deep nesting, resolver integration)
- Add integration tests with real .ash files
- Achieve parity with type checker visibility test coverage"
```

---

## TASK-336: Phase 54 Closeout

**Objective:** Final verification and documentation.

### Verification Steps

#### 1. Full Test Suite
```bash
cargo test --workspace --quiet
```

#### 2. SPEC-009 Compliance Verification
```bash
cargo test --package ash-parser import_resolver --quiet
# Verify expanded visibility coverage passes
cargo test --package ash-typeck visibility --quiet
# Verify type checker still passes (no regressions)
```

#### 3. Clippy Check
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

#### 4. Update TASK-329 Report
Mark gaps as resolved in `docs/plan/tasks/TASK-329-VERIFICATION-REPORT.md`

#### 5. Update PLAN-INDEX.md
Add Phase 54 section.

#### 6. Commit
```bash
git add docs/plan/
git commit -m "docs(plan): TASK-336 - Phase 54 closeout

- All import resolver visibility enforcement implemented
- SPEC-009 compliance achieved
- Tests passing, clippy clean
- Documentation updated"
```

---

## Success Criteria

Phase 54 is complete when:

- [ ] `pub(crate)` only allows imports within same crate
- [ ] `pub(super)` only allows imports from parent modules
- [ ] `pub(in path)` only allows imports from specified path
- [ ] Import resolver visibility coverage materially expanded
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean
- [ ] TASK-329 report updated with gaps resolved
- [ ] PLAN-INDEX.md documents Phase 54 as complete

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| ModuleGraph changes break existing code | Add comprehensive tests before modifying |
| Cross-crate testing is complex | Use in-memory module graphs, not file system |
| Path resolution is ambiguous | Follow type checker path resolution logic |
| Performance regression | Benchmark import resolution before/after |

---

## Dependencies

**External:** None

**Internal:**
- `ash-parser` module graph construction
- `ash-typeck` visibility types (should match)

**Skills Required:**
- /rust-skills (ownership, error handling, testing)
- /subagent-driven-development

---

## Notes

- Follow TDD: write tests before implementation
- Use property-based testing with `proptest`
- Keep changes minimal and focused per task
- Ensure parity between type checker and import resolver behavior
