# SPEC-018: Capability Runtime Verification Matrix

## Status: Draft

## 1. Overview

Workflows declare their capability requirements at compile time. At runtime, they're instantiated with specific obligations and policies. This spec defines the verification matrix and runtime checks to ensure compatibility.

## 2. Capability Requirements (Compile Time)

Workflows declare what they need:

```ash
workflow controller
    observes sensor:temperature     -- Required input
    sets hvac:target               -- Required output
    oblig role:operator            -- Required obligation
{
    observe sensor:temperature as t;
    set hvac:target = calculate(t);
}
```

Compile-time requirements:
- **Input capabilities**: `observes`, `receives`
- **Output capabilities**: `sets`, `sends`
- **Effect level**: Computed from operations
- **Obligations**: Required role/context

## 3. Runtime Context

At instantiation, workflow receives:

```rust
pub struct RuntimeContext {
    pub obligations: Vec<Obligation>,     -- What we're obliged to do
    pub policies: Vec<Policy>,            -- What we're allowed to do
    pub capabilities: CapabilityRegistry, -- What's available
    pub role: Role,                       -- Who's running this
}
```

## 4. Verification Matrix

### 4.1 Capability Availability Matrix

| Workflow Requires | Runtime Provides | Result | Action |
|-------------------|------------------|--------|--------|
| `observes sensor:temp` | Registry has `sensor:temp` | ✅ OK | Proceed |
| `observes sensor:temp` | Registry missing | ❌ ERROR | `CapabilityUnavailable` |
| `sets hvac:target` | Registry has `hvac:target` + writable | ✅ OK | Proceed |
| `sets hvac:target` | Registry has `hvac:target` read-only | ❌ ERROR | `NotWritable` |
| `sets hvac:target` | Registry missing | ❌ ERROR | `CapabilityUnavailable` |
| `sends kafka:orders` | Registry has `kafka:orders` + sendable | ✅ OK | Proceed |
| `sends kafka:orders` | Registry has `kafka:orders` receive-only | ❌ ERROR | `NotSendable` |

### 4.2 Obligation Satisfaction Matrix

| Workflow Obligation | Runtime Obligation | Result | Action |
|---------------------|-------------------|--------|--------|
| `oblig role:operator` | Context has `role:operator` | ✅ OK | Proceed |
| `oblig role:operator` | Context has different role | ❌ ERROR | `RoleMismatch` |
| `oblig maintain_temperature` | Obligation assigned | ✅ OK | Track fulfillment |
| `oblig maintain_temperature` | Obligation not assigned | ⚠️ WARN | `UnfulfilledObligation` |
| No obligation required | Any obligation present | ✅ OK | Ignore extra |

### 4.3 Policy Compliance Matrix

| Workflow Operation | Policy Check | Result | Action |
|-------------------|--------------|--------|--------|
| `observe sensor:temp` | Policy: `Permit` | ✅ OK | Return value |
| `observe sensor:temp` | Policy: `Deny` | ❌ ERROR | `PolicyDenied` |
| `observe sensor:temp` | Policy: `Transform(mask)` | ✅ OK | Return masked value |
| `set hvac:target` | Policy: `Permit` | ✅ OK | Execute set |
| `set hvac:target` | Policy: `Deny` | ❌ ERROR | `PolicyDenied` |
| `set hvac:target` | Policy: `RequireApproval(r)` | ⏸️ WAIT | Queue for approval |
| `send alert:critical` | Rate limit: under | ✅ OK | Send immediately |
| `send alert:critical` | Rate limit: exceeded | ❌ ERROR | `RateLimitExceeded` |

### 4.4 Effect Compatibility Matrix

| Workflow Effect | Runtime Max Effect | Result | Action |
|-----------------|-------------------|--------|--------|
| `Epistemic` (read-only) | Any | ✅ OK | Always allowed |
| `Operational` (has output) | `Epistemic` only | ❌ ERROR | `EffectTooHigh` |
| `Operational` (has output) | `Operational` | ✅ OK | Proceed |

## 5. Runtime Verification Algorithm

```rust
pub struct CapabilityVerifier;

impl CapabilityVerifier {
    /// Verify runtime context satisfies workflow requirements
    pub fn verify(
        &self,
        workflow: &Workflow,
        runtime: &RuntimeContext,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();
        
        -- 1. Check capability availability
        result.merge(self.verify_capabilities(workflow, &runtime.capabilities)?);
        
        -- 2. Check obligation compatibility
        result.merge(self.verify_obligations(workflow, &runtime.obligations)?);
        
        -- 3. Check effect compatibility
        result.merge(self.verify_effect(workflow, runtime.max_effect)?);
        
        -- 4. Pre-check policies (static analysis)
        result.merge(self.verify_policies_static(workflow, &runtime.policies)?);
        
        Ok(result)
    }
    
    fn verify_capabilities(
        &self,
        workflow: &Workflow,
        registry: &CapabilityRegistry,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();
        
        -- Check inputs
        for (cap, channel) in &workflow.required_observes {
            match registry.get(cap, channel) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "observe",
                    capability: format!("{}:{}", cap, channel),
                }),
                Some(provider) => {
                    if !provider.is_observable() {
                        result.add_error(VerificationError::NotObservable { ... });
                    }
                }
            }
        }
        
        -- Check outputs
        for (cap, channel) in &workflow.required_sets {
            match registry.get(cap, channel) {
                None => result.add_error(VerificationError::MissingCapability { ... }),
                Some(provider) => {
                    if !provider.is_settable() {
                        result.add_error(VerificationError::NotSettable { ... });
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    fn verify_obligations(
        &self,
        workflow: &Workflow,
        runtime_obligations: &[Obligation],
    ) -> VerificationResult {
        let mut result = VerificationResult::new();
        
        -- Check required obligations are present
        for required in &workflow.required_obligations {
            if !runtime_obligations.iter().any(|o| o.satisfies(required)) {
                result.add_warning(VerificationWarning::MissingObligation {
                    required: required.clone(),
                });
            }
        }
        
        -- Check role compatibility
        if let Some(required_role) = &workflow.required_role {
            let has_role = runtime_obligations.iter()
                .any(|o| o.grants_role(required_role));
            
            if !has_role {
                result.add_error(VerificationError::RoleMismatch {
                    required: required_role.clone(),
                    available: runtime_obligations.iter()
                        .flat_map(|o| o.roles())
                        .collect(),
                });
            }
        }
        
        Ok(result)
    }
}
```

## 6. Verification Result

```rust
pub struct VerificationResult {
    pub errors: Vec<VerificationError>,
    pub warnings: Vec<VerificationWarning>,
    pub can_execute: bool,
    pub requires_approval: Vec<ApprovalRequirement>,
}

impl VerificationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn can_execute(&self) -> bool {
        self.can_execute && self.errors.is_empty()
    }
}

pub enum VerificationError {
    MissingCapability {
        operation: String,
        capability: String,
    },
    NotObservable { capability: String },
    NotSettable { capability: String },
    NotSendable { capability: String },
    RoleMismatch {
        required: Role,
        available: Vec<Role>,
    },
    EffectTooHigh {
        workflow_effect: Effect,
        max_allowed: Effect,
    },
    PolicyConflict {
        policy: String,
        reason: String,
    },
}

pub enum VerificationWarning {
    MissingObligation { required: Obligation },
    ExtraCapability { capability: String },
    PolicyRestriction { policy: String },
}
```

## 7. Per-Operation Verification

Some checks happen per-operation at runtime:

```rust
pub async fn verify_operation(
    op: &CapabilityOperation,
    runtime: &RuntimeContext,
) -> OperationResult {
    -- 1. Check capability still available
    let provider = runtime.capabilities.get(&op.capability, &op.channel)
        .ok_or(OperationError::CapabilityUnavailable)?;
    
    -- 2. Evaluate dynamic policies
    let policy_ctx = PolicyContext::from_operation(op, runtime);
    match runtime.policy_evaluator.evaluate(&policy_ctx).await? {
        Decision::Permit => {}
        Decision::Deny => return Err(OperationError::PolicyDenied),
        Decision::RequireApproval(role) => {
            return Ok(OperationResult::RequiresApproval(role));
        }
        Decision::Transform(t) => return Ok(OperationResult::Transformed(t)),
    }
    
    -- 3. Check rate limits
    if runtime.rate_limiter.would_exceed(op) {
        return Err(OperationError::RateLimitExceeded);
    }
    
    Ok(OperationResult::Proceed)
}
```

## 8. Decision Flowchart

```
Workflow Instantiation
        │
        ▼
┌─────────────────────┐
│ Verify Capabilities │
│ - All required      │
│   available?        │
└──────────┬──────────┘
           │
     ┌─────┴─────┐
     │           │
     ▼           ▼
  Missing    All Present
     │           │
     ▼           ▼
  ERROR     ┌──────────────────┐
            │ Verify Role      │
            │ - Context has    │
            │   required role? │
            └────────┬─────────┘
                     │
              ┌──────┴──────┐
              │             │
              ▼             ▼
          No Role      Has Role
              │             │
              ▼             ▼
           ERROR      ┌──────────────────┐
                     │ Verify Effect    │
                     │ - Within bounds? │
                     └────────┬─────────┘
                              │
                       ┌──────┴──────┐
                       │             │
                       ▼             ▼
                  Too High      Within Bounds
                       │             │
                       ▼             ▼
                    ERROR      ┌──────────────────┐
                               │ Static Policy    │
                               │ Check            │
                               │ - Any conflicts? │
                               └────────┬─────────┘
                                        │
                                 ┌──────┴──────┐
                                 │             │
                                 ▼             ▼
                           Conflict       No Conflict
                                 │             │
                                 ▼             ▼
                              ERROR      ┌──────────────┐
                                         │ ✅ VERIFIED  │
                                         │ Can Execute  │
                                         └──────────────┘
```

## 9. Examples

### 9.1 Successful Verification

```rust
-- Workflow
workflow controller observes sensor:temp, sets hvac:target {
    ...
}

-- Runtime Context
RuntimeContext {
    obligations: [role:operator],
    policies: [allow_temperature_control],
    capabilities: [sensor:temp (read), hvac:target (read-write)],
    role: operator,
}

-- Result: ✅ VERIFIED
VerificationResult {
    errors: [],
    warnings: [],
    can_execute: true,
}
```

### 9.2 Failed Verification - Missing Capability

```rust
-- Workflow requires set
workflow controller observes sensor:temp, sets hvac:target { ... }

-- Runtime only has read-only
RuntimeContext {
    capabilities: [sensor:temp (read), hvac:target (read-only)],
    ...
}

-- Result: ❌ ERROR
VerificationResult {
    errors: [NotSettable { capability: "hvac:target" }],
    can_execute: false,
}
```

### 9.3 Failed Verification - Role Mismatch

```rust
-- Workflow requires operator
workflow controller oblig role:operator { ... }

-- Runtime has different role
RuntimeContext {
    role: guest,
    obligations: [],
}

-- Result: ❌ ERROR
VerificationResult {
    errors: [RoleMismatch { required: operator, available: [guest] }],
    can_execute: false,
}
```

### 9.4 Warning - Missing Obligation

```rust
-- Workflow has optional obligation
workflow controller oblig maintain_temperature { ... }

-- Runtime doesn't have this obligation
RuntimeContext {
    obligations: [other_obligation],
}

-- Result: ⚠️ WARN (can still execute)
VerificationResult {
    errors: [],
    warnings: [MissingObligation { required: maintain_temperature }],
    can_execute: true,
}
```

## 10. Implementation Tasks

- TASK-114: Capability availability verifier
- TASK-115: Obligation satisfaction checker
- TASK-116: Effect compatibility checker
- TASK-117: Static policy validator
- TASK-118: Per-operation runtime verifier
- TASK-119: Verification result aggregation

## 11. Summary Table

| Check | When | Failure | Success |
|-------|------|---------|---------|
| Capabilities | Instantiation | ERROR | Continue |
| Role | Instantiation | ERROR | Continue |
| Effect | Instantiation | ERROR | Continue |
| Static Policies | Instantiation | ERROR/WARN | Continue |
| Dynamic Policies | Per-operation | ERROR/WAIT | Execute |
| Rate Limits | Per-operation | ERROR | Execute |
