# TASK-261: Lower Implicit Default Role Generation

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement lowering that converts `capabilities: [...]` to implicit role.

**Spec Reference:** SPEC-024 Section 5.1

**File Locations:**
- Modify: `crates/ash-parser/src/lower.rs`
- Test: `crates/ash-parser/tests/implicit_role_tests.rs`

---

## Background

SPEC-024 Section 5.1: `capabilities: [...]` desugars to implicit role.

```ash
-- Surface:
workflow X capabilities: [C1, C2] { ... }

-- Lowered:
role X_default { capabilities: [C1, C2] }
workflow X plays role(X_default) { ... }
```

The implicit role name (`X_default`) is generated, not user-visible.

---

## Step 1: Understand Current Lowering

Review workflow lowering:

```bash
grep -n "fn lower_workflow" crates/ash-parser/src/lower.rs
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-parser/tests/implicit_role_tests.rs
use ash_parser::*;
use ash_core::*;

#[test]
fn test_capabilities_creates_implicit_role() {
    let input = r#"
        workflow processor capabilities: [file] {}
    "#;
    
    let surface = parse_module(input).unwrap();
    let core = lower_module(&surface);
    
    // Should have generated role
    let implicit_role = core.roles.iter()
        .find(|r| r.name == "processor_default");
    assert!(implicit_role.is_some());
    
    // Workflow should reference it
    let workflow = core.workflows.iter()
        .find(|w| w.name == "processor")
        .unwrap();
    assert!(workflow.header.plays_roles.iter()
        .any(|r| r == "processor_default"));
}

#[test]
fn test_capabilities_with_constraints_in_role() {
    let input = r#"
        workflow api
            capabilities: [network @ { hosts: ["*.example.com"] }]
        {}
    "#;
    
    let surface = parse_module(input).unwrap();
    let core = lower_module(&surface);
    
    let role = core.roles.iter()
        .find(|r| r.name == "api_default")
        .unwrap();
    
    assert_eq!(role.capabilities.len(), 1);
    // Check constraint preserved
}

#[test]
fn test_explicit_roles_preserved() {
    let input = r#"
        workflow processor
            plays role(ai_agent)
            capabilities: [network]
        {}
    "#;
    
    let surface = parse_module(input).unwrap();
    let core = lower_module(&surface);
    
    let workflow = core.workflows.iter()
        .find(|w| w.name == "processor")
        .unwrap();
    
    // Should have both explicit and implicit roles
    assert!(workflow.header.plays_roles.contains(&"ai_agent".to_string()));
    assert!(workflow.header.plays_roles.contains(&"processor_default".to_string()));
}

#[test]
fn test_no_capabilities_no_implicit_role() {
    let input = r#"
        workflow simple {}
    "#;
    
    let surface = parse_module(input).unwrap();
    let core = lower_module(&surface);
    
    // No roles should be generated
    let implicit = core.roles.iter()
        .find(|r| r.name.ends_with("_default"));
    assert!(implicit.is_none());
}
```

---

## Step 3: Implement Lowering

Modify `crates/ash-parser/src/lower.rs`:

```rust
pub fn lower_module(surface: &SurfaceModule) -> CoreModule {
    let mut core = CoreModule::new();
    
    // First pass: collect explicit roles
    for role in &surface.roles {
        core.roles.push(lower_role(role));
    }
    
    // Second pass: lower workflows, generating implicit roles
    for workflow in &surface.workflows {
        let (workflow, implicit_role) = lower_workflow_with_implicit_role(workflow);
        
        if let Some(role) = implicit_role {
            core.roles.push(role);
        }
        
        core.workflows.push(workflow);
    }
    
    core
}

fn lower_workflow_with_implicit_role(
    surface: &SurfaceWorkflow
) -> (CoreWorkflow, Option<CoreRole>) {
    let mut header = lower_workflow_header(&surface.header);
    
    // Generate implicit role if capabilities declared
    let implicit_role = if !surface.header.capabilities.is_empty() {
        let role_name = format!("{}_default", surface.header.name);
        
        let role = CoreRole {
            name: role_name.clone(),
            capabilities: surface.header.capabilities
                .iter()
                .map(lower_capability_decl)
                .collect(),
            external: false,
            deterministic: true,
            span: surface.span,
        };
        
        // Add implicit role to workflow's plays_roles
        header.plays_roles.push(role_name);
        
        Some(role)
    } else {
        None
    };
    
    let workflow = CoreWorkflow {
        name: surface.header.name.clone(),
        header,
        body: lower_workflow_body(&surface.body),
        span: surface.span,
    };
    
    (workflow, implicit_role)
}
```

---

## Step 4: Handle Name Collisions

Ensure generated role names don't conflict:

```rust
fn generate_implicit_role_name(workflow_name: &str, existing_roles: &[CoreRole]) -> String {
    let base = format!("{}_default", workflow_name);
    
    if !existing_roles.iter().any(|r| r.name == base) {
        return base;
    }
    
    // Generate unique name
    let mut counter = 1;
    loop {
        let name = format!("{}_default_{}", workflow_name, counter);
        if !existing_roles.iter().any(|r| r.name == name) {
            return name;
        }
        counter += 1;
    }
}
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-parser implicit_role -v
```

---

## Step 6: Commit

```bash
git add crates/ash-parser/src/lower.rs
git add crates/ash-parser/tests/implicit_role_tests.rs
git commit -m "feat: lower capabilities to implicit default role (TASK-261)

- Generate implicit role for 'capabilities: [...]' syntax
- Role named '{workflow}_default' (not user-visible)
- Add implicit role to workflow's plays_roles
- Preserve constraints in generated role
- Handle name collision with existing roles
- Works alongside explicit 'plays role' clauses
- All tests for implicit role generation"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-261 implementation"
  context: |
    Files to verify:
    - crates/ash-parser/src/lower.rs (implicit role generation)
    - crates/ash-parser/tests/implicit_role_tests.rs
    
    Spec reference: SPEC-024 Section 5.1
    Requirements:
    1. capabilities: [...] generates implicit role
    2. Role named '{workflow}_default'
    3. Constraints preserved in role
    4. Workflow plays the implicit role
    5. Works with explicit plays role
    6. No role generated if no capabilities
    7. Name collisions handled
    
    Run and report:
    1. cargo test --package ash-parser implicit_role
    2. cargo clippy --package ash-parser --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-parser
    4. Verify lowered output has correct structure
    5. Test name collision scenario
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Lowering logic implemented
- [ ] Failing tests written
- [ ] Implicit role generation works
- [ ] Constraints preserved
- [ ] Name collision handling
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 8
**Blocked by:** TASK-260
**Blocks:** Phase 46.2 (Type System Integration)
