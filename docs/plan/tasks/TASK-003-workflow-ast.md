# TASK-003: Core Workflow AST Types

## Status: 🟢 Complete

## Description

Implement the core Workflow AST types with all variants from SPEC-001.

## Specification Reference

- SPEC-001: IR - Section 2.2 Workflow AST

## Requirements

### Functional Requirements

1. `Workflow` enum with all variants:
   - **Epistemic layer**: `Observe { capability, pattern, continuation }`
   - **Deliberative layer**: `Orient { expr, continuation }`, `Propose { action, continuation }`
   - **Evaluative layer**: `Decide { expr, policy, continuation }`, `Check { obligation, continuation }`
   - **Operational layer**: `Act { action, guard, provenance }`, `Oblig { role, workflow }`
   - **Control flow**: `Let { pattern, expr, continuation }`, `If { condition, then_branch, else_branch }`, `Seq { first, second }`, `Par { workflows }`, `ForEach { pattern, collection, body }`, `Ret { expr }`
   - **Modal**: `With { capability, workflow }`, `Maybe { primary, fallback }`, `Must { workflow }`
   - **Terminal**: `Done`

2. Supporting types:
   - `Capability { name, effect, constraints }`
   - `Action { name, arguments }`
   - `Pattern` (separate task - TASK-005)
   - `Expr` (separate task - will be defined here, detailed in typeck)
   - `Guard` (separate task)
   - `Name` type alias for identifiers

3. Derive traits:
   - Debug, Clone, PartialEq
   - Serialize, Deserialize

### Property Requirements (proptest)

```rust
// Workflow well-formedness
// - No cycles in Box<Workflow> (by construction)
// - All names are valid identifiers

// Effect monotonicity
// - Workflow effect level is properly tracked
// - Children don't exceed parent effect
```

## TDD Steps

### Step 1: Define Supporting Types (Green)

```rust
pub type Name = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    pub name: Name,
    pub effect: Effect,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Expr>,
}
```

### Step 2: Define Workflow Enum (Green)

Implement all 17+ variants of Workflow.

### Step 3: Define Expr Enum (Green)

Basic expression types:
```rust
pub enum Expr {
    Literal(Value),
    Variable(Name),
    FieldAccess { expr: Box<Expr>, field: Name },
    IndexAccess { expr: Box<Expr>, index: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Call { func: Name, arguments: Vec<Expr> },
}
```

### Step 4: Add Tests (Green)

- Construction tests for each variant
- Serialization roundtrip tests

### Step 5: Refactor (Refactor)

- Ensure Box usage minimizes enum size
- Review for clone efficiency

## Completion Checklist

- [ ] All Workflow variants defined
- [ ] Supporting types (Capability, Action, Expr)
- [ ] UnaryOp and BinaryOp enums
- [ ] Serde support for all types
- [ ] Construction tests
- [ ] Serialization roundtrip tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [ ] Documentation comments

## Estimated Effort

6 hours (many variants to implement)

## Dependencies

- TASK-001 (Effect)
- TASK-002 (Value)

## Blocked By

- TASK-001
- TASK-002

## Blocks

- TASK-004 (Provenance uses Workflow)
- TASK-005 (Pattern used in Workflow)
- All parser tasks
- All interpreter tasks
