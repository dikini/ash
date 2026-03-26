# TASK-242: Replace Yield Placeholder Lowering

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Replace `SurfaceWorkflow::Yield` placeholder lowering with proper CoreWorkflow transformation.

**Spec Reference:** SPEC-023 (Proxy Workflows), Lowering semantics

**File Locations:**
- Modify: `crates/ash-parser/src/lower.rs:388`
- Test: `crates/ash-parser/tests/yield_lowering_tests.rs` (create)

---

## Background

The audit found that `SurfaceWorkflow::Yield` is still lowered to `CoreWorkflow::Done` - a placeholder that loses all yield semantics.

Current code:
```rust
SurfaceWorkflow::Yield { .. } => CoreWorkflow::Done
```

Target: Proper lowering that preserves role, request, continuation.

---

## Step 1: Understand Current AST

Check SurfaceWorkflow::Yield definition:

```bash
grep -n "enum SurfaceWorkflow" crates/ash-parser/src/surface.rs
grep -n "Yield" crates/ash-parser/src/surface.rs
```

Check CoreWorkflow::Yield definition:

```bash
grep -n "enum CoreWorkflow" crates/ash-core/src/ast.rs
grep -n "Yield" crates/ash-core/src/ast.rs
```

Expected structure:
```rust
// Surface
Yield {
    role: RoleRef,
    request: Box<SurfaceExpr>,
    expected_response_type: Option<TypeExpr>,
    continuation: Box<SurfaceWorkflow>,
    span: Span,
}

// Core
Yield {
    role: String,
    request: Box<CoreExpr>,
    expected_response_type: Option<Type>,
    continuation: Box<CoreWorkflow>,
    span: Span,
}
```

---

## Step 2: Write Failing Test

```rust
// crates/ash-parser/tests/yield_lowering_tests.rs
use ash_parser::*;

#[test]
fn test_yield_lowering_not_done() {
    let surface = parse_workflow(r#"
        workflow test {
            yield role(ai_assistant) Request { data: 42 }
            resume result : Response { ... }
        }
    "#).unwrap();
    
    let core = lower_workflow(&surface);
    
    // Should NOT be CoreWorkflow::Done
    assert!(!matches!(core.body, CoreWorkflow::Done));
    
    // Should be CoreWorkflow::Yield
    assert!(matches!(core.body, CoreWorkflow::Yield { .. }));
}

proptest! {
    #[test]
    fn test_yield_lowering_preserves_structure(role in "[a-z_]+") {
        // Property: Lowering preserves role name, request, continuation
    }
}
```

---

## Step 3: Implement Lowering

Modify `crates/ash-parser/src/lower.rs:388`:

```rust
SurfaceWorkflow::Yield {
    role,
    request,
    expected_response_type,
    continuation,
    span,
} => {
    CoreWorkflow::Yield {
        role: role.to_string(),
        request: Box::new(lower_expr(request)),
        expected_response_type: expected_response_type
            .map(|t| lower_type(&t)),
        continuation: Box::new(lower_workflow(continuation)),
        span: lower_span(*span),
    }
}
```

Ensure `lower_workflow` handles `SurfaceWorkflow::Resume` for continuation.

---

## Step 4: Test Resume Lowering

Verify `resume` clause lowers correctly:

```rust
#[test]
fn test_resume_lowering() {
    // Yield with resume clause should lower to:
    // Yield { ..., continuation: Match { arms: [...] } }
}
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-parser yield_lowering -v
cargo test --package ash-parser  # ensure no regressions
```

---

## Step 6: Commit

```bash
git add crates/ash-parser/src/lower.rs
git add crates/ash-parser/tests/yield_lowering_tests.rs
git commit -m "fix: replace Yield placeholder lowering (TASK-242)

- Implement proper SurfaceWorkflow::Yield → CoreWorkflow::Yield lowering
- Preserve role, request, expected_response_type, continuation
- Add property tests for lowering structure preservation
- Enable TASK-243 YIELD execution work"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-242 implementation"
  context: |
    Files to verify:
    - crates/ash-parser/src/lower.rs (Yield lowering)
    - crates/ash-parser/tests/yield_lowering_tests.rs
    
    Spec reference: SPEC-023 lowering semantics
    Requirements:
    1. Yield not lowered to Done
    2. Role name preserved as string
    3. Request expression lowered
    4. Response type lowered if present
    5. Continuation lowered recursively
    6. Span preserved
    
    Run and report:
    1. cargo test --package ash-parser yield
    2. cargo clippy --package ash-parser --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-parser
    4. Check lowering preserves all Yield fields
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Failing tests written
- [ ] Yield lowering implemented
- [ ] Resume clause handling verified
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 8
**Blocked by:** None
**Blocks:** TASK-243 (YIELD execution)
