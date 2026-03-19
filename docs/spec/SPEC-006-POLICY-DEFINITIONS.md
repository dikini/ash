# SPEC-006: Policy Definition Syntax

## Status: Draft

## 1. Overview

This specification defines the canonical policy declaration model in Ash. A `policy <Name> { ... }` form declares a policy schema, closed policy expressions instantiate those schemas, and named policy bindings are what downstream lowering, type checking, and runtime verification reference.

## 2. Motivation

Currently, policies are hardcoded in the Rust implementation. Users need a way to:
- Define domain-specific policies (e.g., "MaxQueueDepth", "MinReplicationFactor")
- Share policy definitions and named policy bindings across workflows
- Get compile-time verification of policy conflicts

## 3. Policy Definition Syntax

### 3.1 Basic Policy Definition

```ash
policy <Name> {
  <field>: <type>,
  ...
}
```

**Example:**
```ash
policy RateLimit {
  requests: Int,
  window_secs: Int
}

policy MaxLatency {
  milliseconds: Int
}

policy GeoRestriction {
  allowed_regions: List<String>,
  forbidden_regions: List<String>
}
```

### 3.2 Policy with Default Values

```ash
policy RetryPolicy {
  max_attempts: Int = 3,
  backoff_ms: Int = 1000,
  max_backoff_ms: Int = 60000
}
```

### 3.3 Policy with Constraints

```ash
policy BoundedResource {
  min: Int,
  max: Int,
} where {
  min <= max
}
```

The `where` clause defines an invariant that must hold for all instances.

### 3.4 Parameterized Policies (Policy Templates)

```ash
policy<T> SizeLimit {
  max_size: T,
  current_size: T
} where {
  current_size <= max_size
}

-- Usage
policy SizeLimit<Int> IntSizeLimit;
policy SizeLimit<Bytes> ByteSizeLimit;
```

## 4. Canonical Policy Pipeline

Ash uses one policy story across the language:

1. **Policy definition**: `policy RateLimit { ... }` declares a schema and any `where` invariants.
2. **Policy expression**: a closed expression built from policy instances and combinators (SPEC-007).
3. **Named policy binding**: a declaration that gives a closed policy expression a stable name.
4. **Lowered/core policy**: the named binding lowers to a normalized policy graph / decision program.
5. **Runtime decision**: evaluating the lowered policy yields a `PolicyDecision`.

Policies are therefore not general first-class runtime values. They are named declarations and references that compile into a normalized policy representation.

The normalized representation is consumer-neutral, but the admissible terminal decisions are not:

- workflow `decide` consumes only `Permit` / `Deny`,
- capability-verification consumers may admit `{Permit, Deny, RequireApproval, Transform}`,
- `Warn` is not a policy decision and is handled as verification metadata outside the policy
  decision taxonomy.

### 4.1 Phase Ownership

- Parser rejection owns malformed policy syntax, malformed combinator syntax, and malformed policy
  binding syntax.
- Lowering owns closure, binding normalization, policy-name assignment, and conversion of a closed
  policy expression into a canonical `CorePolicy` identity.
- Type checking owns resolution of named policy references and consumer-specific decision-domain
  compatibility.
- Runtime and verification own evaluation outcomes, approval routing, transformation application,
  and provenance or warning recording.

## 5. Policy Usage in Workflows

### 5.1 Decide Statement

```ash
policy production_rate = RateLimit { requests: 100, window_secs: 60 };
policy latency_budget = MaxLatency { milliseconds: 500 };

workflow api_call {
  decide request_meta under production_rate then {
    decide request_meta under latency_budget then {
      act http_get with url: "https://api.example.com";
    }
  }
}
```

`DECIDE` is the workflow-level policy gate. It always references a named policy binding and never an anonymous inline policy expression.

### 5.2 Named Policy Bindings

```ash
policy production_rate = RateLimit { requests: 1000, window_secs: 60 };
policy staging_rate = RateLimit { requests: 100, window_secs: 60 };

workflow production_api {
  decide request_meta under production_rate then {
    act process_request;
  }
}
```

Named bindings are the canonical boundary between syntax and lowering. A binding must be closed: all parameters are supplied and all combinators resolve to concrete policy definitions or previously bound policy names.

### 5.3 Policy in Capability Definitions

```ash
policy payment_rate_limit = RateLimit { requests: 100, window_secs: 60 };
policy payment_fraud_gate = FraudCheck { max_risk_score: 0.1 };

capability process_payment : act(amount: Money)
  where policy payment_rate_limit
  where policy payment_fraud_gate;
```

Capability clauses may use a named policy binding directly. Inline policy expressions are allowed only as surface sugar if lowering first assigns them a stable internal name before verification.

## 5. Conflict Detection

### 5.1 Automatic SMT Encoding

Each policy definition generates an SMT encoding:

```rust
// RateLimit { requests: r, window_secs: w }
// Encodes to: actual_rate <= r/w
let actual_rate = Real::new_const(ctx, "actual_rate");
let max_rate = Real::from_real(ctx, r as i64, w as i64);
solver.assert(&actual_rate.le(&max_rate));
```

### 5.2 Where Clause Encoding

```ash
policy BoundedResource { min: Int, max: Int } where { min <= max }
```

Generates:
```rust
let min = Int::new_const(ctx, "min");
let max = Int::new_const(ctx, "max");
solver.assert(&min.le(&max));  -- Invariant
```

### 5.3 Conflict Types Detected

1. **Contradiction**: `RateLimit { r: 100 }` and `RateLimit { r: 50 }`
2. **Invariant Violation**: `BoundedResource { min: 10, max: 5 }`
3. **Resource Exhaustion**: Sum of mins > available
4. **Temporal Impossibility**: Disjoint time windows

## 6. Grammar

```
policy_def    ::= "policy" ["<" type_param ">"] identifier "{"
                    field_def* 
                    ["where" "{" constraint_expr "}"]
                  "}"

policy_binding ::= "policy" identifier "=" policy_expr ";"

field_def     ::= identifier ":" type ["=" expr] ","?

type          ::= "Int" | "Bool" | "String" | "Float" 
                | "List" "<" type ">"
                | "Map" "<" type "," type ">"
                | identifier

constraint_expr ::= expr  -- Must evaluate to Bool

policy_instance ::= identifier "{" field_init* "}"
policy_expr   ::= policy_instance
                | identifier            -- named policy binding
                | ...                   -- combinator forms defined in SPEC-007

field_init    ::= identifier ":" expr ","?
```

`policy_expr` is intentionally shared with SPEC-007. This spec defines the base policy-instance form and named binding form; SPEC-007 defines how combinators extend the expression grammar.

## 7. Semantic Analysis

### 7.1 Well-formedness Rules

1. All fields must be initialized (unless they have defaults)
2. Field types must match declared types
3. Where clause must reference only declared fields
4. Where clause must be a boolean expression
5. No circular dependencies between policies

Named policy bindings add these well-formedness checks:

1. The bound expression must be closed after defaults are applied.
2. Every referenced policy definition or policy binding must resolve uniquely.
3. Every `DECIDE ... under <policy_name>` must resolve to a named policy binding.
4. Policies used by workflow `DECIDE` sites must lower to terminal decisions in `{Permit, Deny}`.
5. Policies used by capability-verification sites may additionally lower to the verification
   outcome set `{Permit, Deny, RequireApproval, Transform}`.
6. Warnings are not policy decisions and must not be modeled as terminal policy outcomes.

### 7.2 Type Checking

```rust
// In type checker
fn check_policy_def(def: &PolicyDef) -> Result<(), TypeError> {
    // Check field types are valid
    for field in &def.fields {
        validate_type(&field.ty)?;
    }
    
    // Check where clause is bool and references valid fields
    let where_ty = infer_expr(&def.where_clause)?;
    if !where_ty.is_bool() {
        return Err(TypeError::ExpectedBool(where_ty));
    }
    
    // Check no undefined fields referenced
    let defined_fields: HashSet<_> = def.fields.iter()
        .map(|f| &f.name)
        .collect();
    let referenced = free_vars(&def.where_clause);
    for var in referenced {
        if !defined_fields.contains(&var) {
            return Err(TypeError::UndefinedField(var));
        }
    }
    
    Ok(())
}
```

## 8. Code Generation

### 8.1 To SMT

Each policy definition generates a struct and encoding function:

```rust
// Generated from: policy RateLimit { requests: Int, window_secs: Int }
pub struct RateLimit {
    pub requests: i64,
    pub window_secs: i64,
}

impl SmtEncodable for RateLimit {
    fn encode(&self, ctx: &Context) -> Vec<Bool> {
        let actual_rate = Real::new_const(ctx, "rate_limit_actual_rate");
        let max_rate = Real::from_real(
            ctx, 
            self.requests as i64, 
            self.window_secs as i64
        );
        vec![actual_rate.le(&max_rate)]
    }
}
```

### 8.2 To Core IR

Policy bindings lower to one canonical core representation:

```rust
pub struct CorePolicy {
    pub name: PolicyName,
    pub params: Vec<PolicyParam>,
    pub graph: PolicyGraph,
}
```

`PolicyGraph` is a normalized decision graph produced from policy instances and combinators. Both static policy declarations and deferred dynamic-policy sources must compile into this same core representation before type checking or runtime use.

Workflow policy checks then reference only the lowered policy name:

```ash
policy production_rate = RateLimit { requests: 100, window_secs: 60 };

decide request_meta under production_rate then {
  act process_request;
}
```

Becomes:
```rust
Workflow::Decide {
    policy: PolicyName::new("production_rate"),
    subject: Expr::Var("request_meta"),
    cont: ...,
}
```

## 9. Examples

### 9.1 Complete Example

```ash
-- Define policies
policy RateLimit {
  requests: Int,
  window_secs: Int
}

policy CircuitBreaker {
  failure_threshold: Int = 5,
  recovery_timeout_ms: Int = 30000
}

policy DataResidency {
  allowed_regions: List<String>,
  encryption_required: Bool = true
}

-- Use in workflow
policy order_rate_limit = RateLimit { requests: 1000, window_secs: 60 };
policy order_circuit_breaker = CircuitBreaker { failure_threshold: 3 };
policy order_data_residency = DataResidency {
  allowed_regions: ["us-east-1", "eu-west-1"],
  encryption_required: true
};

workflow process_order {
  decide request_meta under order_rate_limit then {
    decide request_meta under order_circuit_breaker then {
      decide request_meta under order_data_residency then {
        observe inventory with product_id: order.product_id as inventory;

        if inventory.count > 0 then {
          act charge_payment with amount: order.total;
          act ship_order with order: order;
        } else {
          act notify_backorder with customer: order.customer;
        }
      }
    }
  }
}
```

### 9.2 Policy Library

```ash
-- policies.ash - Reusable policy library

policy RateLimit {
  requests: Int,
  window_secs: Int
}

policy QuotaLimit {
  max_calls: Int,
  period: String  -- "day", "week", "month"
}

policy SecurityLevel {
  min_encryption: String,  -- "AES128", "AES256"
  mfa_required: Bool
}

-- Export for use in other files
export RateLimit, QuotaLimit, SecurityLevel;
```

## 10. Related Documents

- SPEC-001: IR (for runtime representation)
- SPEC-003: Type System (for constraint checking)
- TASK-024b: Z3 SMT Integration (for conflict detection)
