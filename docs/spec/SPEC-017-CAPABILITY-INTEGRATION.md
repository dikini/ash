# SPEC-017: Capability Integration with System Features

## Status: Active (Section 2 Capability Definitions, Section 11 IO Capability Boundary - V1 Frozen)

## 1. Overview

All capabilities (input: `observe`, `receive`; output: `set`, `send`) must integrate with Ash's system features: obligations, policies, effects, provenance, and capability safety.

This specification distinguishes declaration-site capability definitions from usage-site
capability types. Declaration sites use `capability`; usage sites may use `cap <Identifier>`
where a workflow boundary expects a capability witness.

Terminology in this spec follows `docs/reference/type-system-vocabulary-guidance.md`: use
"capability declaration" for the source contract, "capability identity" for the resolved invoked
name, "capability witness" for usage-site `cap C` values at workflow boundaries, and "provider"
for the runtime implementation.

## 1.1 Split Dispatch Overview

This specification adopts the split dispatch model where provider lookup and provider-local action
dispatch are explicit and separate:

**Runtime Dispatch Contract:**
```text
lookup provider by provider_name -> provider
provider.execute(action_name, args) -> result
```

This replaces the previous overloaded model where one name was used for both lookup and dispatch.
The canonical ACT representation carries:
- `provider_name`: Used for registry lookup
- `action_name`: Used for provider-local dispatch
- `arguments`: Evaluated and passed to the provider

**Symbolic Capability Resolution:**

Symbolic capability names in surface syntax resolve to `(provider, action)` pairs:

```ash
fs_read("file.txt")           -- Simple symbolic: resolves from capability declarations
io::fs_read("file.txt")       -- Module-qualified: resolves through module/import paths
io:fs_read("file.txt")        -- Explicit: (provider: "io", action: "fs_read")
```

The resolver produces a `ResolvedCapabilityTarget { provider, action }` which lowers
to the canonical `Act { provider_name, action_name, ... }` IR form.

**Resolution Sources:**

Symbolic operational capability names resolve from module-owned metadata:

1. **Local capability declarations** - `capability fs_read : act (...)` declares a visible symbol
2. **Imported capabilities** - `use io::fs_read` brings a symbol into local scope  
3. **Re-exports** - `pub use io::fs_read` re-exports with optional aliasing
4. **Module-qualified paths** - `io::fs_read` resolves through the module graph

**Explicit Form:**

Explicit `provider:action(...)` syntax bypasses symbolic resolution and provides the
target pair directly. This form is useful when:
- The capability has no symbolic declaration
- The caller wants to bypass import resolution
- Calling a provider action directly without capability metadata

**Compile-Time Resolution:**

Symbolic capability names must resolve at compile time. Unresolved symbolic names are
compile-time errors, not runtime failures. The resolution context is built once from
the module/import graph and shared across:
- Lowering (surface → core AST)
- Type checking (capability witness validation)
- Capability checking (declaration vs usage alignment)

**Implementation Status:** Phase 70 implemented a bridge resolver with built-in mappings
as a transitional step. Phase 71 moves to module-owned resolution where capability
declarations and imports are the authoritative source of symbolic operational metadata.

## 2. Capability Definitions

Capabilities can be defined in `.ash` source files using the `capability` keyword. This allows user-defined capabilities without requiring pre-registration in Rust.

### 2.1 Syntax

```bnf
capability_def  ::= "capability" IDENTIFIER ":" effect_type
                    "(" param_list? ")"
                    ("returns" type)?
                    constraint_list?
                    visibility?

effect_type     ::= "observe" | "read" | "analyze" | "decide" 
                  | "act" | "write" | "external"
                  | "epistemic" | "deliberative" | "evaluative" | "operational"

param_list      ::= param ("," param)*
param           ::= IDENTIFIER ":" param_type

param_type      ::= type | capability_type
capability_type ::= "cap" IDENTIFIER

constraint_list ::= "where" constraint ("," constraint)*
constraint      ::= expression

visibility      ::= "pub" | "private"  -- default: private
```

`cap` is a usage-site type-forming keyword. `cap C` denotes a parameter or other type position
that expects a capability witness for the capability named `C`. It is not a second capability
declaration form.

### 2.2 Examples

Minimal capability (no parameters, no constraints):

```ash
capability get_timestamp : observe ()
```

Capability with parameters:

```ash
capability read_file : read (path : String)
```

Capability with return type:

```ash
capability fetch_user : observe (id : Int) returns User
```

Capability with constraints:

```ash
capability transfer_funds : act (amount : Int, to : Account)
    where amount > 0 and balance >= amount
```

Public capability (exported from module):

```ash
pub capability system_restart : act ()
    where role == admin
```

Runtime-provided capability declaration and use:

```ash
pub capability Args : observe (index : Int) returns Option<String>

workflow main(args: cap Args) -> Result<(), RuntimeError> {
    let argv0 = observe Args 0;
    done;
}
```

Here `cap Args` is the parameter type. The capability itself is still declared with
`capability Args ...`. Capability invocation remains effect-first and explicit; dotted or
method-like forms such as `args.get(0)` and `observe Args.get(0)` are not normative in SPEC-017.

### 2.3 Semantic Requirements

**Effect Type Mapping:**

| Surface Syntax | Core Effect |
|----------------|-------------|
| `observe`, `read`, `epistemic` | `Effect::Epistemic` |
| `analyze`, `deliberative` | `Effect::Deliberative` |
| `decide`, `evaluative` | `Effect::Evaluative` |
| `act`, `write`, `external`, `operational` | `Effect::Operational` |

**Constraint Evaluation:**

- Constraints are evaluated at capability invocation time
- All constraints must evaluate to `true` for invocation to proceed
- Constraint failure results in `ValidationError`
- Constraints may reference capability parameters

**Visibility:**

- `pub` capabilities are exported and can be imported by other modules
- `private` (default) capabilities are module-local
- Visibility is checked during module linking

**Usage-Site Capability Types:**

- `cap C` is well-formed only if `C` resolves to a declared or imported capability identity
- `cap C` describes a required capability witness at a workflow boundary; it does not declare `C`
- The parameter binder for `p: cap C` is an authorization witness for the provided capability, not a method-dispatch receiver; explicit invocation continues to name the capability identity `C`
- A workflow boundary must not declare multiple parameters of the same capability type `cap C`

**Runtime-Provided Capability Values:**

- The runtime may satisfy `cap`-typed parameters at workflow entry or at another runtime-defined boundary only when that boundary explicitly specifies the same authorization contract
- User code does not construct these values directly in this specification

**Invocation Syntax:**

- Capability invocation remains effect-first and explicit
- Read-like runtime capabilities such as `Args` use `observe`, so `observe Args 0` is canonical
- This specification does not introduce ordinary member-call or method-dispatch semantics for capabilities

### 2.4 Core AST Representation

```rust
pub struct CapabilityDef {
    pub name: Name,
    pub effect: Effect,
    pub params: Vec<Param>,
    pub returns: Option<TypeExpr>,
    pub constraints: Vec<Constraint>,
    pub visibility: Visibility,
    pub span: Span,
}

pub struct Param {
    pub name: Name,
    pub ty: TypeExpr,
}
```

### 2.5 Integration Points

**Module System:**

- Capability definitions are module items
- Public capabilities are exported in module interface
- Imported capabilities are resolved during lowering

**Type Checker:**

- Capability parameters are type-checked at definition
- Capability return type is validated at invocation
- Constraints are type-checked as boolean expressions
- Usage-site forms such as `cap Args` resolve `Args` as a capability identity, not as a nominal data type

**Runtime:**

- Capability definitions populate `CapabilityRegistry`
- Runtime looks up capabilities by name
- Constraints are evaluated before capability execution
- Runtime may inject capability witnesses that satisfy `cap`-typed parameters at workflow entry or at another runtime-defined boundary only when that boundary explicitly specifies the same authorization contract

## 3. Effects System

### 3.1 Effect Classification

Current promoted contract: effect classification is computed from Ash workflow forms and
source-level contracts first. Provider metadata may be used for compatibility/validation, but it is
not the primary source of source-level effect typing.

| Capability | Operation | Effect | Rationale |
|------------|-----------|--------|-----------|
| Behaviour | `observe` | Epistemic | Input acquisition and read-only observation |
| Behaviour | `set` | Operational | External side effect that modifies external state |
| Stream | `receive` | Epistemic | Mailbox input acquisition; consumes queued workflow input without producing external side effects |
| Stream | `send` | Operational | External side effect that produces an outgoing event |
| Monitor view | `observe` via `MonitorLink` | Epistemic | Read-only observation of an exposed workflow view |

Runtime-provided capabilities obey the same effect mapping. For example, when `Args` is declared
as an observe-like capability, `observe Args 0` is an epistemic operation.

This section intentionally stays within the current four surfaced coarse grades. Whether Ash should
surface an explicit `Pure` bottom element for control-only forms remains follow-up work rather than
current normative text here.

### 3.2 Effect Lattice

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

### 3.3 Workflow Effect Inference

The current coarse workflow-form mapping is:

| Workflow form | Current coarse effect classification |
|---|---|
| `observe` | `Epistemic` |
| `receive` | `Epistemic` |
| `orient` / `propose` | `Deliberative` |
| `decide` / `check` | `Evaluative` |
| `set` / `send` / `act` | `Operational` |
| control and modal forms | compose by join over enclosed effects; no extra surfaced grade of their own |

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

### 3.4 Effect Tracking in Types

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

## 4. Obligations Integration

### 4.1 Obligations Can Require Capabilities

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

These examples rely only on the canonical role contract: roles contribute authority and
obligations, but no role hierarchy or supervision semantics are assumed here.

### 4.2 Obligation Checking

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

### 4.3 Obligation Violations

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

## 5. Constraint Refinement

Capabilities can be refined with constraints at declaration site:

```ash
capability file {
    effect: Operational,
    permissions: { read: bool, write: bool }
}

-- Usage with constraints
workflow processor capabilities: [
    file @ { 
        paths: ["/var/log/*", "/tmp/llm-*"],
        read: true,
        write: false
    }
]
```

### 5.1 Constraint Semantics

Constraints narrow the capability grant:

- **`paths`**: Limits accessible file paths to the specified glob patterns
- **`permissions`**: Narrows available operations (e.g., `read: true, write: false` grants read-only access)
- **Additional fields**: Depend on the specific capability declaration/schema being described

Constraints are evaluated as a conjunction (all must be satisfied). If any constraint evaluates to false, the capability invocation is denied.

### 5.2 Constraint Checking

Constraints are checked at two phases:

**Static checking (compile time):**

- Constraint field names are validated against capability definition
- Field value types are checked against capability schema
- Constant expressions are evaluated where possible

**Dynamic checking (runtime):**

- Variable constraint values are evaluated at invocation time
- Path patterns are matched against actual access requests
- Permission flags are checked before operations execute

Constraint failures at runtime result in `ValidationError` with details about which constraint was violated.

### 5.3 Relationship to Roles

Roles bundle capabilities with constraints, providing a reusable authorization pattern:

```ash
role log_processor {
    capabilities: [
        file @ { paths: ["/var/log/*"], read: true }
    ]
}

workflow analyzer plays role(log_processor) {
    -- Can only read files matching /var/log/*
    act file.read with { path: "/var/log/app.log" };
    done;
}
```

Workflows can also declare capabilities directly with constraints:

```ash
workflow processor capabilities: [
    file @ { paths: ["/tmp/worker-*"], read: true, write: true }
] {
    -- Has constrained access without explicit role
    done;
}
```

See SPEC-024 for full syntax specification.

## 6. Policies Integration

### 6.1 Policy-Controlled Capabilities

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

These declarations are policy definitions and named policy bindings. Before capability
verification runs, each named binding lowers into the shared `CorePolicy` representation from
SPEC-006 and SPEC-007.
The lowered representation is consumer-neutral, but the terminal decisions are not:

- workflow `decide` sites admit only `Permit` / `Deny`,
- capability-verification sites operate over the verification decision set
  `{Permit, Deny, RequireApproval, Transform}`,
- `Warn` is not a policy decision; it is verification metadata recorded separately in aggregate
  verification or provenance.

Capability-verification consumers may still reject unsupported approval or transformation
outcomes before execution if a particular capability/provider cannot honor that decision class.
That rejection is a verification incompatibility, not a new policy decision kind.

`RequireApproval { role }` names the approval role directly. It does not derive approval authority
from role supervision or inherited hierarchy semantics.

Monitoring authority is a distinct read-only capability. Workflows may expose a monitor view via
`exposes { ... }`; that view can include obligations, behaviours, and values such as
`monitor_count`. Ordinary policy conditions may inspect those exposed values to govern monitor
grant, delegation, or observation decisions. No monitor-specific policy sublanguage is introduced,
and monitor grant/delegation is treated as an explicit atomic capability operation rather than a
two-step read-then-write protocol. `MonitorLink` sharing is non-consuming by default and is
distinct from control transfer, so policy can constrain the visible monitor count without making
monitor authority linear or affine.

### 6.2 Policy Evaluation Context

```rust
pub struct CapabilityContext {
    pub operation: CapabilityOperation,
    pub direction: Direction,  -- Input or Output
    pub provider_name: Name,   -- Provider for lookup
    pub action_name: Name,     -- Provider-local action for dispatch
    pub capability: Name,      -- Legacy: symbolic capability name
    pub channel: Name,
    pub mode: Option<ReceiveMode>,
    pub is_control_stream: bool,
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
    Act,     -- Explicit operational action
}
```

**Split Dispatch Contract:**

Runtime capability execution follows an explicit two-phase dispatch:

1. **Provider Lookup**: `registry.get(provider_name) -> provider`
   - Maps the provider name to a provider implementation
   - Lookup failure results in `RuntimeFailure`

2. **Action Dispatch**: `provider.execute(action_name, args) -> result`
   - The resolved provider handles the action dispatch locally
   - The provider receives only the action name and evaluated argument values

This split removes the previous overload where one name was used for both provider
lookup and provider-local action dispatch.

The runtime context is responsible for:

- provider lookup via `provider_name`,
- action dispatch via `action_name` to the resolved provider,
- mailbox access for declared stream selectors and the implicit control mailbox,
- scheduler execution for the active source scheduling modifier,
- policy evaluation,
- approval routing,
- transformation application,
- provenance capture,
- effect-ceiling enforcement.

### 6.3 Pre-Operation Policy Check

```rust
pub async fn check_policy(
    ctx: &CapabilityContext,
    policy_eval: &PolicyEvaluator,
) -> Result<PolicyDecision, ExecError> {
    let decision = policy_eval.evaluate_capability(ctx)?;
    
    match decision {
        PolicyDecision::Permit => Ok(PolicyDecision::Permit),
        
        PolicyDecision::Deny => Err(ExecError::PolicyDenied {
            operation: format!("{:?}", ctx.operation),
            capability: ctx.capability.clone(),
        }),
        
        PolicyDecision::RequireApproval { role } => Err(ExecError::RequiresApproval {
            role,
            operation: format!("{:?}", ctx.operation),
        }),
        
        PolicyDecision::Transform { transformation } => {
            -- Apply transformation (e.g., masking)
            Ok(PolicyDecision::Transform { transformation })
        }
    }
}
```

### 6.4 Input Transformations

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

Capability-verification policy decisions are canonical:

- `Permit` means the operation may continue unchanged.
- `Deny` is a hard error and the operation does not execute.
- `RequireApproval` pauses or queues the operation for approval handling.
- `Transform` rewrites the observable or transferable value and then execution continues.

Verification warnings are separate from policy decisions:

- `Warn` is advisory only and is recorded in aggregate verification or provenance without blocking
  execution.

### 6.5 Provider-Level Policies

```rust
pub trait PolicyAwareProvider {
    -- Named policies specific to this provider
    fn local_policies(&self) -> Vec<PolicyName>;
    
    -- Check if operation would violate policies
    fn check_policy(&self, ctx: &CapabilityContext) -> Result<PolicyDecision, ExecError>;
}

impl PolicyAwareProvider for DatabaseProvider {
    fn local_policies(&self) -> Vec<PolicyName> {
        vec![
            PolicyName::new("database_rate_limit"),
            PolicyName::new("database_max_result_size"),
            PolicyName::new("database_require_auth"),
        ]
    }
}
```

## 7. Provenance Integration

### 7.1 Provenance Events for All Capabilities

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

### 7.2 Input Provenance

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

### 7.3 Output Provenance

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

### 7.4 Provenance-Aware Execution

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

## 8. Capability Safety

### 8.1 Declaration Requirements

All capabilities must be declared:

```ash
-- Input declarations
workflow reader 
    observes sensor:temperature     -- Behaviour input
    receives kafka:events           -- Stream input
{
    observe sensor:temperature as t;  -- OK
    receive { kafka:events as e => done }; -- OK
    observe sensor:pressure as p;     -- ERROR: not declared
}

-- Output declarations
workflow writer 
    sets hvac:target                 -- Behaviour output
    sends notification:alerts        -- Stream output
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
The runtime scheduler and source scheduling modifier defined in SPEC-013 determine which declared
stream mailbox is probed next; this spec only states that the declared selector is authorized.

Declaration requirements are canonical:

- `observe cap:channel` requires `observes cap:channel`
- `receive { cap:channel as ... => ... }` requires `receives cap:channel`
- `receive control { ... }` requires no explicit declaration because the control mailbox is implicit
- `set cap:channel = ...` requires `sets cap:channel`
- `send cap:channel ...` requires `sends cap:channel`

Runtime-provided capability parameters are an additional authorization source only where a
runtime-defined boundary explicitly says so. At workflow entry, if the boundary requires a
parameter of type `cap C`, the corresponding runtime-provided capability `C` is authorized for the
explicit effect-form operations defined for that capability without requiring a separate
`observes` / `receives` / `sets` / `sends` header clause. This does not create a general exemption
for arbitrary workflows or non-entry bindings; any future boundary with the same behavior must
specify that authorization contract explicitly.

### 8.2 Capability Registry

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

### 8.3 Compile-Time Verification

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

## 9. Type Safety Integration

### 9.1 Read/Write Type Schemas

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

### 9.2 Type Checking Example

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

## 10. Error Handling

### 10.1 Unified Capability Errors

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

### 10.2 Recoverable Handling

Recoverable capability failures are represented explicitly as `Result` values and handled with
`match`.

```ash
workflow resilient 
    observes sensor:temperature
    sets hvac:target 
{
    let reading = sensor_temperature_result();
    match reading {
        Ok { value: t } => {
            let update = set_hvac_target_result(calculate_target(t));
            match update {
                Ok { value: _ } => done,
                Err { error: Disconnected } => {
                    sleep(500ms)
                },
                Err { error: _ } => {
                    act log::error("HVAC control failed");
                    send alert:critical "HVAC failure"
                }
            }
        },
        Err { error: InputTimeout } => {
            sleep(100ms)
        },
        Err { error: _ } => {
            act log::error("sensor unavailable");
            send alert:critical "Sensor failure";
            use default:emergency_shutdown
        }
    }
}
```

## 11. IO Capability Boundary

The standard library `io` module family defines a clear capability boundary between pure computation and host-touching operations.

### 11.1 IO Module Capability Classification

| Module | Capability-Bearing | Rationale |
|--------|-------------------|-----------|
| `io::path` | **No** | Pure path manipulation; computation over values without host interaction |
| `io::stdio` | **Yes** | Reads from stdin, writes to stdout/stderr; host I/O authority required |
| `io::fs` | **Yes** | File creation, reading, writing; filesystem authority required |
| `io::dir` | **Yes** | Directory creation, removal, enumeration; filesystem authority required |
| `io::meta` | **Yes** | Metadata inspection (permissions, timestamps); filesystem authority required |
| `io::buf` | **Yes** | Buffered I/O helpers; delegates to capability-bearing operations |

### 11.2 Capability-Free Path Operations

`io::path` provides pure functions that operate only on path values:

```ash
use io::path::{Path, PathBuf};

workflow pure_path_manipulation {
    action build_path {
        effect: evaluative;  -- No host access, pure computation
        body: || -> {
            let base = PathBuf::new("/home/user");
            let config = base.join("config").join("app.conf");
            let parent = config.parent();  -- Pure computation
            let ext = config.extension();  -- Pure computation
            config
        };
    }
}
```

Path operations are **capability-free** because they:
- Manipulate string-like path representations only
- Perform no filesystem access
- Require no host authority
- Are deterministic and side-effect free

### 11.3 Capability-Bearing IO Operations

All other `io` submodules require host authority:

```ash
use io::fs;
use io::stdio;
use io::dir;

workflow file_operations {
    action write_config {
        effect: operational;  -- Host access required
        body: || -> {
            -- Requires filesystem write capability
            fs::write("/tmp/config.txt", "data")
        };
    }
    
    action print_message {
        effect: operational;  -- Host access required
        body: || -> {
            -- Requires stdout write capability
            stdio::println("Hello, World!")
        };
    }
    
    action list_files {
        effect: operational;  -- Host access required
        body: || -> {
            -- Requires directory read capability
            dir::read_dir("/tmp")
        };
    }
}
```

### 11.4 Result Type Reuse

The `io` modules reuse `result::Result<T, E>` rather than introducing a separate global `Result` ADT:

```ash
use result::Result;
use io::{Error, ErrorKind};

-- io::Result<T> is an alias for result::Result<T, io::Error>
let content: io::Result<String> = io::fs::read_to_string("file.txt");

match content {
    Ok { value: text } -> process(text),
    Err { error: e } -> handle_error(e),
}
```

### 11.5 IO Capability Vocabulary

The initial capability vocabulary for `io` operations includes:

| Capability | Module | Operations |
|------------|--------|------------|
| `stdin_read` | `io::stdio` | `read_line`, `stdin().read()` |
| `stdout_write` | `io::stdio` | `print`, `println`, `stdout().write()` |
| `stderr_write` | `io::stdio` | `eprint`, `eprintln` |
| `file_read` | `io::fs` | `read`, `read_to_string`, `File.read()` |
| `file_write` | `io::fs` | `write`, `append`, `File.write()` |
| `file_create` | `io::fs` | `create`, `create_new` |
| `dir_read` | `io::dir` | `read_dir`, `DirEntry` iteration |
| `dir_create` | `io::dir` | `create_dir`, `create_dir_all` |
| `dir_remove` | `io::dir` | `remove_dir`, `remove_dir_all` |
| `meta_read` | `io::meta` | `metadata`, permission queries |

## 12. Summary Table

| Feature | Input (observe/receive) | Output (set/send) |
|---------|------------------------|-------------------|
| **Effect** | Epistemic | Operational |
| **Obligations** | Can require observation | Can require output |
| **Policies** | Can restrict/transform | Can restrict/approve |
| **Provenance** | Record observed data or message intake | Record value sent |
| **Declaration** | `observes`, `receives` | `sets`, `sends` |
| **Type Safety** | Read schema validated | Write schema validated |
| **Errors** | Unavailable, Timeout | Failed, Rejected, BufferFull |

## 13. Implementation Tasks

- TASK-108: Effect tracking for all capabilities
- TASK-109: Obligation checking with capabilities
- TASK-110: Policy evaluation for input/output
- TASK-111: Provenance tracking for all capabilities
- TASK-112: Capability declaration verification
- TASK-113: Read/write type checking
- TASK-233: Capability definition parsing specification (completed)
- TASK-234: Capability definition parser implementation (pending)
- TASK-493: Freeze Stdlib IO V1 Contract (completed - this spec updated)

## References

- SPEC-001: Core Ash Language Specification
- SPEC-002: Stream and Behaviour Semantics
- SPEC-006: Policy System Overview
- SPEC-007: Policy Decision Framework
- SPEC-013: Source Scheduling
- SPEC-019: Role Runtime Semantics
- SPEC-022: Workflow Typing
- SPEC-023: Proxy Workflows
- SPEC-024: Capability-Role-Workflow Syntax (Reduced)
