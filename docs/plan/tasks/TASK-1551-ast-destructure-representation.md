# TASK-1551: AST Destructure Representation

## Status: 📝 Planned

## Description

Add AST representation for `let` destructuring. The AST must distinguish between record destructuring (field names) and tuple destructuring (positions).

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)
- [TASK-1550](TASK-1550-parser-let-destructors.md) — Parser dependency

## Acceptance Criteria

- [ ] AST has `LetDestructure` or equivalent node
- [ ] Record destructure: list of (field_name, variable_name) pairs
- [ ] Tuple destructure: list of variable names by position
- [ ] Explicit renaming supported: `field: var`
- [ ] AST can be lowered to field access expressions

## Proposed AST

```rust
pub enum Stmt {
    // ... existing variants
    LetDestructure {
        pattern: DestructurePattern,
        value: Box<Expr>,
    },
}

pub enum DestructurePattern {
    Record {
        fields: Vec<(String, Option<String>)>,  // (field_name, var_name) - var_name is None for same name
    },
    Tuple {
        elements: Vec<String>,  // variable names by position
    },
}
```

## Lowering

`LetDestructure { pattern: Record { fields: [("gen", None), ("shrink", None)] }, value: strategy }` lowers to:

```rust
vec![
    Stmt::Let { name: "gen", value: Expr::FieldAccess { object: strategy, field: "gen" } },
    Stmt::Let { name: "shrink", value: Expr::FieldAccess { object: strategy, field: "shrink" } },
]
```

## Verification

- `cargo test -p ash-core` passes (AST tests)
- New AST tests for destructure nodes pass
- Lowering tests verify correct field access generation
