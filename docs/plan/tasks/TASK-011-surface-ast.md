# TASK-011: Surface AST Types

## Status: ✅ Complete

## Description

Define the Surface AST types that represent the parsed surface syntax before lowering to the Core IR. These types mirror the concrete syntax closely.

## Specification Reference

- SPEC-002: Surface Language - Section 3. Grammar
- SPEC-001: IR (for understanding the lowering target)

## Requirements

### AST Structure

The Surface AST represents the parsed source code before desugaring and lowering:

```rust
// Program = definitions + main workflow
pub struct Program {
    pub definitions: Vec<Definition>,
    pub workflow: WorkflowDef,
}

pub enum Definition {
    Capability(CapabilityDef),
    Policy(PolicyDef),
    Role(RoleDef),
    Memory(MemoryDef),
    Datatype(DatatypeDef),
}
```

### Definition Types

```rust
pub struct CapabilityDef {
    pub name: Name,
    pub effect: EffectType,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub constraints: Vec<Constraint>,
    pub span: Span,
}

pub struct PolicyDef {
    pub name: Name,
    pub condition: Expr,
    pub decision: Decision,
    pub span: Span,
}

pub struct RoleDef {
    pub name: Name,
    pub authority: Vec<CapabilityRef>,
    pub obligations: Vec<ObligationRef>,
    pub supervises: Vec<Name>,
    pub span: Span,
}

pub struct WorkflowDef {
    pub name: Name,
    pub body: Workflow,
    pub span: Span,
}
```

### Workflow Types

```rust
pub enum Workflow {
    Observe {
        capability: CapabilityRef,
        binding: Option<Pattern>,
        continuation: Option<Box<Workflow>>,
        span: Span,
    },
    Orient {
        expr: Expr,
        binding: Option<Pattern>,
        continuation: Option<Box<Workflow>>,
        span: Span,
    },
    Propose {
        action: ActionRef,
        binding: Option<Pattern>,
        continuation: Option<Box<Workflow>>,
        span: Span,
    },
    Decide {
        expr: Expr,
        policy: Option<Name>,
        then_branch: Box<Workflow>,
        else_branch: Option<Box<Workflow>>,
        span: Span,
    },
    Check {
        obligation: ObligationRef,
        continuation: Option<Box<Workflow>>,
        span: Span,
    },
    Act {
        action: ActionRef,
        guard: Option<Guard>,
        span: Span,
    },
    Oblige {
        role: Name,
        check: CheckRef,
        continuation: Option<Box<Workflow>>,
        span: Span,
    },
    Let {
        pattern: Pattern,
        expr: Expr,
        continuation: Option<Box<Workflow>>,
        span: Span,
    },
    If {
        condition: Expr,
        then_branch: Box<Workflow>,
        else_branch: Option<Box<Workflow>>,
        span: Span,
    },
    For {
        pattern: Pattern,
        collection: Expr,
        body: Box<Workflow>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Box<Workflow>,
        span: Span,
    },
    Par {
        branches: Vec<Workflow>,
        span: Span,
    },
    With {
        capability: CapabilityRef,
        body: Box<Workflow>,
        span: Span,
    },
    Maybe {
        primary: Box<Workflow>,
        fallback: Box<Workflow>,
        span: Span,
    },
    Must {
        body: Box<Workflow>,
        span: Span,
    },
    Attempt {
        try_body: Box<Workflow>,
        catch_body: Box<Workflow>,
        span: Span,
    },
    Retry {
        body: Box<Workflow>,
        max_attempts: usize,
        span: Span,
    },
    Timeout {
        body: Box<Workflow>,
        duration: Duration,
        span: Span,
    },
    Seq {
        first: Box<Workflow>,
        second: Box<Workflow>,
        span: Span,
    },
    Done {
        span: Span,
    },
}
```

### Expression Types

```rust
pub enum Expr {
    Literal(Literal),
    Variable(Name),
    InputRef(Name),  // $variable
    FieldAccess {
        base: Box<Expr>,
        field: Name,
        span: Span,
    },
    IndexAccess {
        base: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
        span: Span,
    },
    Call {
        func: Name,
        args: Vec<Expr>,
        span: Span,
    },
}

pub enum UnaryOp {
    Not, Neg, Len, Empty,
}

pub enum BinaryOp {
    Add, Sub, Mul, Div,
    And, Or,
    Eq, Neq, Lt, Gt, Leq, Geq,
    In,
}
```

### Supporting Types

```rust
pub enum Pattern {
    Variable(Name),
    Wildcard,
    Tuple(Vec<Pattern>),
    Record(Vec<(Name, Pattern)>),
    List {
        elements: Vec<Pattern>,
        rest: Option<Name>,
    },
    Literal(Literal),
}

pub enum Guard {
    Always,
    Never,
    Pred(Predicate),
    And(Box<Guard>, Box<Guard>),
    Or(Box<Guard>, Box<Guard>),
    Not(Box<Guard>),
}

pub enum Literal {
    Int(i64),
    Float(f64),
    String(Box<str>),
    Bool(bool),
    Null,
}

pub enum EffectType {
    Observe, Read,
    Analyze, Decide,
    Act, Write, External,
}

pub enum Decision {
    Permit,
    Deny,
    RequireApproval { role: Name },
    Escalate,
}
```

## TDD Steps

### Step 1: Define Core Types

Create `crates/ash-parser/src/surface.rs` with all AST types.

### Step 2: Implement Spanned Trait

```rust
pub trait Spanned {
    fn span(&self) -> Span;
}

impl Spanned for Workflow {
    fn span(&self) -> Span {
        match self {
            Workflow::Observe { span, .. } => *span,
            // ... all variants
        }
    }
}
```

### Step 3: Add Display for Debugging

```rust
impl fmt::Display for Workflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Pretty-print for debugging
    }
}
```

### Step 4: Write Construction Tests

```rust
#[test]
fn test_build_observe_workflow() {
    let workflow = Workflow::Observe {
        capability: CapabilityRef {
            name: "read_file".into(),
            args: vec![],
        },
        binding: Some(Pattern::Variable("content".into())),
        continuation: Some(Box::new(Workflow::Done { span: Span::default() })),
        span: Span::default(),
    };
    
    assert!(matches!(workflow, Workflow::Observe { .. }));
}
```

## Completion Checklist

- [ ] All AST types defined according to SPEC-002
- [ ] Spanned trait implemented for all AST nodes
- [ ] Display implemented for debugging
- [ ] Construction tests pass
- [ ] All types derive Clone, Debug, PartialEq
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [ ] Documentation complete with examples

## Self-Review Questions

1. **Completeness**: Does every grammar rule have a corresponding AST node?
2. **Spans**: Is every AST node trackable to source location?
3. **Lowering**: Can each surface node be lowered to core IR?

## Estimated Effort

4 hours

## Dependencies

- TASK-008: Token definitions (uses Span)

## Blocked By

- TASK-008: Token definitions

## Blocks

- TASK-012: Parser core (parses into these types)
- TASK-016: Lowering (lowers from these types)
