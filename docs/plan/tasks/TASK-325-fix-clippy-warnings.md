# TASK-325: Fix Remaining Clippy Warnings

## Status: 🟡 Medium - Code Quality

## Problem

Phase 50 claimed "all clippy warnings fixed" in `docs/plan/PLAN-INDEX.md:1384`, but warnings remain:

```bash
$ cargo clippy --workspace --all-targets --all-features --quiet

warning: redundant closure
   --> crates/ash-engine/src/lib.rs:261:39
    |
261 |                 args: args.iter().map(|ty| Self::surface_type_to_typeck(ty)).collect(),
    |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: replace the closure with the associated function itself: `Self::surface_type_to_typeck`

warning: redundant closure
  --> crates/ash-engine/tests/role_runtime_integration_tests.rs:25:46
   |
25 |         authority: authority.into_iter().map(|c| c.into()).collect(),
   |                                              ^^^^^^^^^^^^
   |
    = help: replace the closure with the method itself: `std::convert::Into::into`

warning: redundant clone
   --> crates/ash-engine/tests/role_runtime_integration_tests.rs:830:21
    |
830 |     let cloned = ctx.clone();
    |                     ^^^^^^^^
    |
    = help: remove this

warning: temporary with significant `Drop` can be early dropped
  --> crates/ash-engine/tests/e2e_capability_provider_tests.rs:64:21
   |
63 | |         if let Some(ref state) = self.shared_state {
   |
64 | |             let mut guard = state.lock().unwrap();
   | |                     ^^^^^
   |
    = note: this might lead to unnecessary resource contention
```

This is technical debt, but it also means the recorded closeout status overstates completion.

## Resolution

**Fix all remaining clippy warnings.** No warnings are acceptable technical debt. Ignored tests must be clearly documented with assigned tasks.

## Implementation

### Warning 1: Redundant Closure in Production Code

**File:** `crates/ash-engine/src/lib.rs:261`

**Current:**
```rust
args: args.iter().map(|ty| Self::surface_type_to_typeck(ty)).collect(),
```

**Fix:**
```rust
args: args.iter().map(Self::surface_type_to_typeck).collect(),
```

### Warning 2: Redundant Closure in Test Code

**File:** `crates/ash-engine/tests/role_runtime_integration_tests.rs:25`

**Current:**
```rust
authority: authority.into_iter().map(|c| c.into()).collect(),
```

**Fix:**
```rust
authority: authority.into_iter().map(std::convert::Into::into).collect(),
```

### Warning 3: Redundant Clone in Test Code

**File:** `crates/ash-engine/tests/role_runtime_integration_tests.rs:830`

**Current:**
```rust
let cloned = ctx.clone();
```

**Fix:** Remove the unused clone or use the cloned value.

### Warning 4: MutexGuard Drop in Test Code

**File:** `crates/ash-engine/tests/e2e_capability_provider_tests.rs:64`

**Current:**
```rust
if let Some(ref state) = self.shared_state {
    let mut guard = state.lock().unwrap();
    guard.insert(
        format!("{}_observed", self.name),
        Value::Int(self.get_count() as i64),
    );
}  // guard dropped here at end of scope
```

**Fix:** Explicitly drop the guard or scope it:
```rust
if let Some(ref state) = self.shared_state {
    let mut guard = state.lock().unwrap();
    guard.insert(
        format!("{}_observed", self.name),
        Value::Int(self.get_count() as i64),
    );
    drop(guard);  // Explicit drop for clarity
}
```

Or use a block to limit scope:
```rust
if let Some(ref state) = self.shared_state {
    {
        let mut guard = state.lock().unwrap();
        guard.insert(
            format!("{}_observed", self.name),
            Value::Int(self.get_count() as i64),
        );
    }  // guard dropped here
}
```

## Files to Modify

- `crates/ash-engine/src/lib.rs:261` - Fix redundant closure
- `crates/ash-engine/tests/role_runtime_integration_tests.rs:25` - Fix redundant closure
- `crates/ash-engine/tests/role_runtime_integration_tests.rs:830` - Fix/remove redundant clone
- `crates/ash-engine/tests/e2e_capability_provider_tests.rs:64` - Fix mutex guard drop

## Verification

```bash
# Should produce no warnings
$ cargo clippy --workspace --all-targets --all-features --quiet
# (no output = success)

# Also verify tests still pass
$ cargo test --workspace --quiet
```

## Completion Checklist

- [ ] Fix redundant_closure in `crates/ash-engine/src/lib.rs:261`
- [ ] Fix redundant_closure_for_method_calls in test file line 25
- [ ] Fix redundant_clone in test file line 830
- [ ] Fix temporary_with_significant_drop in e2e test line 64
- [ ] `cargo clippy --workspace --all-targets --all-features --quiet` produces no warnings
- [ ] All tests still pass
- [ ] CHANGELOG.md updated

**Estimated Hours:** 1
**Priority:** Medium (code quality)
**Related:** TASK-321-fix-clippy-warnings.md (Phase 50 task - may have addressed some warnings)

## Related

- Previous attempt: TASK-321-fix-clippy-warnings.md
- Policy: No clippy warnings in production or test code
- CI: Should gate on `cargo clippy --workspace --all-targets --all-features --quiet`
