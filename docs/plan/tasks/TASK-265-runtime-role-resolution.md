# TASK-265: Runtime Role Resolution

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement runtime resolution of roles to their capability grants.

**Spec Reference:** SPEC-019, SPEC-024

**File Locations:**
- Modify: `crates/ash-interp/src/role_runtime.rs` (or create)
- Test: `crates/ash-interp/tests/role_runtime_tests.rs`

---

## Background

At runtime, when a workflow executes:
1. Its `plays_roles` must be resolved to actual capability grants
2. The implicit default role (if any) must be included
3. Effective capabilities must be computed

---

## Step 1: Create Role Runtime Module

Create `crates/ash-interp/src/role_runtime.rs`:

```rust
use ash_core::*;
use std::collections::HashMap;

/// Runtime role registry
pub struct RoleRegistry {
    roles: HashMap<String, RoleDef>,
}

impl RoleRegistry {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, role: RoleDef) {
        self.roles.insert(role.name.clone(), role);
    }
    
    /// Resolve workflow roles to capability grants
    pub fn resolve_workflow_roles(
        &self,
        workflow: &Workflow,
    ) -> Result<RuntimeCapabilitySet, RoleError> {
        let mut caps = RuntimeCapabilitySet::new();
        
        for role_name in &workflow.header.plays_roles {
            let role = self.roles.get(role_name)
                .ok_or_else(|| RoleError::UnknownRole {
                    name: role_name.clone(),
                })?;
            
            for cap_decl in &role.capabilities {
                caps.grant(cap_decl)?;
            }
        }
        
        Ok(caps)
    }
    
    /// Get role by name
    pub fn get_role(&self, name: &str) -> Option<&RoleDef> {
        self.roles.get(name)
    }
}

/// Runtime capability grants for a workflow
#[derive(Debug, Clone)]
pub struct RuntimeCapabilitySet {
    grants: HashMap<String, CapabilityGrant>,
}

#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    pub capability: String,
    pub constraints: Option<ConstraintBlock>,
    pub granted_by: Vec<String>, // role names
}

impl RuntimeCapabilitySet {
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
        }
    }
    
    pub fn grant(&mut self, decl: &CapabilityDecl) -> Result<(), RoleError> {
        let name = decl.capability.clone();
        
        if let Some(existing) = self.grants.get_mut(&name) {
            // Merge grants
            existing.merge(decl)?;
        } else {
            self.grants.insert(name.clone(), CapabilityGrant {
                capability: name,
                constraints: decl.constraints.clone(),
                granted_by: vec![], // Filled in by caller
            });
        }
        
        Ok(())
    }
    
    pub fn has_capability(&self, name: &str) -> bool {
        self.grants.contains_key(name)
    }
    
    pub fn get_grant(&self, name: &str) -> Option<&CapabilityGrant> {
        self.grants.get(name)
    }
    
    /// Check if use is permitted given constraints
    pub fn check_use(
        &self,
        capability: &str,
        operation: &str,
        args: &Value,
    ) -> Result<(), CapabilityError> {
        let grant = self.grants.get(capability)
            .ok_or(CapabilityError::NotGranted)?;
        
        if let Some(constraints) = &grant.constraints {
            // Check operation against constraints
            self.check_constraints(operation, args, constraints)?;
        }
        
        Ok(())
    }
    
    fn check_constraints(
        &self,
        operation: &str,
        args: &Value,
        constraints: &ConstraintBlock,
    ) -> Result<(), CapabilityError> {
        // Placeholder: Real implementation checks against constraints
        // e.g., file paths, network hosts, etc.
        Ok(())
    }
}

#[derive(Debug)]
pub enum RoleError {
    UnknownRole { name: String },
    IncompatibleGrants { capability: String },
}

#[derive(Debug)]
pub enum CapabilityError {
    NotGranted,
    ConstraintViolation { reason: String },
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-interp/tests/role_runtime_tests.rs
use ash_interp::role_runtime::*;
use ash_core::*;

fn create_test_role(name: &str, caps: Vec<&str>) -> RoleDef {
    RoleDef {
        name: name.to_string(),
        capabilities: caps.into_iter()
            .map(|c| CapabilityDecl::new(c))
            .collect(),
        external: false,
        deterministic: true,
    }
}

#[test]
fn test_role_resolution() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("ai_agent", vec!["file", "process"]));
    
    let workflow = Workflow::new("test")
        .plays_role("ai_agent");
    
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();
    
    assert!(caps.has_capability("file"));
    assert!(caps.has_capability("process"));
    assert!(!caps.has_capability("network"));
}

#[test]
fn test_unknown_role_error() {
    let registry = RoleRegistry::new();
    
    let workflow = Workflow::new("test")
        .plays_role("nonexistent");
    
    let result = registry.resolve_workflow_roles(&workflow);
    assert!(result.is_err());
}

#[test]
fn test_multiple_roles_combined() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("file_user", vec!["file"]));
    registry.register(create_test_role("net_user", vec!["network"]));
    
    let workflow = Workflow::new("test")
        .plays_role("file_user")
        .plays_role("net_user");
    
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();
    
    assert!(caps.has_capability("file"));
    assert!(caps.has_capability("network"));
}

#[test]
fn test_capability_use_check() {
    let mut registry = RoleRegistry::new();
    registry.register(create_test_role("file_user", vec!["file"]));
    
    let workflow = Workflow::new("test")
        .plays_role("file_user");
    
    let caps = registry.resolve_workflow_roles(&workflow).unwrap();
    
    // Should succeed for granted capability
    assert!(caps.check_use("file", "read", &Value::Null).is_ok());
    
    // Should fail for ungranted capability
    assert!(caps.check_use("network", "get", &Value::Null).is_err());
}

proptest! {
    #[test]
    fn test_resolution_deterministic(roles in vec(arbitrary_role(), 1..5)) {
        // Same roles should always produce same capability set
    }
}
```

---

## Step 3: Integrate into Runtime

```rust
// crates/ash-interp/src/lib.rs or context.rs
pub struct RuntimeContext {
    // ... existing fields ...
    role_registry: RoleRegistry,
    current_capabilities: Option<RuntimeCapabilitySet>,
}

impl RuntimeContext {
    pub fn spawn_workflow(&mut self, workflow: &Workflow) -> Result<WorkflowHandle, ExecError> {
        // Resolve roles to capabilities
        let caps = self.role_registry
            .resolve_workflow_roles(workflow)
            .map_err(|e| ExecError::RoleResolutionFailed(e))?;
        
        // Store for capability checks during execution
        self.current_capabilities = Some(caps);
        
        // ... spawn workflow ...
    }
}
```

---

## Step 4: Run Tests

```bash
cargo test --package ash-interp role_runtime -v
```

---

## Step 5: Commit

```bash
git add crates/ash-interp/src/role_runtime.rs
git add crates/ash-interp/tests/role_runtime_tests.rs
git add crates/ash-interp/src/lib.rs
git commit -m "feat: runtime role resolution (TASK-265)

- Add RoleRegistry for runtime role lookup
- Resolve workflow plays_roles to capability grants
- RuntimeCapabilitySet for effective grants
- Capability use checking against grants
- Error handling for unknown roles
- Integration with workflow spawning
- Tests for resolution scenarios"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-265 implementation"
  context: |
    Files to verify:
    - crates/ash-interp/src/role_runtime.rs
    - crates/ash-interp/tests/role_runtime_tests.rs
    - crates/ash-interp/src/lib.rs
    
    Spec reference: SPEC-019, SPEC-024
    Requirements:
    1. Roles resolved from registry
    2. Unknown roles produce error
    3. Multiple roles combined
    4. Capabilities checked at runtime
    5. Registry integrates with context
    6. Errors have useful messages
    
    Run and report:
    1. cargo test --package ash-interp role_runtime
    2. cargo clippy --package ash-interp --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-interp
    4. Test error message quality
    5. Test capability denial
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] RoleRegistry created
- [ ] Failing tests written
- [ ] Role resolution
- [ ] Capability grants
- [ ] Use checking
- [ ] Runtime integration
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 8
**Blocked by:** TASK-264
**Blocks:** TASK-266 (constraint enforcement)
