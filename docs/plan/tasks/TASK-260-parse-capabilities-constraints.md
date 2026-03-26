# TASK-260: Parse `capabilities: [...]` with `@` Constraints

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Add parser support for capability declarations with constraint refinement.

**Spec Reference:** SPEC-024 Section 3

**File Locations:**
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Test: `crates/ash-parser/tests/capability_constraint_tests.rs`

---

## Background

SPEC-024 defines capability declaration syntax:
```ash
capabilities: [
    file @ { paths: ["/tmp/*"], read: true },
    network @ { hosts: ["*.example.com"] }
]
```

The `@ { ... }` syntax refines capabilities with constraints.

---

## Step 1: Update Surface AST

Modify `crates/ash-parser/src/surface.rs`:

```rust
pub struct CapabilityDecl {
    pub capability: Ident,
    pub constraints: Option<ConstraintBlock>,
    pub span: Span,
}

pub struct ConstraintBlock {
    pub fields: Vec<ConstraintField>,
    pub span: Span,
}

pub struct ConstraintField {
    pub name: Ident,
    pub value: ConstraintValue,
    pub span: Span,
}

pub enum ConstraintValue {
    Bool(bool),
    Int(i64),
    String(String),
    Array(Vec<ConstraintValue>),
    Object(Vec<(String, ConstraintValue)>),
}
```

---

## Step 2: Write Failing Tests

```rust
// crates/ash-parser/tests/capability_constraint_tests.rs
use ash_parser::*;

#[test]
fn test_parse_capability_without_constraints() {
    let input = "capabilities: [file]";
    let result = parse_capabilities(input);
    assert!(result.is_ok());
    
    let caps = result.unwrap();
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].capability, "file");
    assert!(caps[0].constraints.is_none());
}

#[test]
fn test_parse_capability_with_constraints() {
    let input = r#"capabilities: [
        file @ { paths: ["/tmp/*"], read: true }
    ]"#;
    
    let result = parse_capabilities(input);
    assert!(result.is_ok());
    
    let caps = result.unwrap();
    assert_eq!(caps.len(), 1);
    
    let constraints = caps[0].constraints.as_ref().unwrap();
    assert_eq!(constraints.fields.len(), 2);
}

#[test]
fn test_parse_multiple_capabilities() {
    let input = r#"capabilities: [
        file @ { paths: ["/tmp/*"] },
        network @ { hosts: ["*.example.com"], ports: [443] }
    ]"#;
    
    let result = parse_capabilities(input);
    assert!(result.is_ok());
    
    let caps = result.unwrap();
    assert_eq!(caps.len(), 2);
}

#[test]
fn test_parse_constraint_types() {
    let input = r#"capabilities: [test @ {
        bool_field: true,
        int_field: 42,
        string_field: "hello",
        array_field: [1, 2, 3],
        nested: { inner: "value" }
    }]"#;
    
    let result = parse_capabilities(input);
    assert!(result.is_ok());
}

proptest! {
    #[test]
    fn test_constraint_roundtrip(path in "[/a-z0-9_*]+") {
        let input = format!(
            r#"capabilities: [file @ {{ paths: ["{}"] }}]"#,
            path
        );
        let result = parse_capabilities(&input);
        assert!(result.is_ok());
    }
}
```

---

## Step 3: Implement Parser

Modify `crates/ash-parser/src/parse_workflow.rs`:

```rust
fn parse_capabilities_clause(input: &mut &str) -> PResult<Vec<CapabilityDecl>> {
    let _ = literal("capabilities").parse_next(input)?;
    ws(input)?;
    let _ = literal(":").parse_next(input)?;
    ws(input)?;
    let _ = literal("[").parse_next(input)?;
    ws(input)?;
    
    let decls = separated0(
        parse_capability_decl,
        (ws, literal(","), ws)
    ).parse_next(input)?;
    
    ws(input)?;
    let _ = literal("]").parse_next(input)?;
    
    Ok(decls)
}

fn parse_capability_decl(input: &mut &str) -> PResult<CapabilityDecl> {
    let capability = parse_ident(input)?;
    ws(input)?;
    
    let constraints = opt(parse_constraint_block).parse_next(input)?;
    
    Ok(CapabilityDecl {
        capability,
        constraints,
        span: calculate_span(input),
    })
}

fn parse_constraint_block(input: &mut &str) -> PResult<ConstraintBlock> {
    let _ = literal("@").parse_next(input)?;
    ws(input)?;
    let _ = literal("{").parse_next(input)?;
    ws(input)?;
    
    let fields = separated0(
        parse_constraint_field,
        (ws, literal(","), ws)
    ).parse_next(input)?;
    
    ws(input)?;
    let _ = literal("}").parse_next(input)?;
    
    Ok(ConstraintBlock {
        fields,
        span: calculate_span(input),
    })
}

fn parse_constraint_field(input: &mut &str) -> PResult<ConstraintField> {
    let name = parse_ident(input)?;
    ws(input)?;
    let _ = literal(":").parse_next(input)?;
    ws(input)?;
    let value = parse_constraint_value(input)?;
    
    Ok(ConstraintField {
        name,
        value,
        span: calculate_span(input),
    })
}

fn parse_constraint_value(input: &mut &str) -> PResult<ConstraintValue> {
    // Try bool
    if let Ok(v) = literal("true").parse_next(input) {
        return Ok(ConstraintValue::Bool(true));
    }
    if let Ok(v) = literal("false").parse_next(input) {
        return Ok(ConstraintValue::Bool(false));
    }
    
    // Try number (int)
    if let Ok(n) = parse_i64(input) {
        return Ok(ConstraintValue::Int(n));
    }
    
    // Try string
    if let Ok(s) = parse_string(input) {
        return Ok(ConstraintValue::String(s));
    }
    
    // Try array
    if input.starts_with('[') {
        return parse_constraint_array(input);
    }
    
    // Try object
    if input.starts_with('{') {
        return parse_constraint_object(input);
    }
    
    Err(ErrMode::Backtrack(Error::new(input, ErrorKind::Verify)))
}

fn parse_constraint_array(input: &mut &str) -> PResult<ConstraintValue> {
    let _ = literal("[").parse_next(input)?;
    let values = separated0(
        parse_constraint_value,
        (ws, literal(","), ws)
    ).parse_next(input)?;
    let _ = literal("]").parse_next(input)?;
    
    Ok(ConstraintValue::Array(values))
}
```

---

## Step 4: Update Lowering

```rust
fn lower_capability_decl(decl: &SurfaceCapabilityDecl) -> CoreCapabilityDecl {
    CoreCapabilityDecl {
        capability: decl.capability.clone(),
        constraints: decl.constraints.as_ref().map(lower_constraints),
    }
}

fn lower_constraints(constraints: &SurfaceConstraintBlock) -> CoreConstraints {
    CoreConstraints {
        fields: constraints.fields.iter().map(|f| {
            (f.name.clone(), lower_constraint_value(&f.value))
        }).collect(),
    }
}
```

---

## Step 5: Run Tests

```bash
cargo test --package ash-parser capability_constraint -v
```

---

## Step 6: Commit

```bash
git add crates/ash-parser/src/surface.rs
git add crates/ash-parser/src/parse_workflow.rs
git add crates/ash-parser/src/lower.rs
git add crates/ash-parser/tests/capability_constraint_tests.rs
git commit -m "feat: parse capabilities with @ constraints (TASK-260)

- Add CapabilityDecl, ConstraintBlock, ConstraintField AST nodes
- Parse '@ { field: value }' constraint syntax
- Support bool, int, string, array, object constraint values
- Multiple capabilities in declaration list
- Lowering to core AST with constraint preservation
- Property tests for constraint parsing"
```

---

## Step 7: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-260 implementation"
  context: |
    Files to verify:
    - crates/ash-parser/src/surface.rs (constraint AST)
    - crates/ash-parser/src/parse_workflow.rs (parser)
    - crates/ash-parser/src/lower.rs (lowering)
    - crates/ash-parser/tests/capability_constraint_tests.rs
    
    Spec reference: SPEC-024 Section 3
    Requirements:
    1. 'capability @ { constraints }' parses
    2. All constraint value types supported
    3. Multiple capabilities parse
    4. Optional constraints (bare capability)
    5. Lowers to core AST
    6. Invalid syntax rejected
    
    Run and report:
    1. cargo test --package ash-parser capability
    2. cargo clippy --package ash-parser --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-parser
    4. Test error cases: missing '{', invalid values
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] AST nodes added
- [ ] Failing tests written
- [ ] Parser implemented
- [ ] Lowering updated
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 10
**Blocked by:** TASK-259
**Blocks:** TASK-261 (implicit role lowering)
