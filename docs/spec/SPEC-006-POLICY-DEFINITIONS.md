# SPEC-006: Policy Definition Syntax

## Status: Draft

## 1. Overview

This specification defines static policy definitions in Ash, allowing users to declare custom policy types with named parameters and constraints that are checked at compile-time using SMT solving.

## 2. Motivation

Currently, policies are hardcoded in the Rust implementation. Users need a way to:
- Define domain-specific policies (e.g., "MaxQueueDepth", "MinReplicationFactor")
- Share policy definitions across workflows
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

## 4. Policy Usage in Workflows

### 4.1 Check Statement

```ash
workflow api_call {
  -- Check a policy instance
  check RateLimit { requests: 100, window_secs: 60 };
  check MaxLatency { milliseconds: 500 };
  
  act http_get with url: "https://api.example.com";
}
```

### 4.2 Named Policy Instances

```ash
let production_rate = RateLimit { requests: 1000, window_secs: 60 };
let staging_rate = RateLimit { requests: 100, window_secs: 60 };

workflow production_api {
  check production_rate;
  act process_request;
}
```

### 4.3 Policy in Capability Definitions

```ash
capability process_payment : act(amount: Money)
  where RateLimit { requests: 100, window_secs: 60 }
  where FraudCheck { max_risk_score: 0.1 };
```

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

field_def     ::= identifier ":" type ["=" expr] ","?

type          ::= "Int" | "Bool" | "String" | "Float" 
                | "List" "<" type ">"
                | "Map" "<" type "," type ">"
                | identifier

constraint_expr ::= expr  -- Must evaluate to Bool

check_stmt    ::= "check" policy_instance ";"

policy_instance ::= identifier "{" field_init* "}"

field_init    ::= identifier ":" expr ","?
```

## 7. Semantic Analysis

### 7.1 Well-formedness Rules

1. All fields must be initialized (unless they have defaults)
2. Field types must match declared types
3. Where clause must reference only declared fields
4. Where clause must be a boolean expression
5. No circular dependencies between policies

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

Policy checks become guard conditions:

```ash
check RateLimit { requests: 100, window_secs: 60 };
```

Becomes:
```rust
Workflow::Check {
    obligation: Obligation::Policy(Box::new(RateLimit { ... })),
    ...
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
workflow process_order {
  check RateLimit { requests: 1000, window_secs: 60 };
  check CircuitBreaker { failure_threshold: 3 };
  check DataResidency { 
    allowed_regions: ["us-east-1", "eu-west-1"],
    encryption_required: true 
  };
  
  observe inventory with product_id: order.product_id as inventory;
  
  if inventory.count > 0 then {
    act charge_payment with amount: order.total;
    act ship_order with order: order;
  } else {
    act notify_backorder with customer: order.customer;
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
