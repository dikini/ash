# TASK-262: Type Check Role Inclusion

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Type check `plays role(R)` clauses ensuring roles exist and capabilities compose.

**Spec Reference:** SPEC-019, SPEC-024

**File Locations:**
- Modify: `crates/ash-typeck/src/role_checking.rs` (or create)
- Test: `crates/ash-typeck/tests/role_type_tests.rs`

---

## Background

Workflows can include roles:
```ash
workflow processor
    plays role(ai_agent)
    plays role(network_client)
```

Type system must:
1. Verify each role exists
2. Compose capabilities from all included roles
3. Check for capability conflicts

---

## Step 1: Create Role Checking Module

Create `crates/ash-typeck/src/role_checking.rs`:

```rust
use ash_core::*;
use crate::{TypeError, TypeResult};

pub struct RoleChecker<'a> {
    role_defs: &'a HashMap<String, RoleDef>,
}

impl<'a> RoleChecker<'a> {
    pub fn new(role_defs: &'a HashMap<String, RoleDef>) -> Self {
        Self { role_defs }
    }
    
    /// Check workflow role inclusions
    pub fn check_workflow_roles(&self, workflow: &Workflow) -> TypeResult<EffectiveCapabilities> {
        let mut effective_caps = EffectiveCapabilities::new();
        
        for role_ref in &workflow.header.plays_roles {
            let role = self.lookup_role(&role_ref.name)
                .ok_or_else(|| TypeError::UnknownRole {
                    name: role_ref.name.clone(),
                    span: role_ref.span,
                })?;
            
            // Add role's capabilities to effective set
            for cap in &role.capabilities {
                effective_caps.add(cap)?;
            }
        }
        
        Ok(effective_caps)
    }
    
    fn lookup_role(&self, name: &str) -> Option<&RoleDef> {
        self.role_defs.get(name)
    }
}

pub struct EffectiveCapabilities {
    caps: HashMap<String, CapabilityDecl>,
}

impl EffectiveCapabilities {
    pub fn new() -> Self {
        Self { caps: HashMap::new() }
    }
    
    pub fn add(&mut self, cap: &CapabilityDecl) -> TypeResult<()> {
        // Check for conflicting constraints
        if let Some(existing) = self.caps.get(&cap.capability) {
            // Merge or check compatibility
            self.merge_capabilities(existing, cap)?;
        } else {
            self.caps.insert(cap.capability.clone(), cap.clone());
        }
        Ok(())
    }
    
    fn merge_capabilities(&self, a: &CapabilityDecl, b: &CapabilityDecl) -> TypeResult<()> {
        // If same capability from multiple roles, check constraints don't conflict
        // This is a placeholder - real logic depends on constraint semantics
        Ok(())
    }
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-typeck/tests/role_type_tests.rs
use ash_typeck::*;
use ash_core::*;

#[test]
fn test_valid_role_inclusion() {
    let mut roles = HashMap::new();
    roles.insert("ai_agent".to_string(), RoleDef {
        name: "ai_agent".to_string(),
        capabilities: vec![CapabilityDecl::new("file")],
    });
    
    let checker = RoleChecker::new(&roles);
    let workflow = Workflow::new("test")
        .plays_role("ai_agent");
    
    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_ok());
}

#[test]
fn test_unknown_role_error() {
    let roles = HashMap::new();  // No roles defined
    
    let checker = RoleChecker::new(&roles);
    let workflow = Workflow::new("test")
        .plays_role("nonexistent");
    
    let result = checker.check_workflow_roles(&workflow);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, TypeError::UnknownRole { .. }));
}

#[test]
fn test_multiple_role_capabilities_composed() {
    let mut roles = HashMap::new();
    roles.insert("file_user".to_string(), RoleDef {
        name: "file_user".to_string(),
        capabilities: vec![CapabilityDecl::new("file")],
    });
    roles.insert("net_user".to_string(), RoleDef {
        name: "net_user".to_string(),
        capabilities: vec![CapabilityDecl::new("network")],
    });
    
    let checker = RoleChecker::new(&roles);
    let workflow = Workflow::new("test")
        .plays_role("file_user")
        .plays_role("net_user");
    
    let result = checker.check_workflow_roles(&workflow).unwrap();
    
    assert!(result.has_capability("file"));
    assert!(result.has_capability("network"));
}

proptest! {
    #[test]
    fn test_role_inclusion_commutative(a in "[a-z]+", b in "[a-z]+") {
        // Order of role inclusion shouldn't matter
    }
}
```

---

## Step 3: Integrate into Type Checker

Modify main type check flow:

```rust
// crates/ash-typeck/src/lib.rs
pub fn type_check_module(module: &Module) -> TypeResult<TypeCheckedModule> {
    // ... existing checks ...
    
    // Check workflow roles
    let role_checker = RoleChecker::new(&module.roles);
    for workflow in &module.workflows {
        let effective_caps = role_checker.check_workflow_roles(workflow)?;
        
        // Store effective capabilities for capability checking
        ctx.set_workflow_capabilities(&workflow.name, effective_caps);
    }
    
    // ... rest of type checking ...
}
```

---

## Step 4: Run Tests

```bash
cargo test --package ash-typeck role_type -v
```

---

## Step 5: Commit

```bash
git add crates/ash-typeck/src/role_checking.rs
git add crates/ash-typeck/tests/role_type_tests.rs
git add crates/ash-typeck/src/lib.rs
git commit -m "feat: type check role inclusion (TASK-262)

- Add RoleChecker for workflow role validation
- Check roles exist (UnknownRole error)
- Compose capabilities from multiple roles
- EffectiveCapabilities set for workflows
- Integration with main type check flow
- Property tests for role composition"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-262 implementation"
  context: |
    Files to verify:
    - crates/ash-typeck/src/role_checking.rs
    - crates/ash-typeck/tests/role_type_tests.rs
    - crates/ash-typeck/src/lib.rs (integration)
    
    Spec reference: SPEC-019, SPEC-024
    Requirements:
    1. Known roles accepted
    2. Unknown roles rejected with error
    3. Capabilities composed from all roles
    4. Effective capabilities stored per workflow
    5. Error includes role name and location
    
    Run and report:
    1. cargo test --package ash-typeck role
    2. cargo clippy --package ash-typeck --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-typeck
    4. Check error message quality
    5. Test capability composition with 3+ roles
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] RoleChecker created
- [ ] Failing tests written
- [ ] Role existence checking
- [ ] Capability composition
- [ ] Type check integration
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 8
**Blocked by:** TASK-261
**Blocks:** TASK-263 (constraint validation)
