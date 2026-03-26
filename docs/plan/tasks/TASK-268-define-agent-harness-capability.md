# TASK-268: Define Agent Harness Capability

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Define `agent_harness` capability type for LLM agent integration.

**Spec Reference:** Agent harness design document

**File Locations:**
- Create: `crates/ash-core/src/capabilities/agent_harness.rs`
- Modify: `crates/ash-core/src/capabilities/mod.rs`

---

## Background

Agent harness capability mediates between Ash runtime and external LLMs:
```ash
capability agent_harness {
    effect: Deliberative,  -- Read/propose, not execute
    permissions: {
        project_context: bool,
        delegate_to_agent: bool,
        validate_response: bool,
        accept_response: bool
    }
}
```

---

## Step 1: Create Capability Definition

Create `crates/ash-core/src/capabilities/agent_harness.rs`:

```rust
use crate::*;

/// Capability for LLM agent harness operations
#[derive(Debug, Clone)]
pub struct AgentHarnessCapability {
    pub project_context: bool,
    pub delegate_to_agent: bool,
    pub validate_response: bool,
    pub accept_response: bool,
}

impl Default for AgentHarnessCapability {
    fn default() -> Self {
        Self {
            project_context: true,
            delegate_to_agent: true,
            validate_response: true,
            accept_response: false,  // Requires human approval by default
        }
    }
}

impl AgentHarnessCapability {
    pub fn full() -> Self {
        Self {
            project_context: true,
            delegate_to_agent: true,
            validate_response: true,
            accept_response: true,
        }
    }
    
    pub fn read_only() -> Self {
        Self {
            project_context: true,
            delegate_to_agent: false,
            validate_response: false,
            accept_response: false,
        }
    }
    
    /// Check if operation is permitted
    pub fn can(&self, operation: &str) -> bool {
        match operation {
            "project_context" => self.project_context,
            "delegate_to_agent" => self.delegate_to_agent,
            "validate_response" => self.validate_response,
            "accept_response" => self.accept_response,
            _ => false,
        }
    }
}

/// Operations available on agent_harness capability
#[derive(Debug, Clone)]
pub enum AgentHarnessOperation {
    /// Extract project context for agent
    ProjectContext { required_view: String },
    
    /// Delegate task to external agent
    DelegateToAgent {
        model: String,
        context: Value,
        timeout_ms: u64,
    },
    
    /// Validate agent response against schema
    ValidateResponse {
        response: Value,
        schema: Type,
    },
    
    /// Accept response into workflow
    AcceptResponse { response: Value },
}

impl AgentHarnessOperation {
    pub fn required_permission(&self) -> &str {
        match self {
            Self::ProjectContext { .. } => "project_context",
            Self::DelegateToAgent { .. } => "delegate_to_agent",
            Self::ValidateResponse { .. } => "validate_response",
            Self::AcceptResponse { .. } => "accept_response",
        }
    }
}

/// Configuration for agent harness
#[derive(Debug, Clone)]
pub struct AgentHarnessConfig {
    pub projection_policy: ProjectionPolicy,
    pub acceptance_mode: AcceptanceMode,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub enum ProjectionPolicy {
    FullContext,
    ObligationsVisible,
    Minimal,
}

#[derive(Debug, Clone)]
pub enum AcceptanceMode {
    Automatic,      // accept_response allowed
    Conditional,    // requires validation
    HumanReview,    // requires human approval
}

impl Default for AgentHarnessConfig {
    fn default() -> Self {
        Self {
            projection_policy: ProjectionPolicy::ObligationsVisible,
            acceptance_mode: AcceptanceMode::Conditional,
            max_retries: 3,
            timeout_ms: 30000,
        }
    }
}
```

---

## Step 2: Add to Capabilities Module

Modify `crates/ash-core/src/capabilities/mod.rs`:

```rust
pub mod agent_harness;

pub use agent_harness::*;
```

---

## Step 3: Write Tests

```rust
// crates/ash-core/tests/agent_harness_tests.rs
use ash_core::capabilities::*;

#[test]
fn test_default_permissions() {
    let cap = AgentHarnessCapability::default();
    
    assert!(cap.can("project_context"));
    assert!(cap.can("delegate_to_agent"));
    assert!(cap.can("validate_response"));
    assert!(!cap.can("accept_response"));  // Default deny
}

#[test]
fn test_full_permissions() {
    let cap = AgentHarnessCapability::full();
    
    assert!(cap.can("project_context"));
    assert!(cap.can("delegate_to_agent"));
    assert!(cap.can("validate_response"));
    assert!(cap.can("accept_response"));
}

#[test]
fn test_operation_permissions() {
    let op = AgentHarnessOperation::DelegateToAgent {
        model: "claude".to_string(),
        context: Value::Null,
        timeout_ms: 30000,
    };
    
    assert_eq!(op.required_permission(), "delegate_to_agent");
}

#[test]
fn test_config_defaults() {
    let config = AgentHarnessConfig::default();
    
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.timeout_ms, 30000);
    matches!(config.acceptance_mode, AcceptanceMode::Conditional);
}
```

---

## Step 4: Run Tests

```bash
cargo test --package ash-core agent_harness -v
```

---

## Step 5: Commit

```bash
git add crates/ash-core/src/capabilities/agent_harness.rs
git add crates/ash-core/src/capabilities/mod.rs
git add crates/ash-core/tests/agent_harness_tests.rs
git commit -m "feat: define agent_harness capability (TASK-268)

- AgentHarnessCapability with 4 permissions
- AgentHarnessOperation enum for operations
- Permission checking (can())
- AgentHarnessConfig for harness configuration
- ProjectionPolicy and AcceptanceMode enums
- Default, full, and read-only capability constructors
- Tests for permissions and configuration"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-268 implementation"
  context: |
    Files to verify:
    - crates/ash-core/src/capabilities/agent_harness.rs
    - crates/ash-core/tests/agent_harness_tests.rs
    - crates/ash-core/src/capabilities/mod.rs
    
    Requirements:
    1. 4 permissions defined correctly
    2. Operation enum covers use cases
    3. Permission checking works
    4. Configuration structured
    5. Default constructors sensible
    6. Tests cover permission cases
    
    Run and report:
    1. cargo test --package ash-core agent_harness
    2. cargo clippy --package ash-core --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-core
    4. Review capability design
    5. Check Deliberative effect level
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Capability struct defined
- [ ] Operations enum defined
- [ ] Failing tests written
- [ ] Permission checking
- [ ] Configuration
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 4
**Blocked by:** Phase 46.3 (optional start)
**Blocks:** TASK-269 (harness workflow)
