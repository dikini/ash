# SPEC-007: Policy Combinators

## Status: Draft

## 1. Overview

This specification defines the policy-expression combinators that extend SPEC-006. Combinators build policy expressions, but those expressions are canonical only once they are bound to a name and lowered into the shared core policy representation.

## 2. Motivation

Policy definitions (SPEC-006) are static and named. Policy combinators provide:
- **Structured composition**: Build richer policy expressions from policy instances and named policy bindings
- **Reusability**: Combine existing policies in new ways
- **Expressiveness**: Express complex constraints declaratively
- **Composability**: Build reusable policy expressions before binding them to a canonical policy name

## 3. Core Combinators

All combinators in this section produce `policy_expr` syntax. A `policy_expr` may appear:

- on the right-hand side of a named `policy <name> = ...` binding,
- in capability policy clauses (directly or as inline sugar that lowering names internally),
- in deferred dynamic-policy compilation.

They do not introduce a general-purpose runtime `Policy` value type.

### 3.1 Logical Combinators

```ash
-- Conjunction: All policies must hold
and(policy1, policy2, ...)

-- Disjunction: At least one policy must hold
or(policy1, policy2, ...)

-- Negation: Policy must not hold
not(policy)

-- Implication: If antecedent then consequent
implies(antecedent, consequent)
```

**Examples:**
```ash
policy production = and(
  rate_limit(1000, 60),
  region(["us-east-1", "eu-west-1"])
);

policy fallback = or(
  primary_region,
  secondary_region
);

policy restricted = not(forbidden_region);
```

### 3.2 Quantifier Combinators

```ash
-- Universal: All items satisfy policy
forall(items, fn(item) -> policy)

-- Existential: At least one item satisfies policy
exists(items, fn(item) -> policy)
```

**Examples:**
```ash
-- All regions must be in allowed list
forall(regions, fn(r) -> region(["us", "eu"]));

-- At least one datacenter must have capacity
exists(datacenters, fn(dc) -> available_capacity(dc, 100));
```

### 3.3 Arithmetic Combinators

```ash
-- Sum of values across policies
sum(policy_extractors) <= limit

-- Product
product(factors) <= limit

-- Min/Max
min(values) >= threshold
max(values) <= threshold
```

**Examples:**
```ash
-- Total budget across all microservices
sum([service1.budget, service2.budget, service3.budget]) <= 10000;

-- Minimum availability across regions
min([region1.availability, region2.availability]) >= 99.9;
```

### 3.4 Temporal Combinators

```ash
-- Sequential: Policies apply in sequence
sequential([policy1, policy2, policy3])

-- Concurrent: Policies apply simultaneously
concurrent([policy1, policy2])

-- Before/After time constraints
before(policy, timestamp)
after(policy, timestamp)
during(policy, time_range)
```

**Examples:**
```ash
-- Rate limit, then check quota
sequential([
  rate_limit(100, 60),
  daily_quota(10000)
]);

-- Valid only during business hours
during(
  high_security,
  business_hours
);
```

### 3.5 Conditional Combinators

```ash
-- If-then-else for policies
when(condition, then_policy, else_policy)

-- Pattern matching on values
match(value, {
  pattern1 => policy1,
  pattern2 => policy2,
  _ => default_policy
})
```

**Examples:**
```ash
-- Different limits for premium vs standard
when(
  user.tier == "premium",
  rate_limit(10000, 60),
  rate_limit(1000, 60)
);

-- Region-specific policies
match(datacenter.region, {
  "us" => us_policy,
  "eu" => gdpr_policy,
  _ => default_policy
});
```

## 4. Higher-Order Policies

### 4.1 Policy Transformers

```ash
-- Retry a policy with backoff
retry(policy, max_attempts: Int, backoff_ms: Int)

-- Timeout for policy evaluation
timeout(policy, duration_ms: Int)

-- Cache policy result
cache(policy, ttl_secs: Int)

-- Circuit breaker around policy
circuit_breaker(policy, failure_threshold: Int, recovery_ms: Int)
```

### 4.2 Policy Composition Patterns

```ash
-- Layered defense: Each layer must pass
defense_in_depth([
  perimeter_security,
  network_security,
  application_security
]);

-- Defense in breadth: Multiple independent checks
defense_in_breadth([
  signature_based_detection,
  behavior_based_detection,
  heuristic_detection
]);

-- Graceful degradation
degrade([
  full_feature_set,
  reduced_feature_set,
  minimum_viable
]);
```

## 5. Combinator DSL

### 5.1 Infix Operators

For common combinators, provide operator syntax:

```ash
policy combined = policy1 & policy2;   -- and
policy either = policy1 | policy2;      -- or
policy inverted = !policy;              -- not
policy sequenced = policy1 >> policy2;  -- sequential
```

### 5.2 Method Chaining

```ash
policy hardened_rate_limit =
  rate_limit(100, 60)
    .and(region(["us", "eu"]))
    .and(encryption_required())
    .with_retry(max_attempts: 3)
    .with_timeout(ms: 5000);
```

## 6. Type System and Lowering Integration

### 6.1 Policy Expression Kind

```rust
// Policy expressions are syntax / lowered IR, not general runtime values.
pub enum PolicyExpr {
    Ref(PolicyName),
    Instance(PolicyInstance),
    And(Vec<PolicyExpr>),
    Or(Vec<PolicyExpr>),
    Not(Box<PolicyExpr>),
    ...
}
```

The type checker validates that every combinator operand is a policy expression and that each resulting expression can be lowered into a `CorePolicy` decision graph. Runtime evaluation consumes the lowered graph and yields `PolicyDecision`; it does not manipulate source-level `PolicyExpr` values directly.

The consumer determines the admissible terminal decision set:

- workflow `decide` sites observe only `Permit` / `Deny`,
- capability-verification sites may observe the verification outcome set
  `{Permit, Deny, RequireApproval, Transform}`,
- `Warn` is not a `PolicyDecision`; it belongs to verification metadata, not the canonical policy
  decision taxonomy.

### 6.2 Type Inference

```ash
policy regional_limit =
  and(rate_limit(100, 60), region(["us"]));

policy tiered_rate_limit =
  match tier {
    "premium" => rate_limit(10000, 60),
    "standard" => rate_limit(1000, 60),
    _ => rate_limit(100, 60)
  };
```

Inference operates over `policy_expr` syntax while the binding remains open. A policy binding is well-formed only when the final expression is closed and can lower to one named `CorePolicy`.

## 7. Semantics

### 7.1 Evaluation Strategy

**Lazy Evaluation**: Policies are only checked when needed

```ash
policy expensive_check = slow_audit();
policy cheap_check = quick_scan();

-- Only evaluate expensive_check if cheap_check passes
and(cheap_check, expensive_check);
```

**Short-circuit**: `and` stops at first failure, `or` stops at first success

### 7.2 Error Reporting

Combinators preserve source location for good error messages:

```
Policy violation at line 42:
  In 'and' combinator, first argument failed:
    rate_limit(100, 60) violated:
      Actual rate: 150 req/s
      Limit: 100 req/s
```

### 7.3 Conflict Detection

Combinators flatten to SMT constraints:

```ash
and(
  rate_limit(100, 60),
  rate_limit(50, 60)
)
```

Becomes:
```smt
(assert (<= actual_rate 1.67))  -- 100/60
(assert (<= actual_rate 0.83))  -- 50/60
-- Conflict: actual_rate cannot be <= 0.83 and satisfy both
```

## 8. Examples

### 8.1 Multi-Tier Rate Limiting

```ash
policy tiered_rate_limit =
  let base = rate_limit(
    match tier {
      "enterprise" => 100000,
      "premium" => 10000,
      "standard" => 1000,
      _ => 100
    },
    60
  );
  
  let burst = match tier {
    "enterprise" => burst_limit(5000),
    "premium" => burst_limit(500),
    _ => always_ok()
  };
  
  base.and(burst);

workflow api_endpoint {
  decide request_meta under tiered_rate_limit then {
    act process_request;
  }
}
```

### 8.2 Complex Security Policy

```ash
policy security_policy = and(
  -- Authentication
  or(
    mfa_enabled(),
    ip_whitelist(["10.0.0.0/8"])
  ),
  
  -- Authorization
  has_permission("payment:write"),
  
  -- Encryption
  and(
    tls_version("1.3"),
    cipher_suite(["AES256-GCM", "CHACHA20-POLY1305"])
  ),
  
  -- Time-based
  during(business_hours, timezone: "America/New_York")
);
```

### 8.3 Resource Allocation

```ash
policy resource_policy =
  let total_cpu = sum(services, fn(s) -> s.cpu_request);
  let total_memory = sum(services, fn(s) -> s.memory_request);
  
  and(
    total_cpu <= cluster.available_cpu,
    total_memory <= cluster.available_memory,
    
    -- Anti-affinity: don't put all on same node
    forall(services, fn(s1) ->
      exists(services, fn(s2) ->
        s1 != s2 and s1.node != s2.node
      )
    )
  );
```

## 9. Grammar Extension

```
policy_expr  ::= primary
               | policy_expr "&" policy_expr    -- and
               | policy_expr "|" policy_expr    -- or
               | "!" policy_expr                -- not
               | policy_expr ">>" policy_expr   -- sequential
               | combinator_call

combinator_call ::= identifier "(" args ")"

primary      ::= identifier
               | "(" policy_expr ")"
               | policy_literal

policy_literal ::= "rate_limit" "(" int "," int ")"
                 | "region" "(" list ")"
                 | "time_range" "(" int "," int ")"
                 | ...
```

## 10. Implementation Strategy

### 10.1 Normalization

All combinators normalize to a canonical form for SMT:

```ash
-- Input
and(a, or(b, c))

-- Normalized (DNF)
or(and(a, b), and(a, c))
```

### 10.2 Optimization

- Constant folding: `and(always_ok, p)` → `p`
- Idempotent: `and(p, p)` → `p`
- Absorption: `and(p, or(p, q))` → `p`
- Contradiction detection: `and(p, not(p))` → `unsat`

### 10.3 Integration with SMT

Each combinator has a corresponding SMT encoding:

```rust
impl PolicyExpr {
    fn to_smt(&self, ctx: &Context) -> Vec<Bool> {
        match self {
            PolicyExpr::And(ps) => {
                ps.iter().flat_map(|p| p.to_smt(ctx)).collect()
            }
            PolicyExpr::Or(ps) => {
                // Create auxiliary variable for disjunction
                let aux = Bool::fresh_const(ctx, "or_result");
                let constraints: Vec<_> = ps.iter()
                    .map(|p| p.to_smt(ctx))
                    .collect();
                // aux <=> (p1 OR p2 OR ...)
                vec![aux]
            }
            // ... etc
        }
    }
}
```

## 11. Related Documents

- SPEC-006: Policy Definition Syntax (for base policies)
- SPEC-003: Type System (for type inference)
- rust-skills: Error handling patterns
