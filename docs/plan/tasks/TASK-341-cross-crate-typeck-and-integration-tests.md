# TASK-341: Align Type Checker and Add Multi-Crate Regression Coverage

## Status: 🟡 High

## Problem

`ash-typeck` still reasons about visibility using loose string heuristics for “external” access, and the current regression suite does not prove that parser/resolver/type-checker behavior stays aligned once real dependency crates are loaded.

**Current (Heuristic):**
```rust
Visibility::Crate => {
    let item_root = item_module.segments.first();
    let from_root = from_module.segments.first();
    item_root == from_root
}
```

**Required (Explicit Cross-Crate Semantics):**
```rust
enum CrateRef {
    Current,
    External(String),
}
```

The implementation may use a different exact type shape, but the semantics must model current crate vs external crate explicitly rather than relying on string-prefix accidents.

## Files to Modify

- `crates/ash-typeck/src/visibility.rs`
- `crates/ash-typeck/tests/visibility_test.rs`
- `crates/ash-parser/tests/multi_crate_visibility.rs` (new)

## Implementation (TDD)

### Step 1: Write failing type-checker tests

Add coverage for:

```rust
#[test]
fn test_pub_crate_not_visible_from_external_alias() { }

#[test]
fn test_pub_visible_from_external_alias() { }

#[test]
fn test_pub_super_not_visible_from_external_alias() { }

#[test]
fn test_pub_in_path_not_visible_from_external_alias() { }
```

### Step 2: Make external crate paths explicit in the checker

Refactor path parsing or representation so tests can express:

```rust
"crate::internal::helper"
"external::util::helpers"
```

without relying on undocumented heuristics.

### Step 3: Add multi-crate regression tests

Use realistic multi-crate source inputs or fixtures to prove:
- public external imports survive parser + resolver flow
- restricted external imports are rejected consistently

## Verification

```bash
cargo test --package ash-typeck visibility --quiet
cargo test --package ash-parser multi_crate_visibility --quiet
```

## Completion Checklist

- [ ] type checker has explicit external crate path semantics
- [ ] cross-crate visibility cases covered in `ash-typeck`
- [ ] parser/resolver integration tests cover multi-crate acceptance and rejection
- [ ] import resolver and type checker expectations stay aligned
- [ ] all new tests pass
- [ ] CHANGELOG.md update planned for implementation phase

**Estimated Hours:** 2-3
**Priority:** High (semantic parity)
**Dependencies:** TASK-340
**Related:** TASK-329 verification follow-up
