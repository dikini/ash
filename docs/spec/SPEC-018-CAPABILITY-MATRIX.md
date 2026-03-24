# SPEC-018: Capability Runtime Verification Matrix

## Status: Draft

## 1. Overview

Workflows declare their capability requirements at compile time. At runtime, they're instantiated with specific obligations and named policies that have already been lowered to the canonical core policy representation. This spec defines the verification matrix and runtime checks to ensure compatibility.

## 2. Capability Requirements (Compile Time)

Workflows declare what they need:

```ash
workflow controller
    observes sensor:temperature     -- Required input
    receives kafka:orders          -- Required stream input
    sets hvac:target               -- Required output
    oblig role:operator            -- Required obligation
{
    observe sensor:temperature as t;
    receive { kafka:orders as order => act process(order) };
    set hvac:target = calculate(t);
}
```

Compile-time requirements:
- **Input capabilities**: `observes`, `receives` (for stream selectors used in `receive` arms)
- **Output capabilities**: `sets`, `sends`
- **Effect level**: Computed from operations
- **Obligations**: Required role/context

Required roles and approval roles are flat named role references. The verification contract does
not assume role hierarchy, role supervision, or inherited authority between roles.

## 3. Runtime Context

At instantiation, workflow receives:

```rust
pub struct RuntimeContext {
    pub obligations: Vec<Obligation>,     -- What we're obliged to do
    pub policy_registry: PolicyRegistry,  -- Named lowered policies available at runtime
    pub capabilities: CapabilityRegistry, -- What's available
    pub mailboxes: MailboxRegistry,       -- Declared stream mailboxes + implicit control mailbox
    pub scheduler: SourceScheduler,       -- Implements source scheduling modifiers for receive
    pub approval_queue: ApprovalQueue,    -- For RequireApproval outcomes
    pub provenance: ProvenanceSink,       -- Records warnings/decisions/effects
    pub max_effect: Effect,               -- Highest effect this runtime will permit
    pub role: Role,                       -- Who's running this
}
```

The runtime context owns verification-time responsibilities. It does not redefine workflow syntax; it supplies the resources needed to enforce the declared contract.

## 4. Verification Matrix

### 4.1 Capability Availability Matrix

| Workflow Requires | Runtime Provides | Result | Action |
|-------------------|------------------|--------|--------|
| `observes sensor:temp` | Registry has `sensor:temp` | ✅ OK | Proceed |
| `observes sensor:temp` | Registry missing | ❌ ERROR | `MissingCapability` |
| `receives kafka:orders` | Registry has `kafka:orders` + receivable | ✅ OK | Proceed |
| `receives kafka:orders` | Registry has `kafka:orders` but cannot receive | ❌ ERROR | `NotReceivable` |
| `receives kafka:orders` | Registry missing | ❌ ERROR | `MissingCapability` |
| `sets hvac:target` | Registry has `hvac:target` + writable | ✅ OK | Proceed |
| `sets hvac:target` | Registry has `hvac:target` read-only | ❌ ERROR | `NotSettable` |
| `sets hvac:target` | Registry missing | ❌ ERROR | `MissingCapability` |
| `sends kafka:orders` | Registry has `kafka:orders` + sendable | ✅ OK | Proceed |
| `sends kafka:orders` | Registry has `kafka:orders` receive-only | ❌ ERROR | `NotSendable` |

### 4.2 Obligation Satisfaction Matrix

| Workflow Obligation | Runtime Obligation | Result | Action |
|---------------------|-------------------|--------|--------|
| `oblig role:operator` | Context has `role:operator` | ✅ OK | Proceed |
| `oblig role:operator` | Context has different role | ❌ ERROR | `RoleMismatch` |
| `oblig maintain_temperature` | Obligation assigned | ✅ OK | Track fulfillment |
| `oblig maintain_temperature` | Obligation not assigned | ⚠️ WARN | `MissingObligation` |
| No obligation required | Any obligation present | ✅ OK | Ignore extra |

### 4.3 Policy Compliance Matrix

| Workflow Operation | Policy Check | Result | Action |
|-------------------|--------------|--------|--------|
| `observe sensor:temp` | Policy: `Permit` | ✅ OK | Return value |
| `observe sensor:temp` | Policy: `Deny` | ❌ ERROR | `PolicyDenied` |
| `observe sensor:temp` | Policy: `Transform(mask)` | ✅ OK | Return masked value |
| `receive { kafka:orders as order => ... }` | Policy: `Permit` | ✅ OK | Select message and run matched arm |
| `receive { kafka:orders as order => ... }` | Policy: `Deny` | ❌ ERROR | `PolicyDenied` |
| `receive control { "shutdown" => ... }` | Control mailbox message present | ✅ OK | Consume control message and run matched arm |
| `set hvac:target` | Policy: `Permit` | ✅ OK | Execute set |
| `set hvac:target` | Policy: `Deny` | ❌ ERROR | `PolicyDenied` |
| `set hvac:target` | Policy: `RequireApproval(r)` | ⏸️ WAIT | Queue for approval |
| `send alert:critical` | Rate limit: under | ✅ OK | Send immediately |
| `send alert:critical` | Rate limit: exceeded | ❌ ERROR | `RateLimitExceeded` |

### 4.4 Effect Compatibility Matrix

| Workflow Effect | Runtime Max Effect | Result | Action |
|-----------------|-------------------|--------|--------|
| `Epistemic` (input acquisition and read-only observation) | `Epistemic` or higher | ✅ OK | Proceed |
| `Deliberative` (analysis, planning, proposal formation) | `Deliberative` or higher | ✅ OK | Proceed |
| `Evaluative` (policy and obligation evaluation) | `Evaluative` or higher | ✅ OK | Proceed |
| `Operational` (external side effects and irreversible outputs) | `Operational` | ✅ OK | Proceed |
| Any workflow effect | Lower than required by the workflow | ❌ ERROR | `EffectTooHigh` |

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
        
        -- 4. Pre-check policy availability / compatibility
        result.merge(self.verify_policies_static(workflow, &runtime.policy_registry)?);
        
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

        for (cap, channel) in &workflow.required_receives {
            match registry.get(cap, channel) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "receive",
                    capability: format!("{}:{}", cap, channel),
                }),
                Some(provider) => {
                    if !provider.is_receivable() {
                        result.add_error(VerificationError::NotReceivable { ... });
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
    pub transforms: Vec<PlannedTransformation>,
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
    NotReceivable { capability: String },
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
        .ok_or(OperationError::MissingCapability)?;
    
    -- 2. Evaluate normalized policies
    let policy_ctx = PolicyContext::from_operation(op, runtime);
    match runtime.policy_evaluator.evaluate(&policy_ctx).await? {
        PolicyDecision::Permit => {}
        PolicyDecision::Deny => return Err(OperationError::PolicyDenied),
        PolicyDecision::RequireApproval(role) => {
            return Ok(OperationResult::RequiresApproval(role));
        }
        PolicyDecision::Transform(t) => return Ok(OperationResult::Transformed(t)),
    }
    
    -- 3. Check rate limits
    if runtime.rate_limiter.would_exceed(op) {
        return Err(OperationError::RateLimitExceeded);
    }
    
    Ok(OperationResult::Proceed)
}
```

Runtime verification consumes only normalized `PolicyDecision` outcomes produced from named lowered policies. It does not operate on source-level policy expressions directly.

Canonical policy decision taxonomy:

- `Permit`: the policy allows execution to continue unchanged.
- `Deny`: hard policy failure; operation stops.
- `RequireApproval`: operation is suspended or queued pending approval.
- `Transform`: operation continues with a transformed input or output value.

Aggregate verification reports `Proceed` when all required checks succeed. `Warn` is not a policy
decision; it is verification metadata used by aggregate verification or provenance and does not
alter the canonical policy decision taxonomy.

Capability-verification consumers operate over this verification decision set. If a specific
capability or provider cannot honor an approval or transformation outcome before execution, the
operation is rejected as a verification incompatibility rather than introducing a new policy
decision kind.

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
    policy_registry: [allow_temperature_control],
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
