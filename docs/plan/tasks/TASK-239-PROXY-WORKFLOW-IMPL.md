# TASK-239: Implement Proxy Workflow Runtime

## Status: Blocked on TASK-238

## Description

Implement proxy workflow runtime support as specified in SPEC-023, enabling workflows to represent external personas and handle role-based message routing.

## Background

After TASK-238 completes, SPEC-023 will define proxy workflow semantics. This task implements:
1. Proxy workflow AST and parser support
2. Role-to-proxy registry
3. Message routing for `yield role(name)`
4. Quorum/consensus patterns

## Requirements

### 1. AST Extensions

Extend `crates/ash-core/src/ast.rs`:

```rust
/// Proxy workflow definition
pub struct ProxyDef {
    pub name: Name,
    pub handled_role: Role,
    pub inputs: Vec<InputCapability>,
    pub body: Workflow,
    pub span: Span,
}

/// Module can contain workflows, proxies, and definitions
pub enum ModuleItem {
    Workflow(WorkflowDef),
    Proxy(ProxyDef),  // NEW
    Capability(Capability),
    Policy(Policy),
    Role(Role),
}
```

### 2. Parser Support

Add to `crates/ash-parser/src/parse_module.rs`:

```rust
pub fn proxy_def(input: &mut Input) -> PResult<ProxyDef> {
    // Parse: proxy name handles role(role_name) receives ... { body }
}
```

### 3. Role-to-Proxy Registry

Create `crates/ash-interp/src/proxy_registry.rs`:

```rust
pub struct ProxyRegistry {
    role_proxies: HashMap<RoleName, InstanceAddr>,
    proxy_roles: HashMap<InstanceAddr, HashSet<RoleName>>,
}

impl ProxyRegistry {
    pub fn register(&mut self, role: RoleName, proxy: InstanceAddr);
    pub fn unregister(&mut self, role: &RoleName);
    pub fn lookup(&self, role: &RoleName) -> Option<InstanceAddr>;
    pub fn get_roles(&self, proxy: &InstanceAddr) -> Option<&HashSet<RoleName>>;
}
```

### 4. Yield-to-Role Routing

Modify `crates/ash-interp/src/execute.rs`:

```rust
pub fn eval_yield(
    &self,
    role: &Role,
    request: &Value,
    expected_response_type: &Type,
    continuation: &Workflow,
) -> EvalResult<YieldState> {
    // Look up proxy for role
    let proxy_addr = self.proxy_registry
        .lookup(&role.name)
        .ok_or(EvalError::NoProxyForRole)?;
    
    // Send request to proxy
    self.send_message(proxy_addr, request)?;
    
    // Suspend workflow, waiting for resume
    Ok(YieldState::Suspended {
        correlation_id: generate_correlation_id(),
        expected_response_type: expected_response_type.clone(),
        continuation: continuation.clone(),
    })
}
```

### 5. Resume from Proxy

```rust
pub fn eval_resume(
    &self,
    correlation_id: CorrelationId,
    response: &Value,
) -> EvalResult<()> {
    // Find suspended workflow by correlation ID
    let suspended = self.suspended_workflows
        .get(&correlation_id)
        .ok_or(EvalError::UnknownCorrelationId)?;
    
    // Validate response type
    self.type_check(response, &suspended.expected_response_type)?;
    
    // Resume workflow with response
    self.resume_workflow(suspended, response)
}
```

### 6. Quorum Patterns

Implement `par` with quorum semantics:

```rust
pub fn eval_par_quorum(
    &self,
    branches: Vec<YieldBranch>,
    quorum: QuorumSpec,  // Unanimous, Majority, AtLeast(n)
) -> EvalResult<ParResult> {
    // Collect responses from all branches
    // Apply quorum specification
    // Return result or timeout
}
```

## Test Requirements

### Basic Proxy Tests
- [ ] Proxy workflow parses and spawns
- [ ] Role-to-proxy registry registration/lookup
- [ ] Yield to role routes to proxy
- [ ] Resume from proxy resumes original workflow

### Message Routing Tests
- [ ] Request reaches proxy mailbox
- [ ] Response correlated to correct workflow
- [ ] Multiple concurrent yields handled

### Quorum Tests
- [ ] Unanimous consent pattern works
- [ ] Timeout when quorum not reached
- [ ] Partial responses handled correctly

### Failure Tests
- [ ] Yield to role without proxy fails
- [ ] Proxy crash detection and handling
- [ ] Resume with wrong type fails
- [ ] Resume with unknown correlation ID fails

### Integration Tests
- [ ] End-to-end: workflow → proxy → workflow
- [ ] Human proxy simulation
- [ ] AI proxy simulation
- [ ] Multi-member board proxy

## Acceptance Criteria

- [ ] ProxyDef AST and parser implemented
- [ ] Role-to-proxy registry working
- [ ] `yield role(name)` routes to proxy
- [ ] `resume` from proxy resumes workflow
- [ ] Correlation IDs link requests/responses
- [ ] Quorum patterns work correctly
- [ ] All test cases pass
- [ ] Audit trail captures handoffs

## Dependencies

- TASK-238: SPEC-023 Proxy Workflows (must be complete)
- TASK-236: Role runtime (recommended, for role context)

## Estimated Effort

6-8 weeks
- 1 week: AST/parser
- 2 weeks: Registry and routing
- 2 weeks: Yield/resume implementation
- 2 weeks: Quorum patterns
- 1-2 weeks: Integration and testing

## Related Documents

- `docs/spec/SPEC-023-PROXY-WORKFLOWS.md` (to be created)
- `crates/ash-interp/src/execute.rs`
- `crates/ash-core/src/ast.rs`

## Notes

This is a major feature that enables human-AI collaboration. Key implementation challenges:
- Correlation ID generation and management
- Workflow suspension/resumption
- Timeout handling for stalled proxies
- Type-safe response validation

Consider starting with a simplified version (single proxy, no quorum) and adding complexity incrementally.
