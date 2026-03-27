# TASK-335: Add Comprehensive Visibility Tests to Import Resolver

## Status: 🟡 High

## Problem

Import resolver has only a small set of visibility-focused tests, while type checker has broader SPEC-009 coverage. Need enough parity for confidence in import-side visibility enforcement.

## Current Test Coverage

| Visibility | Type Checker | Import Resolver |
|------------|--------------|-----------------|
| pub | ✅ 2 tests | ✅ 2 tests |
| pub(crate) | ✅ 4 tests | ❌ 0 tests |
| pub(super) | ✅ 6 tests | ❌ 0 tests |
| pub(self) | ✅ 3 tests | ❌ 0 tests |
| pub(in path) | ✅ 4 tests | ❌ 0 tests |
| Inherited | ✅ 4 tests | ✅ 2 tests |
| Edge cases | ✅ 10 tests | ❌ 0 tests |

**Total: Type checker 33 tests, Import resolver visibility coverage is materially incomplete**

## Implementation (TDD)

### Step 1: Create Test Infrastructure

Add to `crates/ash-parser/src/import_resolver.rs` test module:

```rust
#[cfg(test)]
mod visibility_tests {
    use super::*;
    
    /// Helper: Create a single-crate module graph for testing
    fn single_crate_graph() -> ModuleGraph {
        // Creates:
        // crate::root
        //   ├── crate::root::child_a
        //   │   └── crate::root::child_a::grandchild
        //   ├── crate::root::child_b
        //   └── crate::root::shared
    }
    
    /// Helper: Create deeply nested graph
    fn deep_graph(depth: usize) -> ModuleGraph {
        // Creates: crate::l0::l1::l2::...::lN
    }
    
    /// Helper: Create graph with sibling modules
    fn siblings_graph(count: usize) -> ModuleGraph {
        // Creates: crate::root with N siblings
    }
}
```

### Step 2: Write pub(crate) Tests (3+ tests)

```rust
#[test]
fn test_import_pub_crate_same_crate() { }

#[test]
fn test_import_pub_crate_root_module() { }

#[test]
fn test_import_pub_crate_nested_modules() { }
```

### Step 3: Write pub(super) Tests (6 tests)

```rust
#[test]
fn test_import_pub_super_direct_parent() { }

#[test]
fn test_import_pub_super_grandparent() { }

#[test]
fn test_import_pub_super_multi_level() { }

#[test]
fn test_import_pub_super_sibling_rejected() { }

#[test]
fn test_import_pub_super_child_rejected() { }

#[test]
fn test_import_pub_super_at_root() { }
```

### Step 4: Write pub(in path) Tests (4 tests)

```rust
#[test]
fn test_import_pub_in_path_exact() { }

#[test]
fn test_import_pub_in_path_descendant() { }

#[test]
fn test_import_pub_in_path_sibling_rejected() { }

#[test]
fn test_import_pub_in_path_parent_rejected() { }
```

### Step 5: Write Edge Case Tests

```rust
#[test]
fn test_import_visibility_deep_nesting() {
    // Test with 10+ levels of nesting
}

#[test]
fn test_import_visibility_circular_not_possible() {
    // Verify circular module refs don't affect visibility
}

#[test]
fn test_import_visibility_re_export() {
    // Test visibility through re-exports
}

#[test]
fn test_import_visibility_glob_import() {
    // Test glob imports respect visibility
}

#[test]
fn test_import_visibility_pub_self_same_module_only() {
    // Test Visibility::Self_ parity explicitly rather than treating it as N/A
}
```

### Step 6: Add Focused Deterministic Tests

```rust
#[test]
fn test_import_visibility_nested_import_respects_visibility() { }

#[test]
fn test_import_visibility_alias_import_respects_visibility() { }
```

### Step 7: Add Integration Tests

If unit tests inside `import_resolver.rs` cannot exercise a realistic parser/import flow cleanly, create `crates/ash-parser/tests/visibility_integration_test.rs`:

```rust
//! Integration tests for visibility enforcement

use ash_parser::parse_module::parse_module_decl;
use ash_parser::input::new_input;
use winnow::Parser;

#[test]
fn test_real_file_pub_crate() {
    let source = r#"
        pub(crate) workflow internal { done }
    "#;
    let module = parse_module_decl.parse(new_input(source)).unwrap();
    // Verify imports respect pub(crate)
}

#[test]
fn test_real_file_pub_super() {
    let source = r#"
        pub(super) workflow parent_only { done }
    "#;
    let module = parse_module_decl.parse(new_input(source)).unwrap();
    // Verify imports respect pub(super)
}
```

## Verification

```bash
# Count tests
cargo test --package ash-parser import_resolver -- --list | wc -l
# Target: materially expanded visibility coverage

# Run all visibility tests
cargo test --package ash-parser visibility --quiet

```

## Completion Checklist

- [ ] Test infrastructure created (graph builders)
- [ ] pub(crate) tests (3+ tests)
- [ ] pub(super) tests (6 tests)
- [ ] pub(in path) tests (4 tests)
- [ ] pub(self) tests added explicitly
- [ ] Edge case tests (deep nesting, re-exports, globs)
- [ ] Integration tests with real .ash files
- [ ] Import resolver visibility coverage materially expanded
- [ ] All tests pass
- [ ] CHANGELOG.md updated

**Estimated Hours:** 2-3
**Priority:** High (confidence/coverage)
**Dependencies:** TASK-332, TASK-333, TASK-334 (implementation to test)
**Related:** TASK-329 verification report
