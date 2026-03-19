# SPEC-017: Capability Integration with System Features

## Status: Draft

## 1. Overview

All capabilities (input: `observe`, `receive`; output: `set`, `send`) must integrate with Ash's system features: obligations, policies, effects, provenance, and capability safety.

## 2. Effects System

### 2.1 Effect Classification

| Capability | Operation | Effect | Rationale |
|------------|-----------|--------|-----------|
| Behaviour | `observe` | Epistemic | Input acquisition and read-only observation |
| Behaviour | `set` | Operational | External side effect that modifies external state |
| Stream | `receive` | Epistemic | Mailbox input acquisition; consumes queued workflow input without producing external side effects |
| Stream | `send` | Operational | External side effect that produces an outgoing event |

### 2.2 Effect Lattice

```
Epistemic < Deliberative < Evaluative < Operational

Epistemic = input acquisition and read-only observation
Deliberative = analysis, planning, and proposal formation
Evaluative = policy and obligation evaluation
Operational = external side effects and irreversible outputs

observe ──────┐
receive ──────┼──► Epistemic
              │
set ──────────┤
send ─────────┼──► Operational
act ──────────┘
```

### 2.3 Workflow Effect Inference

```ash
workflow reader observes sensor:temp {
    -- Effect: Epistemic
}

workflow writer sets actuator:position {
    -- Effect: Operational
}

workflow mixed 
    observes sensor:temp 
    receives events:alerts 
    sets actuator:position 
{
    -- Effect: Operational (join of all)
}
```

### 2.4 Effect Tracking in Types

```rust
pub struct WorkflowType {
    pub input: Effect,   -- Max effect of inputs
    pub output: Effect,  -- Max effect of outputs
    pub total: Effect,   -- Join of input and output
}

-- Input-only workflow: Epistemic
-- Analysis-only workflow: Deliberative
-- Policy/obligation gate: Evaluative
-- Workflow with external outputs: Operational
```

## 3. Obligations Integration

### 3.1 Obligations Can Require Capabilities

```ash
-- Obligation that requires observation
role monitor {
    obligations: [check_temperature]
}

workflow check_temperature observes sensor:temp {
    observe sensor:temp as t;
    if t > 100 then
        act alert::trigger("overheat")
}

-- Obligation that requires output
role controller {
    obligations: [maintain_temperature]
}

workflow maintain_temperature 
    observes sensor:temp 
    sets hvac:target 
{
    observe sensor:temp as t;
    if t > 75 then
        set hvac:target = 72
    else if t < 65 then
        set hvac:target = 68
}
```

### 3.2 Obligation Checking

```rust
pub struct ObligationChecker;

impl ObligationChecker {
    pub fn verify_capabilities(
        &self,
        obligation: &Obligation,
        workflow: &Workflow,
    ) -> Result<(), ObligationError> {
        -- Check workflow has required capabilities
        for required in &obligation.required_capabilities {
            if !workflow.has_capability(required) {
                return Err(ObligationError::MissingCapability {
                    obligation: obligation.name.clone(),
                    capability: required.clone(),
                });
            }
        }
        
        -- Check effect level is sufficient
        if workflow.effect() < obligation.min_effect {
            return Err(ObligationError::InsufficientEffect {
                required: obligation.min_effect,
                actual: workflow.effect(),
            });
        }
        
        Ok(())
    }
}
```

### 3.3 Obligation Violations

```rust
pub enum ObligationViolation {
    InputNotAvailable {
        obligation: Name,
        capability: Name,
        reason: String,
    },
    OutputFailed {
        obligation: Name,
        operation: OutputOperation,
        error: ExecError,
    },
    EffectMismatch {
        obligation: Name,
        required: Effect,
        actual: Effect,
    },
}
```

## 4. Policies Integration

### 4.1 Policy-Controlled Capabilities

Policies can restrict both input and output:

```ash
-- Input policy: Who can observe sensitive data
policy sensitive_data_access:
    when observe sensor:security_camera
    and role not in [security, admin]
    then deny

-- Output policy: Who can control critical systems  
policy hvac_control:
    when set hvac:target
    and role != facilities_manager
    and time:outside_business_hours
    then require_approval(role:admin)

-- Rate limiting policy
policy alert_rate_limit:
    when send notification:alerts
    and count_last_5min > 10
    then deny

-- Data residency policy
policy data_residency:
    when observe database:customer_data
    and region != "EU"
    then mask(fields: ["personal_id", "address"])
```

### 4.2 Policy Evaluation Context

```rust
pub struct CapabilityContext {
    pub operation: CapabilityOperation,
    pub direction: Direction,  -- Input or Output
    pub capability: Name,
    pub channel: Name,
    pub value: Option<Value>,  -- For output: value being set/sent
    pub constraints: Vec<Constraint>, -- For input: query constraints
    pub actor: Role,
    pub timestamp: DateTime<Utc>,
    pub location: Option<Region>,
}

pub enum Direction {
    Input,   -- observe, receive
    Output,  -- set, send
}

pub enum CapabilityOperation {
    Observe,
    Receive,
    Set,
    Send,
}
```

### 4.3 Pre-Operation Policy Check

```rust
pub async fn check_policy(
    ctx: &CapabilityContext,
    policy_eval: &PolicyEvaluator,
) -> PolicyResult {
    let decision = policy_eval.evaluate_capability(ctx)?;
    
    match decision {
        Decision::Permit => Ok(()),
        
        Decision::Deny => Err(ExecError::PolicyDenied {
            operation: format!("{:?}", ctx.operation),
            capability: ctx.capability.clone(),
        }),
        
        Decision::RequireApproval { role } => Err(ExecError::RequiresApproval {
            role,
            operation: format!("{:?}", ctx.operation),
        }),
        
        Decision::Transform { transformation } => {
            -- Apply transformation (e.g., masking)
            Ok(transformation)
        }
    }
}
```

### 4.4 Input Transformations

Policies can transform input data:

```rust
pub enum Transformation {
    Permit,           -- No change
    Mask { fields: Vec<Name> },  -- Mask sensitive fields
    Filter,           -- Return empty/null
    Replace { value: Value },  -- Return different value
}

-- Example: Data masking
when observe database:users
and role != admin
then mask(fields: ["ssn", "salary"])
```

### 4.5 Provider-Level Policies

```rust
pub trait PolicyAwareProvider {
    -- Policies specific to this provider
    fn local_policies(&self) -> Vec<Policy>;
    
    -- Check if operation would violate policies
    fn check_policy(&self, ctx: &CapabilityContext) -> PolicyResult;
}

impl PolicyAwareProvider for DatabaseProvider {
    fn local_policies(&self) -> Vec<Policy> {
        vec![
            Policy::RateLimit(100),  -- Max 100 queries/minute
            Policy::MaxResultSize(10000),  -- Max 10k rows
            Policy::RequireAuth,
        ]
    }
}
```

## 5. Provenance Integration

### 5.1 Provenance Events for All Capabilities

```rust
pub struct ProvenanceEvent {
    pub event_type: CapabilityEventType,
    pub direction: Direction,
    pub capability: Name,
    pub channel: Name,
    pub value: Option<Value>,  -- None for sensitive data
    pub constraints: Option<Vec<Constraint>>,
    pub workflow_id: WorkflowId,
    pub run_id: RunId,
    pub timestamp: DateTime<Utc>,
    pub effect: Effect,
    pub policy_decisions: Vec<PolicyDecision>,
}

pub enum CapabilityEventType {
    Observed,
    Received,
    Set,
    Sent,
}
```

### 5.2 Input Provenance

```
[2024-01-15T10:30:00Z] workflow:analyzer
  OBSERVE sensor:temperature
  Constraints: { location: "room_a" }
  Result: { value: 22.5, unit: "celsius" }
  Policy: Permit
  Effect: Epistemic
  
[2024-01-15T10:30:01Z] workflow:analyzer
  RECEIVE kafka:events
  Value: { type: "sensor_update", ... }
  Policy: Permit
  Effect: Epistemic
```

### 5.3 Output Provenance

```
[2024-01-15T10:31:00Z] workflow:controller
  DECIDE policy:hvac_control = Permit
  
[2024-01-15T10:31:00Z] workflow:controller
  SET hvac:target
  Value: 21
  Policy: Permit
  Effect: Operational
  
[2024-01-15T10:31:05Z] workflow:controller
  SEND notification:alerts
  Value: { level: "info", message: "Temperature adjusted" }
  Policy: Permit
  Effect: Operational
```

### 5.4 Provenance-Aware Execution

```rust
pub async fn execute_capability(
    op: CapabilityOperation,
    ctx: &CapabilityContext,
    provenance: &mut ProvenanceTracker,
    policy_eval: &PolicyEvaluator,
    provider: &dyn CapabilityProvider,
) -> ExecResult<Value> {
    -- 1. Record intent
    provenance.record(Intent { ... });
    
    -- 2. Check policy
    let policy_result = check_policy(ctx, policy_eval).await?;
    
    -- 3. Execute
    let result = match op {
        CapabilityOperation::Observe => provider.sample(...).await,
        CapabilityOperation::Set => provider.set(...).await.map(|_| Value::Null),
        -- ...
    };
    
    -- 4. Record outcome
    provenance.record(ProvenanceEvent {
        event_type: op.into(),
        direction: ctx.direction,
        policy_decisions: vec![policy_result],
        result: result.clone(),
        ...
    });
    
    result
}
```

## 6. Capability Safety

### 6.1 Declaration Requirements

All capabilities must be declared:

```ash
-- Input declarations
workflow reader 
    observes sensor:temperature      -- Behaviour input
    receives kafka:events           -- Stream input
{
    observe sensor:temperature as t;  -- OK
    receive { kafka:events as e => done }; -- OK
    observe sensor:pressure as p;     -- ERROR: not declared
}

-- Output declarations
workflow writer 
    sets hvac:target                -- Behaviour output
    sends notification:alerts       -- Stream output
{
    set hvac:target = 72;            -- OK
    send notification:alerts "...";  -- OK
    set config:mode = "auto";        -- ERROR: not declared
}

-- Mixed
workflow mixed
    observes sensor:temperature
    receives kafka:commands
    sets hvac:target
    sends kafka:status
{
    -- Can use all four
}
```

`receive` statements use the canonical arm-based surface form from SPEC-002. The declaration authorizes the `capability:channel` selectors named in those arms.

### 6.2 Capability Registry

```rust
pub struct WorkflowCapabilities {
    pub observes: Vec<CapabilityRef>,
    pub receives: Vec<CapabilityRef>,
    pub sets: Vec<CapabilityRef>,
    pub sends: Vec<CapabilityRef>,
}

impl WorkflowCapabilities {
    pub fn can_observe(&self, cap: &str, channel: &str) -> bool {
        self.observes.iter().any(|c| c.matches(cap, channel))
    }
    
    pub fn can_receive(&self, cap: &str, channel: &str) -> bool {
        self.receives.iter().any(|c| c.matches(cap, channel))
    }
    
    pub fn can_set(&self, cap: &str, channel: &str) -> bool {
        self.sets.iter().any(|c| c.matches(cap, channel))
    }
    
    pub fn can_send(&self, cap: &str, channel: &str) -> bool {
        self.sends.iter().any(|c| c.matches(cap, channel))
    }
}
```

### 6.3 Compile-Time Verification

```rust
pub fn verify_capabilities(workflow: &Workflow) -> Result<(), CapabilityError> {
    for op in workflow.operations() {
        let allowed = match &op {
            Operation::Observe { cap, channel } => 
                workflow.capabilities.can_observe(cap, channel),
            Operation::Receive { cap, channel } => 
                workflow.capabilities.can_receive(cap, channel),
            Operation::Set { cap, channel } => 
                workflow.capabilities.can_set(cap, channel),
            Operation::Send { cap, channel } => 
                workflow.capabilities.can_send(cap, channel),
            _ => true,
        };
        
        if !allowed {
            return Err(CapabilityError::Undeclared {
                operation: op.name(),
                capability: op.capability(),
            });
        }
    }
    Ok(())
}
```

## 7. Type Safety Integration

### 7.1 Read/Write Type Schemas

```rust
pub struct CapabilitySchema {
    pub read: Option<Type>,   -- Type returned by observe/receive
    pub write: Option<Type>,  -- Type accepted by set/send
}

impl CapabilitySchema {
    pub fn validate_input(&self, value: &Value) -> Result<(), TypeError> {
        match &self.read {
            Some(schema) if schema.matches(value) => Ok(()),
            Some(schema) => Err(TypeError::InputMismatch { ... }),
            None => Err(TypeError::NotReadable),
        }
    }
    
    pub fn validate_output(&self, value: &Value) -> Result<(), TypeError> {
        match &self.write {
            Some(schema) if schema.matches(value) => Ok(()),
            Some(schema) => Err(TypeError::OutputMismatch { ... }),
            None => Err(TypeError::NotSettable),
        }
    }
}
```

### 7.2 Type Checking Example

```ash
-- Schema: sensor:temperature reads Int, writes nothing
workflow bad sets sensor:temperature {
    set sensor:temperature = 72  -- ERROR: not settable
}

-- Schema: hvac:target reads Int, writes Int
workflow good observes sensor:temp, sets hvac:target {
    observe sensor:temp as t;     -- t: Int
    set hvac:target = t + 5;      -- OK: Int = Int + Int
}

-- Type mismatch
workflow bad_type observes sensor:temp, sets config:mode {
    observe sensor:temp as t;     -- t: Int
    set config:mode = t;          -- ERROR: config:mode expects String
}
```

## 8. Error Handling

### 8.1 Unified Capability Errors

```rust
pub enum CapabilityError {
    -- Declaration errors
    NotDeclared { operation: String, capability: Name },
    
    -- Input errors
    InputUnavailable { capability: Name, reason: String },
    InputTimeout { capability: Name, duration: Duration },
    InputTransformFailed { capability: Name, error: String },
    
    -- Output errors
    OutputFailed { capability: Name, error: String },
    OutputRejected { capability: Name, reason: String },
    BufferFull { capability: Name },
    Disconnected { capability: Name },
    
    -- Policy errors
    PolicyDenied { policy: Name, operation: String },
    RequiresApproval { role: Role },
    
    -- Type errors
    TypeMismatch { expected: Type, actual: Type },
    NotReadable,
    NotSettable,
}
```

### 8.2 Error Recovery

```ash
workflow resilient 
    observes sensor:temperature
    sets hvac:target 
{
    attempt {
        observe sensor:temperature as t
    } retry 3 on InputTimeout {
        sleep(100ms)
    } on error {
        act log::error("sensor unavailable");
        send alert:critical "Sensor failure";
        use default:emergency_shutdown
    };
    
    attempt {
        set hvac:target = calculate_target(t)
    } retry 3 on Disconnected {
        sleep(500ms)
    } on BufferFull {
        act log::warn("HVAC buffer full, dropping command");
        continue
    } on error {
        act log::error("HVAC control failed");
        send alert:critical "HVAC failure"
    }
}
```

## 9. Summary Table

| Feature | Input (observe/receive) | Output (set/send) |
|---------|------------------------|-------------------|
| **Effect** | Epistemic | Operational |
| **Obligations** | Can require observation | Can require output |
| **Policies** | Can restrict/transform | Can restrict/approve |
| **Provenance** | Record observed data or message intake | Record value sent |
| **Declaration** | `observes`, `receives` | `sets`, `sends` |
| **Type Safety** | Read schema validated | Write schema validated |
| **Errors** | Unavailable, Timeout | Failed, Rejected, BufferFull |

## 10. Implementation Tasks

- TASK-108: Effect tracking for all capabilities
- TASK-109: Obligation checking with capabilities
- TASK-110: Policy evaluation for input/output
- TASK-111: Provenance tracking for all capabilities
- TASK-112: Capability declaration verification
- TASK-113: Read/write type checking
