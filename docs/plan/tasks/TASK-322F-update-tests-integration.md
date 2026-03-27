# TASK-322F: Update Tests and Integration for capabilities: Syntax

## Status: 🔴 Blocking - TASK-322 Sub-task

## Problem

All existing tests use old `authority:` syntax and need updating. Integration tests needed for end-to-end constraint enforcement.

## Scope

**This task updates all tests and adds integration tests.** Lowering is already updated (TASK-322E).

## Implementation

### 1. Update All Role Tests

Find and replace `authority:` with `capabilities:` in test code:

```bash
# Find all occurrences
grep -r "authority:" crates/ --include="*.rs"

# Update each occurrence:
# authority: [file, network]
# ->
# capabilities: [file, network]
```

### 2. Update Test Helpers

```rust
// Old helper
fn create_role_def(name: &str, authority: Vec<&str>) -> RoleDef {
    RoleDef {
        name: name.into(),
        authority: authority.into_iter().map(Into::into).collect(),  // OLD
        obligations: vec![],
        span: test_span(),
    }
}

// New helper
fn create_role_def(name: &str, capabilities: Vec<CapabilityDecl>) -> RoleDef {
    RoleDef {
        name: name.into(),
        capabilities,  // NEW
        obligations: vec![],
        span: test_span(),
    }
}

// Convenience helper for simple capabilities
fn create_simple_capability(name: &str) -> CapabilityDecl {
    CapabilityDecl {
        capability: name.into(),
        constraints: None,
        span: Span::default(),
    }
}
```

### 3. Add End-to-End Integration Tests

```rust
#[tokio::test]
async fn test_e2e_role_constraint_enforcement() {
    let source = r#"
        capability file {
            effect: Operational,
        }
        
        role limited_agent {
            capabilities: [
                file @ { paths: ["/tmp/*"], read: true, write: false }
            ]
        }
        
        workflow read_file plays role(limited_agent) {
            act file.read with { path: "/tmp/test.txt" };
        }
    "#;
    
    let engine = Engine::new()
        .with_stdio_capabilities()
        .build()
        .unwrap();
    
    let result = engine.run(source).await;
    assert!(result.is_ok());  // /tmp/test.txt is allowed
}

#[tokio::test]
async fn test_e2e_role_constraint_violation() {
    let source = r#"
        capability file {
            effect: Operational,
        }
        
        role limited_agent {
            capabilities: [
                file @ { paths: ["/tmp/*"], read: true, write: false }
            ]
        }
        
        workflow write_file plays role(limited_agent) {
            act file.write with { path: "/etc/passwd", data: "x" };
        }
    "#;
    
    let engine = Engine::new()
        .with_stdio_capabilities()
        .build()
        .unwrap();
    
    let result = engine.run(source).await;
    assert!(result.is_err());  // /etc/passwd violates path constraint
    assert!(result.unwrap_err().to_string().contains("constraint"));
}
```

## TDD Steps

### Step 1: Count and List All Tests Needing Update

```bash
# Find all test files with authority:
grep -rl "authority:" crates/ --include="*.rs" | grep test

# Count occurrences
grep -r "authority:" crates/ --include="*.rs" | wc -l
```

### Step 2: Update Tests Systematically

Update each test file:
1. Change `authority:` to `capabilities:` in test data
2. Update `RoleDef` construction to use `capabilities` field
3. Add constraint tests where appropriate

### Step 3: Verify All Tests Pass

```bash
cargo test --workspace
# Expected: All pass
```

### Step 4: Add Integration Tests

Add new integration tests for constraint enforcement.

## Step 5: Code Review Sub-Process

Spawn a code review sub-agent to verify:

```
Review Focus for TASK-322F:
- All authority: syntax has been replaced
- Test helpers are consistent and reusable
- Integration tests cover realistic scenarios
- Edge cases are tested (empty constraints, multiple roles, etc.)
- No test is ignored without documentation
- Test names are descriptive
- No flakiness in async tests
```

### Review Checklist (Rust-Specific)

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets` clean (no new warnings)
- [ ] `cargo test --workspace` all tests pass
- [ ] `cargo test --workspace --all-features` passes (if applicable)
- [ ] No ignored tests without documented reason
- [ ] Property tests added for constraint checking
- [ ] Integration tests use realistic Ash code

### Review Output

Reviewer should provide:
1. **Critical issues** (must fix before merge)
2. **Suggestions** (can be addressed or noted for follow-up)
3. **Approval** or **Request Changes**

## Files to Modify

- `crates/ash-parser/src/parse_module.rs` - Update any authority: in tests
- `crates/ash-typeck/src/role_checking.rs` - Update test helpers
- `crates/ash-interp/src/role_runtime.rs` - Update test helpers
- `crates/ash-engine/tests/` - Add integration tests
- Any other test files with `authority:`

## Completion Checklist

- [ ] All `authority:` syntax replaced with `capabilities:` in tests
- [ ] All `RoleDef` construction uses `capabilities` field
- [ ] All parser tests pass
- [ ] All type checker tests pass
- [ ] All interpreter tests pass
- [ ] All engine tests pass
- [ ] New integration tests for constraint enforcement added
- [ ] `cargo test --workspace` passes
- [ ] **Code review completed** with no critical issues
- [ ] Review feedback addressed (if any)

**Estimated Hours:** 2-3 (including review)
**Priority:** Blocking
**Blocked By:** TASK-322E
**Blocks:** None (final task)
