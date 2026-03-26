# TASK-273: Fix arb_pattern Binding Name Uniqueness

**Objective:** Fix the `arb_pattern()` generator in proptest_helpers to ensure generated binding names are unique within a pattern.

**Spec Reference:** Test Infrastructure

**File Locations:**
- Modify: `crates/ash-core/src/proptest_helpers.rs`
- Test: `crates/ash-core/src/proptest_helpers.rs` (existing tests)

---

## Background

The `arb_pattern()` generator can create patterns with duplicate binding names when a rest pattern (`G_`) matches a variable name (`G_`) in the same record.

**Failing Test:** `test_arb_pattern_bindings_unique`
**Error:** Pattern generator creates patterns where the same name appears multiple times

Example problematic pattern:
```rust
Record {
    ("field1", Variable("G_")),
    ("field2", Rest("G_"))  // Same name as field1!
}
```

---

## Implementation

### Step 1: Understand Current Implementation

Read `crates/ash-core/src/proptest_helpers.rs`:
- Find `arb_pattern()` function
- Understand how names are generated
- Identify where uniqueness should be enforced

### Step 2: Implement Name Tracking

Modify `arb_pattern()` to:
1. Track used names during generation
2. Generate unique names for each binding
3. Ensure rest patterns don't conflict with field patterns

```rust
fn arb_pattern_with_context(used_names: &mut HashSet<String>) -> impl Strategy<Value = Pattern> {
    // Generate name, check against used_names, add to set
}
```

### Step 3: Run Tests

```bash
cargo test --package ash-core test_arb_pattern_bindings_unique -v
```

### Step 4: Commit

```bash
git add crates/ash-core/src/proptest_helpers.rs
git commit -m "fix: ensure arb_pattern generates unique binding names (TASK-273)

- Track used names during pattern generation
- Prevent duplicate bindings in generated patterns
- Fix test_arb_pattern_bindings_unique property test"
```

---

## Completion Checklist

- [ ] arb_pattern tracks used names
- [ ] Generated patterns have unique bindings
- [ ] test_arb_pattern_bindings_unique passes
- [ ] Other proptest helpers tests still pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 4
**Priority:** Low (non-blocking follow-up)
