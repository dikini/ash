# TASK-061: Policy Definition Syntax Implementation

## Status: ✅ Complete

## Description

Implement static policy definitions in Ash (SPEC-006), allowing users to declare custom policy types with named parameters and constraints that are checked at compile-time using SMT solving.

## Specification Reference

- SPEC-006: Policy Definition Syntax

## Requirements

### 1. Parser Extension

Extend `ash-parser` to support:
- `policy` keyword and definitions
- Policy fields with types and defaults
- `where` clauses for invariants
- `check` statements in workflows

**Files to modify:**
- `crates/ash-parser/src/surface.rs` - Add PolicyDef AST node
- `crates/ash-parser/src/parse_workflow.rs` - Parse policy definitions
- `crates/ash-parser/src/token.rs` - Add `policy` keyword if needed

### 2. Surface AST Types

```rust
// In surface.rs
pub struct PolicyDef {
    pub name: Name,
    pub type_params: Vec<Name>,  // For <T> parameters
    pub fields: Vec<PolicyField>,
    pub where_clause: Option<Expr>,
    pub span: Span,
}

pub struct PolicyField {
    pub name: Name,
    pub ty: Type,
    pub default: Option<Expr>,
    pub span: Span,
}

pub enum Workflow {
    // ... existing variants ...
    Check {
        policy: PolicyInstance,
        span: Span,
    },
}

pub struct PolicyInstance {
    pub name: Name,
    pub fields: Vec<(Name, Expr)>,
    pub span: Span,
}
```

### 3. Type Checking

In `ash-typeck`:
- Validate policy definitions
- Type-check policy instances
- Generate SMT constraints from `where` clauses
- Check for conflicts between policy instances

**Files to modify/create:**
- `crates/ash-typeck/src/policy_check.rs` - New module
- `crates/ash-typeck/src/lib.rs` - Export policy checking

### 4. SMT Integration

Generate SMT constraints from policy definitions:

```rust
// In smt.rs
pub struct PolicyEncoder<'ctx> {
    ctx: &'ctx Context,
}

impl<'ctx> PolicyEncoder<'ctx> {
    pub fn encode_policy_def(&self, def: &PolicyDef) -> Result<(), EncodeError>;
    pub fn encode_policy_instance(&self, instance: &PolicyInstance) -> Vec<Bool<'ctx>>;
}
```

### 5. Core IR Lowering

Lower policy checks to the Core IR:
- Policy checks become `Workflow::Check` nodes
- Policy instances become runtime-checkable values

### 6. Runtime Support

In `ash-interp`:
- Evaluate policy instances at runtime
- Support for policy check failures

## TDD Steps

### Step 1: Parser Tests

```rust
#[test]
fn test_parse_simple_policy() {
    let input = r#"
        policy RateLimit {
            requests: Int,
            window_secs: Int
        }
    "#;
    let result = parse_policy_def(input);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "RateLimit");
}

#[test]
fn test_parse_policy_with_defaults() {
    let input = r#"
        policy RetryPolicy {
            max_attempts: Int = 3,
            backoff_ms: Int = 1000
        }
    "#;
    let result = parse_policy_def(input);
    assert!(result.unwrap().fields[0].default.is_some());
}

#[test]
fn test_parse_policy_where_clause() {
    let input = r#"
        policy BoundedResource {
            min: Int,
            max: Int
        } where {
            min <= max
        }
    "#;
    let result = parse_policy_def(input);
    assert!(result.unwrap().where_clause.is_some());
}

#[test]
fn test_parse_check_statement() {
    let input = r#"
        check RateLimit { requests: 100, window_secs: 60 };
    "#;
    let result = parse_check_stmt(input);
    assert!(result.is_ok());
}
```

### Step 2: Type Checker Tests

```rust
#[test]
fn test_policy_type_checking() {
    let policy_def = PolicyDef {
        name: "RateLimit".into(),
        fields: vec![
            PolicyField { name: "requests".into(), ty: Type::Int, default: None },
        ],
        where_clause: None,
    };
    
    let result = type_check_policy_def(&policy_def);
    assert!(result.is_ok());
}

#[test]
fn test_policy_conflict_detection() {
    let policies = vec![
        PolicyInstance { 
            name: "RateLimit".into(),
            fields: vec![("requests".into(), Expr::Int(100))],
        },
        PolicyInstance {
            name: "RateLimit".into(),
            fields: vec![("requests".into(), Expr::Int(50))],
        },
    ];
    
    let result = check_policy_conflicts(&policies);
    assert!(result.is_err()); // Conflicting rate limits
}
```

### Step 3: SMT Encoding Tests

```rust
#[test]
fn test_encode_budget_policy() {
    let ctx = SmtContext::new();
    let encoder = PolicyEncoder::new(&ctx);
    
    let policy = PolicyInstance {
        name: "Budget".into(),
        fields: vec![("max".into(), Expr::Int(1000))],
    };
    
    let constraints = encoder.encode_policy_instance(&policy);
    assert!(!constraints.is_empty());
}

#[test]
fn test_encode_where_clause() {
    let ctx = SmtContext::new();
    let def = PolicyDef {
        name: "Bounded".into(),
        fields: vec![
            PolicyField { name: "min".into(), ty: Type::Int, default: None },
            PolicyField { name: "max".into(), ty: Type::Int, default: None },
        ],
        where_clause: Some(parse_expr("min <= max")),
    };
    
    let result = ctx.encode_policy_def(&def);
    assert!(result.is_ok());
}
```

## Completion Checklist

- [ ] Parser supports `policy` definitions
- [ ] Parser supports `check` statements
- [ ] Surface AST includes PolicyDef, PolicyField, PolicyInstance
- [ ] Type checker validates policy definitions
- [ ] Type checker validates policy instances
- [ ] SMT encoding for policy definitions
- [ ] SMT encoding for policy instances
- [ ] Conflict detection between policies
- [ ] Where clause encoding
- [ ] Core IR lowering
- [ ] Runtime evaluation
- [ ] 15+ unit tests
- [ ] Integration test with full workflow
- [ ] Documentation with examples
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Dependencies

- TASK-024b: SMT Integration (for conflict detection)
- TASK-011: Surface AST (extend with PolicyDef)
- TASK-012: Parser Core (extend parser)

## Estimated Effort

12 hours (parser + type checker + SMT encoding + tests)

## Blocked By

- TASK-024b (SMT integration)

## Blocks

- TASK-062: Policy Combinators (builds on base policies)
