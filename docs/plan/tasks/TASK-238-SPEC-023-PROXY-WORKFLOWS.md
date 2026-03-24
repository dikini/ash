# TASK-238: SPEC-023 - Proxy Workflows

## Status: Ready for Development

## Description

Create a new specification (SPEC-023) defining proxy workflows - workflows that represent external personas (humans, AI agents) and handle role-based message routing.

## Background

The idea space document `todo-examples/definitions/actors.md` describes external personas:
- Human reasoners (interact via mail, chat, websites)
- AI reasoners (interact via agent harness/MCP)
- External sensors, actuators, services

The `todo-examples/definitions/yield-resume.md` document describes the "Logical Handoff Protocol" for initiative transfer between workflows.

Proxy workflows bridge these concepts: they represent external personas within the Ash runtime, enabling governed interaction between deterministic workflows and external agents.

## Requirements

### 1. Proxy Workflow Definition

Define syntax for proxy workflows:

```ash
proxy board_proxy
    handles role(board_members)  -- Proxy represents this role
    receives requests:approval   -- Input stream
{
    loop {
        receive {
            req : ApprovalRequest => {
                -- Manage collective initiative
                par {
                    yield role(m1) req resume v1 => { ... },
                    yield role(m2) req resume v2 => { ... }
                }
                -- Fulfill the original contract
                resume Approved(consensual_sig);
            }
        }
    }
}
```

Key aspects:
- `proxy` keyword (vs `workflow`)
- `handles role(role_name)` declaration
- Message routing to proxy instead of direct role access

### 2. Role-to-Proxy Registry

Specify runtime registry:

```rust
pub struct ProxyRegistry {
    /// Maps role names to proxy workflow instances
    role_proxies: HashMap<RoleName, InstanceAddr>,
    /// Maps proxy instances to handled roles
    proxy_roles: HashMap<InstanceAddr, Vec<RoleName>>,
}
```

Registry operations:
- Register proxy for role
- Unregister proxy
- Route messages to role proxy
- Handle proxy failure/restart

### 3. Message Routing Semantics

Specify how `yield role(name)` routes to proxy:

```ash
-- In a regular workflow
yield role(manager) req : HandoffRequest
resume res : HandoffResponse { ... }

-- Runtime behavior:
-- 1. Look up manager role proxy in registry
-- 2. Send request to proxy's mailbox
-- 3. Suspend workflow (initiative transfer)
-- 4. Proxy receives request via `receive`
-- 5. Proxy eventually calls `resume` with response
-- 6. Original workflow resumes
```

### 4. Quorum and Consensus

Specify collective decision patterns:

```ash
-- Unanimous consent
par {
    yield role(m1) req resume Approved(_) => true,
    yield role(m2) req resume Approved(_) => true,
    yield role(m3) req resume Approved(_) => true
}

-- Majority vote
-- (requires collecting all responses and counting)
```

### 5. Failure Handling

Specify proxy failure scenarios:
- Proxy crash: route to backup proxy if available
- Proxy stall: timeout and escalate
- Role without proxy: error at yield point

### 6. Integration with Capability System

Specify proxy workflow capabilities:
- Proxies have same capability model as workflows
- Proxies can be spawned with specific roles
- Proxy authority checked like any workflow

## Acceptance Criteria

- [ ] Proxy workflow syntax specified
- [ ] Role-to-proxy registry semantics defined
- [ ] Message routing semantics documented
- [ ] Quorum/consensus patterns specified
- [ ] Failure handling scenarios documented
- [ ] Integration with capability system documented
- [ ] Examples: human proxy, AI proxy, multi-member proxy
- [ ] Sequence diagrams for yield/resume flow

## Dependencies

- None (specification-only task)
- Informed by DECISION-237 (obligation syntax)
- Related to TASK-235/SPEC-019 (roles)

## Related Documents

- `todo-examples/definitions/actors.md` (personas)
- `todo-examples/definitions/yield-resume.md` (handoff protocol)
- `todo-examples/definitions/workflows.md` (instance spawning)
- TASK-235/SPEC-019 (role semantics)

## Notes

Proxy workflows are the key feature enabling human-AI collaboration in Ash. They:
- Allow external agents to participate in workflows
- Provide governed boundaries for non-deterministic agents
- Enable collective decision-making (quorum)
- Maintain audit trail across workflow boundaries

The specification should clarify the relationship between:
- Proxy workflows and regular workflows
- Role proxies and role authority
- Yield/resume and message passing

This specification will inform TASK-239 (implementation).
