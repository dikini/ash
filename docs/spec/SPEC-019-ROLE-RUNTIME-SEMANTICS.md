# SPEC-019: Role Runtime Semantics

## Status: Active

## 1. Overview

Define runtime enforcement of role `authority` and `obligations`. Currently, the runtime uses `Role` as a lightweight identity for policy evaluation. This specification extends roles to enforce authority (what capabilities a role can access) and obligations (what duties a role must fulfill).

**Target:** Release 0.6.0  
**Effort Estimate:** 3-4 weeks implementation

---

## 2. Role Structure

### 2.1 Core Definition

```rust
/// Core role metadata with authority and obligations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    pub name: Name,
    pub authority: Vec<Capability>,      // What the role CAN do
    pub obligations: Vec<RoleObligationRef>, // What the role MUST do
}

/// Reference to a role-level obligation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleObligationRef {
    pub name: Name,
}
```

### 2.2 Role Definition Syntax

```ash
role developer {
    authority: [git_read, git_write, test_run],
    obligations: [code_review, security_check]
}

role admin {
    authority: [system_restart, user_manage, git_read, git_write],
    obligations: [audit_log]
}
```

### 2.3 Semantic Properties

- **Authority is closed**: If a capability is not in `authority`, the role cannot access it
- **Obligations are mandatory**: All obligations must be discharged before workflow completion
- **Role assignment is static**: Assigned at spawn time, cannot change during execution
- **Authority check precedes policy**: Role authority checked before policy evaluation

---

## 3. Authority Enforcement

### 3.1 Authority Check

Before any capability invocation:

```rust
pub fn check_authority(
    role: &Role,
    capability: &Capability,
) -> Result<(), AuthorityError> {
    if role.authority.contains(capability) {
        Ok(())
    } else {
        Err(AuthorityError::NotAuthorized {
            role: role.name.clone(),
            capability: capability.name.clone(),
        })
    }
}
```

### 3.2 Integration with Capability Invocation

```rust
pub async fn invoke_capability(
    &self,
    capability: &Capability,
    args: &Value,
    role_context: &RoleContext,  // Contains active role
) -> Result<Value, CapabilityError> {
    // 1. Check role authority (NEW)
    role_context.check_authority(capability)?;
    
    // 2. Evaluate policy (existing)
    self.evaluate_policy(capability, role_context).await?;
    
    // 3. Execute capability (existing)
    self.execute(capability, args).await
}
```

### 3.3 Authority Error Handling

```rust
pub enum AuthorityError {
    NotAuthorized {
        role: Name,
        capability: Name,
    },
    NoRoleAssigned,  // Workflow has no role context
}
```

**Behavior:**
- Authority denial is a **hard error** (not policy-deny which can be escalated)
- Logged to audit trail with role and capability names
- Workflow execution stops (cannot proceed without authority)

### 3.4 Example

```ash
role developer {
    authority: [git_read, git_write]
}

workflow code_task {
    act git_read;      -- Allowed
    act git_write;     -- Allowed
    act deploy_prod;   -- ERROR: not in developer authority
}
```

---

## 4. Obligation Enforcement

### 4.1 Obligation Tracking

```rust
pub struct RoleContext {
    pub active_role: Role,
    pub discharged_obligations: RefCell<HashSet<Name>>,
}

impl RoleContext {
    /// Check if obligation has been discharged
    pub fn is_discharged(&self, obligation: &str) -> bool {
        self.discharged_obligations.borrow().contains(obligation)
    }
    
    /// Discharge an obligation (returns false if already discharged)
    pub fn discharge(&self, obligation: &str) -> bool {
        self.discharged_obligations.borrow_mut().insert(obligation.into())
    }
    
    /// Check all role obligations are discharged
    pub fn all_discharged(&self) -> bool {
        let discharged = self.discharged_obligations.borrow();
        self.active_role.obligations.iter().all(|o| discharged.contains(&o.name))
    }
    
    /// Get pending obligations
    pub fn pending_obligations(&self) -> Vec<Name> {
        let discharged = self.discharged_obligations.borrow();
        self.active_role.obligations.iter()
            .filter(|o| !discharged.contains(&o.name))
            .map(|o| o.name.clone())
            .collect()
    }
}
```

### 4.2 Discharging Role Obligations

Syntax for discharging role obligations (to be decided in DECISION-237):

**Option A: Role-qualified check**
```ash
check role.developer.code_review;
```

**Option B: Implicit role context**
```ash
check code_review;  -- Checks both local and role obligations
```

### 4.3 Workflow Completion Check

Before workflow completes (`ret` or `done`):

```rust
pub fn check_completion(
    workflow: &Workflow,
    local_obligations: &ObligationSet,
    role_context: Option<&RoleContext>,
) -> Result<(), CompletionError> {
    // Check local obligations (from SPEC-022)
    local_obligations.verify_all_discharged()?;
    
    // Check role obligations
    if let Some(ctx) = role_context {
        if !ctx.all_discharged() {
            return Err(CompletionError::RoleObligationsPending {
                pending: ctx.pending_obligations(),
            });
        }
    }
    
    Ok(())
}
```

### 4.4 Completion Error

```rust
pub enum CompletionError {
    LocalObligationsUndischarged(Vec<Name>),
    RoleObligationsPending { pending: Vec<Name> },
}
```

---

## 5. Role Assignment

### 5.1 Assignment at Spawn

```ash
-- Spawn with explicit role
spawn code_task {
    role: developer,      -- Assign developer role
    init: { repo: "myapp" }
}

-- Spawn without role (no authority/obligations enforced)
spawn utility_task {
    init: { data: value }
}
```

### 5.2 Role Context in Runtime

```rust
pub struct RuntimeContext {
    // ... existing fields ...
    pub role_context: Option<RoleContext>,  // NEW
}

impl RuntimeContext {
    pub fn with_role(mut self, role: Role) -> Self {
        self.role_context = Some(RoleContext::new(role));
        self
    }
}
```

### 5.3 Role Inheritance

**Option A: No inheritance (default)**
- Spawned workflows get explicit role or no role
- No automatic inheritance from parent

**Option B: Optional inheritance**
```ash
spawn child_task {
    role: inherit,  -- Inherit parent's role
    init: { ... }
}
```

**Recommendation:** Start with Option A (no inheritance), add Option B later if needed.

---

## 6. Integration Points

### 6.1 Module System

- Role definitions are module items
- Public roles exported: `pub role admin { ... }`
- Role resolution during lowering

### 6.2 Type Checker

- Verify role exists at spawn time
- Check role authority contains required capabilities
- Track which obligations need discharge

### 6.3 Audit Trail

Log events:
- `RoleAssigned`: Role assigned to workflow instance
- `AuthorityDenied`: Capability access blocked by authority check
- `ObligationDischarged`: Role obligation discharged
- `CompletionBlocked`: Workflow completion blocked by pending obligations

### 6.4 Policy System

Authority check happens **before** policy evaluation:

```
Capability Request
      │
      ▼
Role Authority Check ──❌──► AuthorityError
      │
      ✓
      ▼
Policy Evaluation ──❌──► PolicyDecision::Deny/Escalate
      │
      ✓
      ▼
Capability Execution
```

---

## 7. Examples

### 7.1 Simple Authority Check

```ash
role reader {
    authority: [file_read]
}

workflow read_task {
    let content = act file_read with { path: "/data.txt" };
    ret content;
}

-- Spawned with role: succeeds
spawn read_task { role: reader, init: {} }

-- Spawned without role: fails (no authority)
spawn read_task { init: {} }  -- ERROR: no role, cannot access file_read
```

### 7.2 Obligation Tracking

```ash
role reviewer {
    authority: [approve, reject],
    obligations: [check_guidelines]
}

workflow review_pr {
    -- Do review work
    let result = analyze_changes();
    
    -- Discharge obligation
    check reviewer.check_guidelines;
    
    -- Complete workflow
    if result.ok {
        act approve;
    } else {
        act reject;
    }
}

-- If check_guidelines not discharged before ret/act:
-- ERROR: RoleObligationsPending ["check_guidelines"]
```

### 7.3 Multiple Obligations

```ash
role developer {
    authority: [git_push],
    obligations: [tests_pass, code_reviewed, security_scanned]
}

workflow push_code {
    -- Must discharge all three obligations
    check developer.tests_pass;
    check developer.code_reviewed;
    check developer.security_scanned;
    
    act git_push;
}
```

---

## 8. Implementation Tasks

- TASK-235: SPEC-019 Role Runtime Semantics (this spec) - **Complete**
- TASK-236: Implement role runtime enforcement - Pending
  - Implement `RoleContext` with authority/obligations
  - Integrate authority check into capability invocation
  - Implement obligation discharge tracking
  - Add workflow completion checks
  - Extend spawn syntax with role assignment
  - Add audit trail events
  - Comprehensive tests

---

## 9. Relationship to Other Specifications

- **SPEC-022**: Workflow obligations (local) - Role obligations are in addition to local
- **SPEC-017**: Capability integration - Authority check integrates with capability invocation
- **SPEC-006**: Policy definitions - Authority check precedes policy evaluation
- **DECISION-237**: Will determine role-bound obligation syntax

---

## 10. Open Questions

1. **Role inheritance**: Should child workflows inherit parent role? (Section 5.3)
2. **Default roles**: Should workflow types have default roles?
3. **Dynamic role switching**: Should workflows ever change roles mid-execution? (Recommendation: No)
4. **Role composition**: Can a workflow have multiple roles? (Recommendation: No, single role per workflow)
