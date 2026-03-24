# TASK-236: Implement Role Runtime Enforcement

## Status: Blocked on TASK-235

## Description

Implement runtime enforcement of role `authority` and `obligations` as specified in SPEC-019.

## Background

After TASK-235 completes, SPEC-019 will define complete runtime semantics for roles. This task implements:
1. Authority checking before capability invocation
2. Obligation tracking and discharge verification
3. Role context management
4. Integration with existing capability and obligation systems

## Requirements

### 1. RoleContext Implementation

Create `crates/ash-interp/src/role_context.rs`:

```rust
pub struct RoleContext {
    pub active_role: Role,
    pub discharged_obligations: RefCell<HashSet<String>>,
}

impl RoleContext {
    /// Check if capability is in role's authority
    pub fn can_access(&self, capability: &Capability) -> bool;
    
    /// Check if role obligation has been discharged
    pub fn is_obligation_discharged(&self, obligation: &str) -> bool;
    
    /// Mark role obligation as discharged
    pub fn discharge_obligation(&self, obligation: &str) -> bool;
    
    /// Check all role obligations are discharged
    pub fn all_obligations_discharged(&self) -> bool;
}
```

### 2. Authority Enforcement

Modify `crates/ash-interp/src/capability_policy.rs`:

```rust
pub fn evaluate_capability_access(
    &self,
    capability: &Capability,
    role_context: &RoleContext,
    // ... existing params
) -> PolicyResult {
    // NEW: Check role authority first
    if !role_context.can_access(capability) {
        return PolicyResult::Denied(Reason::NotInRoleAuthority);
    }
    
    // Existing policy evaluation
    self.evaluate_policy(...)
}
```

### 3. Obligation Enforcement

Integrate with existing obligation system:

```rust
// In workflow execution
pub fn check_workflow_completion(
    workflow: &Workflow,
    local_obligations: &ObligationSet,
    role_context: Option<&RoleContext>,
) -> Result<(), ObligationError> {
    // Check local obligations (from SPEC-022)
    local_obligations.verify_all_discharged()?;
    
    // NEW: Check role obligations
    if let Some(role_ctx) = role_context {
        if !role_ctx.all_obligations_discharged() {
            return Err(ObligationError::RoleObligationsPending);
        }
    }
    
    Ok(())
}
```

### 4. Role Assignment at Spawn

Extend spawn syntax and execution:

```rust
// In spawn evaluation
pub fn eval_spawn(
    &self,
    workflow_type: &str,
    role: Option<Role>,  // NEW: optional role assignment
    init_args: &Value,
) -> EvalResult<Instance> {
    let role_context = role.map(RoleContext::new);
    // ... spawn with role context
}
```

## Test Requirements

### Authority Tests
- [ ] Role with authority can access allowed capability
- [ ] Role without authority cannot access restricted capability
- [ ] Authority check happens before policy evaluation
- [ ] Missing role context fails closed (deny access)

### Obligation Tests
- [ ] Role obligation can be discharged via check
- [ ] Double-discharge returns false (linear)
- [ ] Workflow completion blocked with pending role obligations
- [ ] Role obligations tracked separately from local obligations

### Integration Tests
- [ ] Spawn with role assignment
- [ ] Workflow with both local and role obligations
- [ ] Authority denial logged to audit trail
- [ ] Obligation discharge logged to audit trail

## Acceptance Criteria

- [ ] RoleContext implemented with all methods
- [ ] Authority checked before capability access
- [ ] Role obligations tracked and enforced
- [ ] Workflow completion checks role obligations
- [ ] All test cases pass
- [ ] Audit trail captures role checks
- [ ] Works with existing capability/policy system

## Dependencies

- TASK-235: SPEC-019 Role Runtime Semantics (must be complete)
- SPEC-022: Workflow obligations (already implemented)

## Estimated Effort

3-4 weeks (1 week RoleContext, 1 week authority integration, 1 week obligation integration, 1 week testing)

## Related Documents

- `docs/spec/SPEC-019-ROLE-RUNTIME-SEMANTICS.md` (to be created)
- `crates/ash-interp/src/capability_policy.rs`
- `crates/ash-core/src/ast.rs` (Role struct)
- `crates/ash-interp/src/context.rs`
