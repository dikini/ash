# SPEC-023: Proxy Workflows

## Status: Draft

## 1. Overview

Enable workflows to represent external personas (humans, AI agents) and facilitate governed interaction between deterministic workflows and non-deterministic external agents via message passing.

**Key Insight:** Every external participant is represented by a workflow. Communication between workflows and external agents uses standard Ash message passing with correlation IDs for request/response pairing.

**Target:** Release 0.6.0  
**Effort Estimate:** 6-8 weeks implementation

---

## 2. Motivation

### 2.1 Problem

Existing Ash primitives (`signal`, `receive`) are passive. They don't natively capture the **initiative handoff**—the intentional suspension of a workflow to allow another persona to perform a turn and provide a governed response.

### 2.2 Solution

Proxy workflows:
- Represent external personas within the Ash runtime
- Handle role-based message routing
- Enable typed request/response dialogues
- Support collective decision-making (quorum)
- Maintain audit trail across workflow boundaries

---

## 3. Core Concepts

### 3.1 Proxy Workflow

A workflow that handles messages for a specific role:

```ash
proxy board_proxy
    handles role(board_members)      -- Proxy represents this role
    receives requests:approval_request  -- Input stream
{
    loop {
        receive {
            req : ApprovalRequest => {
                -- Handle request
                act process_request with req;
                
                -- Send response back to original workflow
                resume Approved(decision) : ApprovalResponse;
            }
        }
    }
}
```

### 3.2 Role-to-Proxy Registry

Runtime registry mapping roles to proxy instances:

```rust
pub struct ProxyRegistry {
    /// Maps role names to proxy workflow instances
    role_proxies: HashMap<RoleName, InstanceAddr>,
    
    /// Maps proxy instances to handled roles
    proxy_roles: HashMap<InstanceAddr, HashSet<RoleName>>,
}

impl ProxyRegistry {
    pub fn register(&mut self, role: RoleName, proxy: InstanceAddr);
    pub fn unregister(&mut self, role: &RoleName);
    pub fn lookup(&self, role: &RoleName) -> Option<InstanceAddr>;
}
```

### 3.3 Initiative Handoff

When a workflow yields to a role, the runtime:

1. Looks up proxy for the role in registry
2. Sends request to proxy's mailbox with correlation ID
3. Suspends the yielding workflow
4. Proxy receives request, processes it
5. Proxy sends response with matching correlation ID
6. Runtime resumes original workflow with response

---

## 4. Syntax

### 4.1 Proxy Definition

```bnf
proxy_def       ::= "proxy" IDENTIFIER
                    "handles" "role" "(" IDENTIFIER ")"
                    input_capabilities?
                    "{" workflow_body "}"

input_capabilities ::= ("observes" | "receives") capability_ref ("," capability_ref)*
```

### 4.2 Yield Expression

```bnf
yield_expr      ::= "yield" "role" "(" IDENTIFIER ")" expression
                    "resume" IDENTIFIER ":" type "{" resume_arms "}"

resume_arms     ::= resume_arm ("," resume_arm)*
resume_arm      ::= pattern "=>" workflow
```

### 4.3 Resume Statement

```bnf
resume_stmt     ::= "resume" expression ":" type
```

---

## 5. Examples

### 5.1 Simple Approval Workflow

```ash
-- Workflow requesting approval
workflow transfer_funds
    requires: HasCapability(transfer)
{
    let amount = 10000;
    
    -- Yield to manager role for approval
    yield role(manager) TransferRequest { amount: amount }
    resume response : TransferResponse {
        Approved(sig) => {
            act transfer with { amount: amount, signature: sig };
            ret Success;
        },
        Denied(reason) => {
            ret Failure(reason);
        }
    }
}

-- Proxy handling manager approvals
proxy manager_proxy
    handles role(manager)
    receives requests:approval_request
{
    loop {
        receive {
            req : TransferRequest from origin => {
                -- Present to human manager via UI
                let decision = await_human_decision(req);
                
                match decision {
                    Approved => resume Approved(manager_sig) : TransferResponse,
                    Denied(r) => resume Denied(r) : TransferResponse
                }
            }
        }
    }
}
```

### 5.2 Quorum Decision

```ash
proxy board_proxy
    handles role(board_members)
    receives requests:board_decision
{
    loop {
        receive {
            req : DecisionRequest => {
                -- Collect votes from all board members
                par {
                    yield role(m1) req resume v1 => { v1 },
                    yield role(m2) req resume v2 => { v2 },
                    yield role(m3) req resume v3 => { v3 }
                } collect votes;
                
                -- Count approvals
                let approvals = count_approvals(votes);
                
                if approvals >= 2 {
                    resume Approved(consensus_sig) : DecisionResponse;
                } else {
                    resume Denied("insufficient_approvals") : DecisionResponse;
                }
            }
        }
    }
}
```

### 5.3 AI Agent Proxy

```ash
proxy ai_analyzer_proxy
    handles role(ai_analyzer)
    receives requests:analysis_request
{
    loop {
        receive {
            req : AnalysisRequest => {
                -- Call external AI service
                let result = act ai_service::analyze with req.data;
                
                match result {
                    Ok(analysis) => resume Success(analysis) : AnalysisResponse,
                    Err(e) => resume Failure(e) : AnalysisResponse
                }
            }
        }
    }
}
```

---

## 6. Operational Semantics

### 6.1 Yield Execution

```rust
pub fn eval_yield(
    &mut self,
    role_name: &Name,
    request: &Value,
    expected_response_type: &Type,
    correlation_id: CorrelationId,
) -> EvalResult<YieldState> {
    // 1. Look up proxy for role
    let proxy_addr = self.proxy_registry
        .lookup(role_name)
        .ok_or(EvalError::NoProxyForRole(role_name.clone()))?;
    
    // 2. Construct message with correlation ID
    let message = Message {
        correlation_id,
        payload: request.clone(),
        origin: self.current_workflow_addr(),
    };
    
    // 3. Send to proxy
    self.send_message(proxy_addr, message)?;
    
    // 4. Suspend workflow
    Ok(YieldState::Suspended {
        correlation_id,
        expected_response_type: expected_response_type.clone(),
    })
}
```

### 6.2 Resume Execution

```rust
pub fn eval_resume(
    &mut self,
    response: &Value,
    response_type: &Type,
    correlation_id: CorrelationId,
) -> EvalResult<()> {
    // Find suspended workflow
    let suspended = self.suspended_yields
        .get(&correlation_id)
        .ok_or(EvalError::UnknownCorrelationId(correlation_id))?;
    
    // Validate response type
    self.type_check(response, &suspended.expected_response_type)?;
    
    // Send response to suspended workflow's mailbox
    let response_message = Message {
        correlation_id,
        payload: response.clone(),
        origin: self.current_workflow_addr(),
    };
    
    self.send_message(suspended.workflow_addr, response_message)?;
    
    // Remove from suspended yields
    self.suspended_yields.remove(&correlation_id);
    
    Ok(())
}
```

### 6.3 Resumption in Original Workflow

```rust
pub fn handle_resumption(
    &mut self,
    message: Message,
) -> EvalResult<Value> {
    // Verify correlation ID matches our suspension
    let state = self.current_yield_state()
        .ok_or(EvalError::NotAwaitingYield)?;
    
    if message.correlation_id != state.correlation_id {
        return Err(EvalError::CorrelationMismatch);
    }
    
    // Type check response
    self.type_check(&message.payload, &state.expected_response_type)?;
    
    // Continue workflow with response value
    Ok(message.payload)
}
```

---

## 7. Type Safety

### 7.1 Request/Response Type Matching

The compiler ensures type safety:

```ash
yield role(manager) TransferRequest { amount: 100 }  -- Request type: TransferRequest
resume response : TransferResponse {                   -- Response type: TransferResponse
    Approved(sig) => ...,                              -- Pattern match on response
    Denied(reason) => ...
}
```

Type checking rules:
- Request value must match role's expected request type
- Response patterns must exhaustively cover response type
- Response from proxy must match declared response type

### 7.2 Protocol Definition

Define request/response types as ADTs:

```ash
type TransferRequest = { amount: Int, from: Account, to: Account };

type TransferResponse =
    | Approved { signature: Signature }
    | Denied { reason: String };
```

---

## 8. Error Handling

### 8.1 Error Types

```rust
pub enum ProxyError {
    NoProxyForRole(RoleName),
    UnknownCorrelationId(CorrelationId),
    CorrelationMismatch,
    TypeMismatch { expected: Type, actual: Type },
    ProxyTimeout { role: RoleName, duration: Duration },
    ProxyCrashed { role: RoleName },
}
```

### 8.2 Timeout Handling

```ash
yield role(manager) req resume response : Response {
    Approved => { ... },
    Denied => { ... }
}
timeout 5m {
    -- Escalate on timeout
    yield role(admin) Escalation { original: req }
    resume escalated : EscalationResponse { ... }
}
```

### 8.3 Proxy Failure

If a proxy crashes:
1. Runtime detects proxy failure
2. Suspended yields for that proxy fail with `ProxyCrashed`
3. Original workflows can catch and handle error
4. Backup proxy can be registered if available

---

## 9. Audit Trail

### 9.1 Events

```rust
pub enum ProxyEvent {
    YieldInitiated {
        correlation_id: CorrelationId,
        from_workflow: InstanceAddr,
        to_role: RoleName,
        request_type: Type,
    },
    YieldResumed {
        correlation_id: CorrelationId,
        response_type: Type,
        duration: Duration,
    },
    ProxyRegistered {
        role: RoleName,
        proxy: InstanceAddr,
    },
    ProxyUnregistered {
        role: RoleName,
        proxy: InstanceAddr,
    },
    ProxyTimeout {
        correlation_id: CorrelationId,
        role: RoleName,
    },
}
```

### 9.2 Initiative Path

The audit trail records the complete initiative path:

```
[10:00:00] workflow:payment_processor yields to role:manager (cid: 0x1234)
[10:00:01] proxy:manager_proxy receives request (cid: 0x1234)
[10:00:05] proxy:manager_proxy yields to role:user_1 (cid: 0x5678)
[10:00:30] role:user_1 resumes proxy:manager_proxy (cid: 0x5678)
[10:00:31] proxy:manager_proxy resumes workflow:payment_processor (cid: 0x1234)
```

---

## 10. Implementation Tasks

- **TASK-238**: SPEC-023 Proxy Workflows (this spec) - **Draft**
- **TASK-239**: Implement proxy workflow runtime - Pending
  - Proxy workflow AST and parser
  - Role-to-proxy registry
  - Yield/resume execution
  - Correlation ID management
  - Quorum pattern support
  - Timeout handling
  - Comprehensive tests

---

## 11. Relationship to Other Specifications

- **SPEC-019**: Role Runtime Semantics - Proxy workflows handle role messages
- **DECISION-237**: Obligation syntax - Role-bound obligations discharged in proxies
- **SPEC-022**: Workflow obligations - Local obligations continue to work
- **SPEC-006**: Policy definitions - Policies apply to proxy workflows

---

## 12. Open Questions

1. **Multiple proxies per role?** Load balancing vs single point of failure.
2. **Proxy supervision?** Who monitors proxy health and restarts?
3. **Static vs dynamic proxy registration?** Register at compile time or runtime?
4. **Cross-node proxies?** For distributed Ash runtime.

---

## 13. Future Work

- **Yield/Resume composition**: Nested yields across multiple proxies
- **Async yield**: Non-blocking yield with callback pattern
- **Proxy chains**: A → B → C proxy delegation
- **Generic proxies**: Proxy that handles multiple roles
