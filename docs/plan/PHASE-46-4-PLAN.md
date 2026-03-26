# Phase 46.4: Agent Harness Implementation Plan

## Overview

Phase 46.4 implements the **Agent Harness** feature - an optional sub-phase providing MCP (Model Context Protocol) integration for AI agent workflows. The agent harness mediates between the Ash runtime and external LLMs, enabling controlled, permission-based AI agent interactions.

### Feature Capabilities

The agent harness provides four core operations:
1. **project_context** - Extract relevant project context for agent consumption
2. **delegate_to_agent** - Send tasks to external LLM agents via MCP
3. **validate_response** - Validate agent responses against expected schemas
4. **accept_response** - Accept validated responses into the workflow

### Permission Model

```ash
capability agent_harness {
    effect: Deliberative,  -- Read/propose, not execute
    permissions: {
        project_context: bool,   -- Access workflow context
        delegate_to_agent: bool, -- Send tasks to LLMs
        validate_response: bool, -- Validate responses
        accept_response: bool    -- Accept into workflow (requires approval by default)
    }
}
```

---

## Task Breakdown

### TASK-268: Define Agent Harness Capability

**Objective:** Define `agent_harness` capability type for LLM agent integration

**Estimated Hours:** 4

**Files:**
- Create: `crates/ash-core/src/capabilities/agent_harness.rs`
- Modify: `crates/ash-core/src/capabilities/mod.rs`
- Test: `crates/ash-core/tests/agent_harness_tests.rs`

**Implementation Steps:**
1. Create capability definition with AgentHarnessCapability struct
2. Define AgentHarnessOperation enum (4 operations)
3. Create AgentHarnessConfig with ProjectionPolicy and AcceptanceMode
4. Add to capabilities module
5. Write tests for permissions and configuration
6. Codex verification

**Key Components:**
- `AgentHarnessCapability` - 4 permission flags with default(), full(), read_only() constructors
- `AgentHarnessOperation` - Enum covering all 4 operations with required_permission() method
- `AgentHarnessConfig` - Configuration with projection_policy, acceptance_mode, max_retries, timeout_ms
- `ProjectionPolicy` - FullContext, ObligationsVisible, Minimal
- `AcceptanceMode` - Automatic, Conditional, HumanReview

**Test Requirements:**
- Default permissions test (accept_response denied by default)
- Full permissions test (all allowed)
- Operation permission mapping test
- Configuration defaults test

**Dependencies:** Phase 46.3 (optional start)
**Blocks:** TASK-269

---

### TASK-269: Implement Harness Workflow Pattern

**Objective:** Implement the agent harness workflow pattern for LLM integration

**Estimated Hours:** 12

**Files:**
- Create: `crates/ash-engine/src/harness.rs`
- Test: `crates/ash-engine/tests/harness_tests.rs`
- Modify: `crates/ash-engine/src/lib.rs`

**Implementation Steps:**
1. Create AgentHarness struct with builder pattern
2. Implement project_context operation with projection policies
3. Implement delegate_to_agent (async, requires MCP provider)
4. Implement validate_response with schema matching
5. Implement accept_response with acceptance modes
6. Define HarnessError enum
7. Write tests for permissions, validation, projection
8. Codex verification

**Key Components:**
- `AgentHarness` - Core struct with config, optional MCP provider, capability
- `project_context()` - Context extraction based on ProjectionPolicy
- `delegate_to_agent()` - Async delegation to LLM via MCP
- `validate_response()` - Type checking against schema
- `accept_response()` - Mode-aware acceptance logic
- `HarnessError` - PermissionDenied, NoMcpProvider, DelegationFailed, ValidationFailed, RequiresHumanApproval

**Test Requirements:**
- Permission checking (read_only can project_context, cannot delegate)
- Response validation against types
- Acceptance mode enforcement (HumanReview requires approval)
- Projection policy correctness (FullContext vs Minimal)

**Dependencies:** TASK-268
**Blocks:** TASK-270

---

### TASK-270: MCP Capability Provider

**Objective:** Implement MCP (Model Context Protocol) capability provider for LLM communication

**Estimated Hours:** 10

**Files:**
- Create: `crates/ash-engine/src/providers/mcp.rs`
- Test: `crates/ash-engine/tests/mcp_provider_tests.rs`
- Modify: `crates/ash-engine/src/providers/mod.rs`

**Implementation Steps:**
1. Create McpProvider with reqwest HTTP client
2. Implement JSON-RPC 2.0 request/response format
3. Add error handling for HTTP, JSON-RPC, network errors
4. Implement initialize, call_tool, get_prompt helpers
5. Implement Provider trait for sync/async bridging
6. Add to provider registry
7. Write tests with wiremock for HTTP mocking
8. Codex verification

**Key Components:**
- `McpProvider` - HTTP client with configurable base_url, timeout
- `McpConfig` - Configuration struct with defaults
- `McpSession` - Server capabilities tracking
- `call()` - Core JSON-RPC method invocation
- `call_tool()` - MCP tools/call wrapper
- `get_prompt()` - MCP prompts/get wrapper
- Provider trait implementation with runtime bridging

**Test Requirements:**
- Successful MCP call with mocked response
- JSON-RPC error response handling
- Tool call with correct request format
- Timeout handling with short timeout

**Dependencies:** TASK-269
**Blocks:** Phase 46 closeout

---

## Implementation Order

```
TASK-268 (4h) -> TASK-269 (12h) -> TASK-270 (10h)
     |                |                |
     v                v                v
[Capability] -> [Workflow] -> [MCP Provider]
   Definition      Pattern        Integration
```

**Total Estimated Hours:** 26 hours

---

## Key Design Decisions

### 1. Deliberative Effect Level
The agent_harness capability has a **Deliberative** effect level, meaning it can read and propose but not execute actions autonomously. This ensures human oversight for AI agent interactions.

### 2. Projection Policies
Three levels of context exposure:
- **FullContext** - Complete workflow state (obligations, bindings, location)
- **ObligationsVisible** - Only obligations and contract requirements (recommended default)
- **Minimal** - No context exposed (Value::Null)

### 3. Acceptance Modes
Three approval workflows:
- **Automatic** - Responses accepted immediately (requires accept_response permission)
- **Conditional** - Responses accepted after validation (default)
- **HumanReview** - Requires explicit human approval

### 4. Default Permission Deny
By default, `accept_response` is **denied**. This ensures workflows cannot autonomously incorporate AI outputs without explicit capability grants.

### 5. Async/Sync Bridging
The Provider trait provides synchronous invoke() while internally using async HTTP calls. This allows the harness to work in both sync and async contexts.

### 6. JSON-RPC 2.0 Protocol
MCP uses standard JSON-RPC 2.0 over HTTP for communication, supporting both tools and prompts capabilities.

---

## Testing Strategy

### Unit Tests (Per Task)

**TASK-268:**
- Permission flag tests (default, full, read_only)
- Operation-to-permission mapping
- Configuration default values
- Enum variant coverage

**TASK-269:**
- Permission denial tests (read_only cannot delegate)
- Projection policy output verification
- Acceptance mode enforcement
- Error variant testing
- Schema validation (mocked)

**TASK-270:**
- HTTP request format verification (wiremock)
- JSON-RPC response parsing
- Error code handling
- Timeout behavior
- Provider trait invoke() bridging

### Integration Tests

- End-to-end harness workflow with mocked MCP
- Permission flow through all 4 operations
- Error propagation from MCP to harness to caller

### Verification Checklist (Per Task)

Each task requires Codex verification:
- Code compiles without warnings (clippy clean)
- Formatting passes (fmt clean)
- All tests pass
- Design review completed
- Async/await correctness verified

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| MCP protocol changes | High | Isolate protocol logic in McpProvider, version configuration |
| Async runtime conflicts | Medium | Use Handle::try_current() with fallback to new Runtime |
| Timeout handling edge cases | Medium | Comprehensive timeout tests with wiremock delays |
| Permission bypass | High | All operations check capability.can() before execution |
| Schema validation complexity | Low | Placeholder matches_schema() for initial implementation |
| External HTTP dependencies | Medium | Mock all HTTP in tests with wiremock |

### Security Considerations

1. **Least Privilege**: Default capability denies accept_response
2. **Context Exposure**: Projection policies limit what agents can see
3. **Human Oversight**: HumanReview mode prevents autonomous acceptance
4. **Timeout Protection**: Configurable timeouts prevent indefinite waits
5. **Error Sanitization**: Error messages don't expose internal state

---

## Deliverables

### New Files (7)
1. `crates/ash-core/src/capabilities/agent_harness.rs`
2. `crates/ash-core/tests/agent_harness_tests.rs`
3. `crates/ash-engine/src/harness.rs`
4. `crates/ash-engine/tests/harness_tests.rs`
5. `crates/ash-engine/src/providers/mcp.rs`
6. `crates/ash-engine/tests/mcp_provider_tests.rs`

### Modified Files (3)
1. `crates/ash-core/src/capabilities/mod.rs`
2. `crates/ash-engine/src/lib.rs`
3. `crates/ash-engine/src/providers/mod.rs`

### Documentation
1. `docs/plan/PHASE-46-4-PLAN.md` (this file)
2. CHANGELOG.md entries (per task)

---

## Dependencies Required

### Cargo Dependencies
- `reqwest` - HTTP client for MCP (already in ash-engine)
- `serde_json` - JSON handling (already present)
- `wiremock` - Test HTTP mocking (dev dependency)
- `tokio` - Async runtime (already present)

### External Services
- MCP-compatible LLM server (for production use)
- No external services required for testing (mocked)

---

## Success Criteria

1. All 3 tasks completed with passing tests
2. Codex verification passed for each task
3. No clippy warnings, formatting clean
4. CHANGELOG.md updated
5. Integration test: harness -> MCP -> mock response flow works

---

## Notes for Sub-Agents

1. **TASK-268** is foundational - must be completed first and verified before proceeding
2. **TASK-269** uses types from TASK-268 (AgentHarnessCapability, Config, etc.)
3. **TASK-270** is called by TASK-269's delegate_to_agent() method
4. All tasks require Codex verification - do not skip
5. Tests should be written first (failing), then implementation
6. Use wiremock for HTTP testing, not real network calls
7. Keep error messages actionable and clear

---

*Plan created: Phase 46.4 Agent Harness*
*Total estimated effort: 26 hours*
*Status: Ready for implementation*
