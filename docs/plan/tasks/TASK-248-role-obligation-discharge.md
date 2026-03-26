# TASK-248: Fix Role Obligation Discharge

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Fix `RoleContext::discharge` to only accept declared obligations, per SPEC-019.

**Spec Reference:** SPEC-019 (Role Semantics), Obligation discharge semantics

**File Locations:**
- Modify: `crates/ash-interp/src/role_context.rs:86`
- Test: `crates/ash-interp/tests/role_obligation_tests.rs` (create)

---

## Background

The audit found `RoleContext::discharge` accepts any string:

```rust
// Current (line 86)
pub fn discharge(&mut self, obligation: &str) {
    self.discharged.insert(obligation.to_string());  // No validation!
}
```

This conflicts with SPEC-019: role obligations are closed, mandatory obligations. Only declared obligations should be dischargeable.

---

## Step 1: Understand Role Obligations

Find role definition structure:

```bash
grep -n "struct Role\|struct RoleDef" crates/ash-core/src/
grep -n "obligations" crates/ash-core/src/role.rs
```

Check RoleContext structure:

```bash
grep -n "struct RoleContext" crates/ash-interp/src/role_context.rs
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-interp/tests/role_obligation_tests.rs
use ash_interp::*;
use ash_core::*;

#[test]
fn test_discharge_declared_obligation_succeeds() {
    let role = Role::new("test_role")
        .with_obligation("audit_log")
        .with_obligation("notify");
    
    let mut ctx = RoleContext::new(role);
    
    // Should succeed - obligation was declared
    ctx.discharge("audit_log").unwrap();
    assert!(ctx.is_discharged("audit_log"));
}

#[test]
fn test_discharge_undeclared_obligation_fails() {
    let role = Role::new("test_role")
        .with_obligation("audit_log");
    
    let mut ctx = RoleContext::new(role);
    
    // Should fail - obligation not declared
    let result = ctx.discharge("undeclared_obligation");
    assert!(result.is_err());
}

#[test]
fn test_discharge_unknown_string_fails() {
    let role = Role::new("test_role")
        .with_obligation("audit_log");
    
    let mut ctx = RoleContext::new(role);
    
    // Random string not in role obligations
    let result = ctx.discharge("random_typo");
    assert!(result.is_err());
}

#[test]
fn test_all_obligations_must_be_discharged() {
    let role = Role::new("test_role")
        .with_obligation("audit_log")
        .with_obligation("notify");
    
    let ctx = RoleContext::new(role);
    
    ctx.discharge("audit_log").unwrap();
    // notify not discharged
    
    assert!(!ctx.all_obligations_discharged());
}
```

---

## Step 3: Implement Closed Obligation Set

Modify `crates/ash-interp/src/role_context.rs`:

```rust
pub struct RoleContext {
    role: RoleRef,
    /// Only obligations declared on the role can be discharged
    declared_obligations: HashSet<String>,
    discharged: HashSet<String>,
}

impl RoleContext {
    pub fn new(role: RoleRef) -> Self {
        let declared = role.obligations.iter()
            .map(|o| o.name.clone())
            .collect();
        
        Self {
            role,
            declared_obligations: declared,
            discharged: HashSet::new(),
        }
    }
    
    /// Discharge a declared obligation
    /// 
    /// # Errors
    /// Returns Err if obligation was not declared on the role
    pub fn discharge(&mut self, obligation: &str) -> Result<(), RoleError> {
        if !self.declared_obligations.contains(obligation) {
            return Err(RoleError::UndeclaredObligation {
                name: obligation.to_string(),
                role: self.role.name.clone(),
                declared: self.declared_obligations.clone(),
            });
        }
        
        self.discharged.insert(obligation.to_string());
        Ok(())
    }
    
    pub fn is_discharged(&self, obligation: &str) -> bool {
        self.discharged.contains(obligation)
    }
    
    pub fn all_obligations_discharged(&self) -> bool {
        self.declared_obligations.iter()
            .all(|o| self.discharged.contains(o))
    }
    
    /// Returns obligations still pending
    pub fn pending_obligations(&self) -> Vec<String> {
        self.declared_obligations.iter()
            .filter(|o| !self.discharged.contains(*o))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum RoleError {
    UndeclaredObligation {
        name: String,
        role: String,
        declared: HashSet<String>,
    },
}

impl std::fmt::Display for RoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoleError::UndeclaredObligation { name, role, declared } => {
                write!(f, 
                    "Cannot discharge obligation '{}' on role '{}': 
                     not declared. Declared obligations: {:?}",
                    name, role, declared
                )
            }
        }
    }
}

impl std::error::Error for RoleError {}
```

---

## Step 4: Update Callers

Find and update code that calls `discharge`:

```bash
grep -rn "\.discharge(" crates/ash-interp/src/
```

Update to handle Result:

```rust
// Before
ctx.discharge("audit_log");

// After
ctx.discharge("audit_log")
    .expect("audit_log should be declared on role");
// Or proper error handling
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-interp role_obligation -v
```

---

## Step 6: Commit

```bash
git add crates/ash-interp/src/role_context.rs
git add crates/ash-interp/tests/role_obligation_tests.rs
git commit -m "fix: enforce declared obligations in RoleContext::discharge (TASK-248)

- RoleContext now tracks declared_obligations from role definition
- discharge() returns Err for undeclared obligations
- Add RoleError::UndeclaredObligation with helpful message
- Add all_obligations_discharged() and pending_obligations()
- Tests for valid discharge, invalid discharge, and completion check
- Aligns with SPEC-019 closed obligation semantics"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-248 implementation"
  context: |
    Files to verify:
    - crates/ash-interp/src/role_context.rs (discharge implementation)
    - crates/ash-interp/tests/role_obligation_tests.rs
    
    Spec reference: SPEC-019 role semantics
    Requirements:
    1. discharge() checks against declared obligations
    2. Undeclared obligations return error
    3. Error includes list of valid obligations
    4. Declared obligations discharge successfully
    5. all_obligations_discharged() works
    6. pending_obligations() lists remaining
    
    Run and report:
    1. cargo test --package ash-interp role_obligation
    2. cargo clippy --package ash-interp --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-interp
    4. Check discharge() callers updated
    5. Verify SPEC-019 compliance
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Role obligation structure understood
- [ ] Failing tests written
- [ ] RoleContext modified with declared_obligations
- [ ] discharge() returns Result with validation
- [ ] RoleError added
- [ ] Callers updated
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 6
**Blocked by:** None
**Blocks:** None
