# TASK-245: Redesign SmtContext Threading

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Remove or redesign `unsafe impl Send/Sync` for `SmtContext` to eliminate undefined behavior risk.

**Spec Reference:** Security best practices, Z3 API documentation

**File Locations:**
- Modify: `crates/ash-typeck/src/smt.rs:118`
- Test: `crates/ash-typeck/tests/smt_threading_tests.rs` (create)

---

## Background

The audit flagged critical security issue: `SmtContext` has `unsafe impl Send + Sync` with weak justification. The comment says "used single-threaded" but the type is globally `Send + Sync`. This is unsound.

Current code:
```rust
// crates/ash-typeck/src/smt.rs:118
unsafe impl Send for SmtContext {}
unsafe impl Sync for SmtContext {}
```

Options:
1. **Make it !Send !Sync** - Force single-threaded use (safest)
2. **Worker thread** - All Z3 access through dedicated thread
3. **Document sync boundary** - If Send/Sync needed, prove Z3 is thread-safe

Recommended: Option 1 or 2. Option 3 requires Z3 expertise we shouldn't assume.

---

## Step 1: Analyze Current Usage

Find all uses of SmtContext:

```bash
grep -rn "SmtContext" crates/ash-typeck/src/
grep -rn "Send\|Sync" crates/ash-typeck/src/smt.rs
```

Check if it's actually used across threads:

```bash
grep -rn "spawn\|thread\|async" crates/ash-typeck/src/ | grep -i smt
```

---

## Step 2: Design Safe Alternative

### Option A: !Send !Sync (Simplest)

```rust
use std::marker::PhantomData;
use std::rc::Rc;

pub struct SmtContext {
    context: Box<z3::Context>,
    // PhantomData<Rc<()>> makes this !Send !Sync
    _not_send_sync: PhantomData<Rc<()>>,
}

// Deliberately NOT implementing Send or Sync
```

### Option B: Thread-Local or Dedicated Thread

```rust
// Use thread_local! for per-thread contexts
thread_local! {
    static Z3_CONTEXT: RefCell<Option<SmtContext>> = RefCell::new(None);
}

// Or spawn dedicated worker thread for all Z3 operations
```

### Decision

Check if SmtContext needs to move between threads. If not, use Option A.

---

## Step 3: Write Failing Test

```rust
// crates/ash-typeck/tests/smt_threading_tests.rs

// This should NOT compile if we make SmtContext !Send !Sync:
// fn assert_send<T: Send>() {}
// fn assert_sync<T: Sync>() {}
// 
// fn test_not_send() {
//     assert_send::<SmtContext>();  // Should fail
// }
// fn test_not_sync() {
//     assert_sync::<SmtContext>();  // Should fail
// }

// Instead, test that single-threaded use works:
#[test]
fn test_smt_single_threaded() {
    let ctx = SmtContext::new();
    // Use ctx... should work fine
}

// Test that we can create multiple contexts in same thread
#[test]
fn test_multiple_contexts_same_thread() {
    let ctx1 = SmtContext::new();
    let ctx2 = SmtContext::new();
    // Both usable independently
}
```

---

## Step 4: Implement Safe SmtContext

Replace unsafe impls with PhantomData approach:

```rust
// crates/ash-typeck/src/smt.rs

use std::marker::PhantomData;
use std::rc::Rc;

pub struct SmtContext {
    context: Box<z3::Context>,
    solver: z3::Solver,
    _not_send_sync: PhantomData<Rc<()>>,  // Makes !Send !Sync
}

impl SmtContext {
    pub fn new() -> Self {
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);
        let solver = z3::Solver::new(&ctx);
        
        Self {
            context: Box::new(ctx),
            solver,
            _not_send_sync: PhantomData,
        }
    }
    
    // ... other methods ...
}

// NOTE: No `unsafe impl Send` or `unsafe impl Sync`
// The PhantomData<Rc<()>> ensures these are NOT implemented
```

---

## Step 5: Fix Compilation Errors

If any code was relying on SmtContext being Send/Sync:

```bash
cargo check --package ash-typeck
```

Fix approaches:
1. **Create context per thread** - If threaded use needed
2. **Use thread-local** - Store context in thread_local
3. **Dedicated thread** - Spawn Z3 worker thread

Most likely: Ash typeck is single-threaded, so no changes needed.

---

## Step 6: Run Tests

```bash
cargo test --package ash-typeck smt -v
cargo check --package ash-typeck --all-targets
```

---

## Step 7: Commit

```bash
git add crates/ash-typeck/src/smt.rs
git add crates/ash-typeck/tests/smt_threading_tests.rs
git commit -m "security: remove unsafe Send/Sync from SmtContext (TASK-245)

- Add PhantomData<Rc<()>> to make SmtContext !Send !Sync
- Remove unsafe impl Send/Sync blocks
- Add tests for single-threaded usage
- Z3 context is now thread-safe by construction"
```

---

## Step 8: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-245 implementation"
  context: |
    Files to verify:
    - crates/ash-typeck/src/smt.rs (SmtContext definition)
    - crates/ash-typeck/tests/smt_threading_tests.rs
    
    Security requirement:
    1. No unsafe impl Send for SmtContext
    2. No unsafe impl Sync for SmtContext
    3. SmtContext is !Send (verify with compile test if possible)
    4. SmtContext is !Sync (verify with compile test if possible)
    5. All existing functionality still works
    
    Run and report:
    1. cargo test --package ash-typeck
    2. cargo check --package ash-typeck --all-targets
    3. cargo clippy --package ash-typeck --all-targets --all-features -- -D warnings
    4. Verify no unsafe Send/Sync impls remain
    5. Check for any code relying on Send/Sync
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Current usage analyzed
- [ ] Design decision made (likely !Send !Sync)
- [ ] Tests written
- [ ] Unsafe impls removed
- [ ] PhantomData added
- [ ] Compilation errors fixed
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added (security fix)

---

**Estimated Hours:** 8
**Blocked by:** None
**Blocks:** None

**Security Impact:** HIGH - Removes undefined behavior risk
