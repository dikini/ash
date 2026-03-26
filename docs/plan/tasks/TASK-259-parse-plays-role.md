# TASK-259: Parse `plays role(R)` Clause

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Add parser support for `plays role(R)` workflow clause.

**Spec Reference:** SPEC-024 Section 4

**File Locations:**
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Test: `crates/ash-parser/tests/plays_role_tests.rs`

---

## Background

SPEC-024 defines workflow header syntax:
```ash
workflow processor
    plays role(ai_agent)
    capabilities: [network]
{ ... }
```

Need to add `plays role(R)` parsing.

---

## Step 1: Update Surface AST

Modify `crates/ash-parser/src/surface.rs`:

```rust
pub struct WorkflowHeader {
    pub name: Ident,
    pub plays_roles: Vec<RoleRef>,  // NEW
    pub capabilities: Vec<CapabilityDecl>,
    pub handles: Option<HandlerClause>,
}

pub struct RoleRef {
    pub name: Ident,
    pub span: Span,
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-parser/tests/plays_role_tests.rs
use ash_parser::*;

#[test]
fn test_parse_plays_role() {
    let input = r#"
        workflow processor plays role(ai_agent) {}
    "#;
    
    let result = parse_workflow(input);
    assert!(result.is_ok());
    
    let workflow = result.unwrap();
    assert_eq!(workflow.header.plays_roles.len(), 1);
    assert_eq!(workflow.header.plays_roles[0].name, "ai_agent");
}

#[test]
fn test_parse_multiple_roles() {
    let input = r#"
        workflow processor
            plays role(executor)
            plays role(ai_agent)
        {}
    "#;
    
    let result = parse_workflow(input);
    assert!(result.is_ok());
    
    let workflow = result.unwrap();
    assert_eq!(workflow.header.plays_roles.len(), 2);
}

#[test]
fn test_parse_plays_role_with_capabilities() {
    let input = r#"
        workflow processor
            plays role(ai_agent)
            capabilities: [network]
        {}
    "#;
    
    let result = parse_workflow(input);
    assert!(result.is_ok());
    
    let workflow = result.unwrap();
    assert_eq!(workflow.header.plays_roles.len(), 1);
    assert_eq!(workflow.header.capabilities.len(), 1);
}

proptest! {
    #[test]
    fn test_plays_role_roundtrip(role_name in "[a-z_][a-z0-9_]*") {
        let input = format!("workflow w plays role({}) {{}}", role_name);
        let result = parse_workflow(&input);
        assert!(result.is_ok());
    }
}
```

---

## Step 3: Implement Parser

Modify `crates/ash-parser/src/parse_workflow.rs`:

```rust
use winnow::{
    combinator::{opt, preceded, separated0},
    token::literal,
    PResult, Parser,
};

fn parse_workflow_header(input: &mut &str) -> PResult<WorkflowHeader> {
    let name = parse_ident(input)?;
    
    // Parse zero or more "plays role(X)" clauses
    let plays_roles = separated0(
        parse_plays_role_clause,
        ws
    ).parse_next(input)?;
    
    // Parse optional capabilities clause
    let capabilities = opt(parse_capabilities_clause).parse_next(input)?;
    
    // Parse optional handles clause
    let handles = opt(parse_handles_clause).parse_next(input)?;
    
    Ok(WorkflowHeader {
        name,
        plays_roles,
        capabilities: capabilities.unwrap_or_default(),
        handles,
    })
}

fn parse_plays_role_clause(input: &mut &str) -> PResult<RoleRef> {
    let _ = literal("plays").parse_next(input)?;
    ws(input)?;
    let _ = literal("role").parse_next(input)?;
    ws(input)?;
    let _ = literal("(").parse_next(input)?;
    let name = parse_ident(input)?;
    let _ = literal(")").parse_next(input)?;
    
    Ok(RoleRef {
        name,
        span: calculate_span(input),  // however you get span
    })
}
```

---

## Step 4: Update Lowering

Ensure lowering handles plays_roles:

```rust
// crates/ash-parser/src/lower.rs
fn lower_workflow_header(header: &SurfaceWorkflowHeader) -> CoreWorkflowHeader {
    CoreWorkflowHeader {
        name: header.name.clone(),
        plays_roles: header.plays_roles.iter().map(lower_role_ref).collect(),
        capabilities: header.capabilities.iter().map(lower_capability_decl).collect(),
        handles: header.handles.as_ref().map(lower_handles),
    }
}
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-parser plays_role -v
```

---

## Step 6: Commit

```bash
git add crates/ash-parser/src/surface.rs
git add crates/ash-parser/src/parse_workflow.rs
git add crates/ash-parser/src/lower.rs
git add crates/ash-parser/tests/plays_role_tests.rs
git commit -m "feat: parse plays role(R) clause (TASK-259)

- Add plays_roles field to WorkflowHeader AST
- Parse 'plays role(ident)' syntax
- Support multiple role inclusions
- Integration with capabilities clause
- Property tests for valid identifiers
- Lowering to core AST"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-259 implementation"
  context: |
    Files to verify:
    - crates/ash-parser/src/surface.rs (RoleRef, WorkflowHeader)
    - crates/ash-parser/src/parse_workflow.rs (parser)
    - crates/ash-parser/src/lower.rs (lowering)
    - crates/ash-parser/tests/plays_role_tests.rs
    
    Spec reference: SPEC-024 Section 4
    Requirements:
    1. 'plays role(ident)' parses correctly
    2. Multiple 'plays role' clauses supported
    3. Works with capabilities clause
    4. Lowers to core AST
    5. Invalid syntax rejected
    
    Run and report:
    1. cargo test --package ash-parser plays_role
    2. cargo clippy --package ash-parser --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-parser
    4. Test parse error for: 'plays role()'
    5. Test parse error for: 'plays agent' (missing 'role')
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Surface AST updated
- [ ] Failing tests written
- [ ] Parser implemented
- [ ] Lowering updated
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 6
**Blocked by:** TASK-258
**Blocks:** TASK-260 (capabilities parsing)
