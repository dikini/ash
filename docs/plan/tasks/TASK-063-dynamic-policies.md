# TASK-063: Dynamic Policy Registration (Deferred)

## Status: ⏸️ Deferred

## Description

Implement runtime policy registration and hot-reloading from external sources (SPEC-008). This enables policies to be loaded from databases, configuration files, or policy servers during workflow execution.

## Specification Reference

- SPEC-008: Dynamic Policy Registration

## Deferred Rationale

This task is deferred because:

1. **Complexity**: Requires runtime SMT solving or complex caching strategies
2. **Performance**: Dynamic checking adds significant latency without caching
3. **Safety**: Runtime policy errors are harder to debug than compile-time errors
4. **Precedence**: Static policies (TASK-061) and combinators (TASK-062) cover 80% of use cases
5. **Infrastructure**: Needs policy registry service, which is significant infrastructure

## Unblocking Criteria

This task can be unblocked when:
- [ ] Static policies are mature and widely used
- [ ] Users request runtime policy updates as a critical feature
- [ ] Performance of dynamic checking is acceptable (with caching)
- [ ] Policy registry infrastructure exists
- [ ] SMT solving overhead at runtime is acceptable

## Requirements (for future implementation)

### 1. Policy Registry Capability

```ash
capability policy_registry : observe(name: String, version: String?) -> Policy
  effect: epistemic;
```

### 2. Runtime Policy Loading

```ash
workflow dynamic_api {
  -- Load policy from external registry
  let rate_limit = observe policy_registry 
    with name: "api_rate_limit", version: "v2.1.0";
  
  -- Apply dynamically loaded policy
  check rate_limit with tier: user.tier;
  
  act process_request;
}
```

### 3. Hot Reloading

```ash
-- Policy cache refreshes periodically
with policy_registry.refresh_interval(secs: 60) do {
  workflow auto_reload {
    check policy_registry.get("current_security_policy");
    act handle_request;
  }
}
```

### 4. Implementation Components

**When implementing, create:**

1. `crates/ash-interp/src/dynamic_policy.rs` - Runtime policy engine
2. `crates/ash-interp/src/policy_cache.rs` - Validation cache
3. `crates/ash-cli/src/commands/policy.rs` - CLI for policy management
4. Policy registry client trait for different backends

### 5. Key Challenges

**SMT at Runtime**:
- Need to cache pre-computed valid ranges
- Or use lightweight structural checks
- Consider WASM sandboxing for policy evaluation

**Security**:
- Verify policy signatures
- Prevent policy injection attacks
- Audit log all policy changes

**Performance**:
- Bloom filters for quick rejection
- LRU cache for valid policies
- Async policy fetching

## Estimated Effort (when unblocked)

40 hours (significant infrastructure)

## Dependencies (when unblocked)

- TASK-061: Policy Definitions (static base)
- TASK-024b: SMT Integration (for runtime solving)
- Infrastructure: Policy registry service

## Related

- SPEC-006: Policy Definition Syntax
- SPEC-007: Policy Combinators
- SPEC-008: Dynamic Policy Registration
