# SPEC-008: Dynamic Policy Registration

## Status: Draft (Deferred)

## 1. Overview

This specification defines the deferred runtime-loading story for policies in Ash. Dynamic policy sources do not introduce a separate policy model; they must compile into the same named core policy representation described in SPEC-006 and SPEC-007 before runtime verification consumes them.

## 2. Motivation

Static policies (SPEC-006) and combinators (SPEC-007) are defined at compile-time. Dynamic policies enable:
- **External configuration**: Update policies without redeployment
- **Policy servers**: Centralized policy management
- **A/B testing**: Gradual rollout of new policies
- **Emergency overrides**: Immediate policy changes for incidents
- **Multi-tenancy**: Per-customer policy customization

## 3. Deferred Rationale

This feature is deferred because:
1. **Complexity**: Requires runtime SMT solving or policy caching
2. **Performance**: Dynamic checking adds latency
3. **Safety**: Runtime policy errors are harder to debug
4. **Precedence**: Static policies cover 80% of use cases

## 4. Design Sketch

### 4.1 Policy Registry Capability

```ash
capability policy_registry : observe(name: String) -> PolicyBundle
  effect: epistemic;
```

`PolicyBundle` is an external source artifact, not a workflow-level first-class `Policy` value. The host/runtime compiles it into a named `CorePolicy` before any `DECIDE` or capability verification step can reference it.

### 4.2 Loading Policies at Runtime

```text
1. Runtime fetches `PolicyBundle` from `policy_registry`
2. Runtime validates signature / version / schema compatibility
3. Runtime compiles the bundle to `CorePolicy { name, graph, ... }`
4. Runtime registers that compiled policy under an explicit policy name
5. Workflows and capability checks continue to reference that explicit policy name
```

### 4.3 Policy Hot-Reloading

Policies may refresh periodically, but each refresh still produces a new compiled `CorePolicy` version under a stable policy name or versioned alias.

### 4.4 Versioned Policies

Pinning a specific version affects which external policy bundle is loaded, not the language-level policy syntax. The workflow contract remains `DECIDE ... under policy_name`.

## 5. Implementation Approach

### 5.1 Runtime SMT Solving

```rust
pub struct DynamicPolicyEngine {
    z3_context: Arc<Mutex<Context>>,
    policy_cache: Arc<RwLock<HashMap<String, CachedPolicy>>>,
    registry_client: Box<dyn PolicyRegistryClient>,
}

impl DynamicPolicyEngine {
    pub async fn check_policy(
        &self,
        policy_name: &str,
        context: &RuntimeContext
    ) -> Result<PolicyResult, PolicyError> {
        // 1. Check cache
        if let Some(cached) = self.policy_cache.read().unwrap().get(policy_name) {
            if !cached.is_expired() {
                return self.evaluate_cached(cached, context);
            }
        }
        
        // 2. Fetch from registry
        let policy_def = self.registry_client.fetch(policy_name).await?;
        
        // 3. Parse and compile to the canonical core policy representation
        let compiled = self.compile_policy(&policy_def)?;
        
        // 4. Evaluate
        let result = self.evaluate(&compiled, context)?;
        
        // 5. Cache result
        self.policy_cache.write().unwrap().insert(
            policy_name.to_string(),
            CachedPolicy::new(compiled)
        );
        
        Ok(result)
    }
}
```

### 5.2 Policy Validation Cache

To avoid runtime SMT solving latency:

```rust
pub struct PolicyValidationCache {
    /// Pre-computed valid parameter ranges
    valid_ranges: HashMap<String, ParameterRange>,
    /// Quick rejection for known-bad values
    bloom_filter: BloomFilter,
}

impl PolicyValidationCache {
    /// Fast path: O(1) check
    pub fn quick_check(&self, params: &Params) -> PolicyResult {
        if self.bloom_filter.contains(params) {
            // Might be invalid, do full check
            PolicyResult::NeedFullCheck
        } else {
            // Definitely valid (with high probability)
            PolicyResult::Valid
        }
    }
}
```

## 6. Security Considerations

### 6.1 Policy Integrity

```ash
-- Host/runtime verifies a signed policy bundle before registration
policy_registry.get_signed(
  name: "critical_security_policy",
  public_key: org_security_key
);
```

### 6.2 Audit Logging

All dynamic policy changes must be logged:
- Policy loaded
- Policy version changed
- Policy check result
- Cache hit/miss

## 7. Error Handling

```rust
pub enum DynamicPolicyError {
    RegistryUnreachable { url: String, cause: String },
    PolicyNotFound { name: String },
    ParseError { source: String, line: usize },
    ValidationFailed { errors: Vec<String> },
    Timeout { elapsed_ms: u64 },
    VersionConflict { expected: String, found: String },
}
```

## 8. Future Work

When this is implemented:

1. **Policy DSL**: Simple text format for non-programmers
2. **Policy IDE**: Web UI for editing policies
3. **Policy simulation**: Test policies before deployment
4. **ML-based optimization**: Learn optimal policy parameters
5. **Distributed consensus**: Consistent policies across regions

Dynamic policies therefore affect policy provisioning, not the core syntax/type/runtime contract:

- syntax still references explicit policy names,
- type checking still validates policy references and decision subsets,
- runtime still consumes `PolicyDecision` values from normalized core policies.

## 9. Related Documents

- SPEC-006: Policy Definition Syntax (compile-time)
- SPEC-007: Policy Combinators (functional composition)
- SPEC-005: CLI (for policy management commands)
