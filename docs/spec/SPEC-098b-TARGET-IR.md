---
id: spec.ash.ir.target
title: Ash Intermediate Representation — Target State
description: Target IR with unified effect rows, effect item identities, and a shared computation substrate
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-18
verified_against:
  specs:
    - docs/spec/SPEC-095a-CURRENT-GRAMMAR.md
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-096a-CURRENT-EFFECT-SYSTEM.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097a-CURRENT-TYPE-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098a-CURRENT-IR.md
---

# SPEC-098b: Ash Intermediate Representation — Target State

**Status:** Draft — target IR for unified effect rows
**Scope:** This document defines the IR representation we want Ash to have.
It is a goal-state living document that will be refined as implementation progresses.
**Depends on:** SPEC-095b (Target Grammar), SPEC-096b (Target Effect System), SPEC-097b (Target Type System)

## 1. Summary

The target IR unifies Ash's computation representation into one substrate with effect-row
annotations. The key changes:

1. Add `EffectRow` to function types and computation expressions.
2. Replace separate `Act`, `Proc`, and `Workflow` AST variants with a unified `Computation` type
   carrying an effect row.
3. Add effect item identities and namespaces.
4. Add contract effect nodes for static/evidence/dynamic discharge tracking.
5. Add handler stack representation for effect dispatch.
6. Preserve backward compatibility during migration.

## 2. Target AST Types

### 2.1 Expression AST

```rust
pub enum Expr {
    -- ... existing variants ...

    -- Unified computation with effect row
    Computation {
        row: EffectRow,
        body: Vec<Stmt>,
    },

    -- Effect handler boundary
    Handle {
        effect: EffectItem,
        handler: HandlerDef,
        body: Box<Expr>,
    },

    -- Raise an effect
    Raise {
        effect: EffectItem,
        arguments: Vec<Expr>,
    },

    -- Legacy compatibility aliases
    Act { ... },    -- lowered to Computation with Act profile row
    Do { ... },     -- lowered to Computation with inferred/target row
    Proc { ... },   -- lowered to Computation with Proc profile row
    Workflow { ... }, -- lowered to Computation with Workflow profile row
}
```

### 2.2 Statement AST

```rust
pub enum Stmt {
    Let { pattern, value },
    Bind { name, computation },
    Return { expr },
    Expr { expr },
    Handle { effect, handler },
}
```

### 2.3 Type AST

```rust
pub enum Type {
    -- ... existing variants ...

    -- Effect row type
    EffectRow {
        items: Vec<EffectItem>,
        tail: Option<RowVar>,
    },

    -- Function type with effect row
    Fn {
        params: Vec<Type>,
        ret: Box<Type>,
        row: EffectRow,
    },

    -- Legacy compatibility aliases
    Fun { params, ret, effect },  -- lowered to Fn with EffectRow
}
```

## 3. Target Value Types

```rust
pub enum Value {
    -- ... existing variants ...

    -- Unified computation value
    Computation {
        row: EffectRow,
        state: ComputationState,
    },

    -- Handler stack
    HandlerStack(Vec<Handler>),

    -- Legacy compatibility aliases
    ActEnvToken(...),  -- preserved during migration
    Proc(...),         -- preserved during migration
}
```

## 4. Effect Row Representation

### 4.1 Row Carrier

```rust
pub struct EffectRow {
    pub items: Vec<EffectItem>,
    pub tail: Option<RowVar>,
}

pub struct RowVar {
    pub name: Name,
    pub constraints: Vec<RowConstraint>,
}
```

### 4.2 Effect Item Identity

```rust
pub enum EffectItem {
    Capability(CapabilityEffect),
    Resource(ResourceEffect),
    Role(RoleEffect),
    Policy(PolicyEffect),
    Contract(ContractEffect),
    Channel(ChannelEffect),
    Process(ProcessEffect),
    Failure(FailureEffect),
    Evidence(EvidenceEffect),
    Group(EffectGroupRef),
}
```

See SPEC-097b for the full type definitions.

## 5. Lowering Pipeline

### 5.1 Target Lowering

```text
surface AST (with effect rows)
    |
    v
lower.rs -- lowers to unified IR
    |
    v
core AST (with EffectRow)
    |
    v
type checker (with row discharge)
    |
    v
interpreter (with handler stack)
```

### 5.2 Lowering Rules

| Surface | Target IR |
|---------|-----------|
| `do { ... }` | `Expr::Computation { row: inferred, body }` |
| `do:Act { ... }` | `Expr::Computation { row: Act_profile, body }` |
| `do:Proc { ... }` | `Expr::Computation { row: Proc_profile, body }` |
| `do:Workflow { ... }` | `Expr::Computation { row: Workflow_profile, body }` |
| `act { ... }` | `Expr::Computation { row: Act_profile, body }` |
| `workflow { ... }` | `Expr::Computation { row: Workflow_profile, body }` |
| `fn ... -> {row} T` | `Type::Fn { params, ret, row }` |
| `handle E with { ... }` | `Expr::Handle { effect, handler, body }` |
| `raise E` | `Expr::Raise { effect, arguments }` |

## 6. Handler Stack Representation

```rust
pub struct HandlerStack {
    pub handlers: Vec<Handler>,
}

pub struct Handler {
    pub effect: EffectItem,
    pub body: HandlerBody,
}

pub enum HandlerBody {
    Static,      -- discharged by type checker
    Evidence,    -- discharged by proof/test evidence
    Dynamic(Box<dyn Fn(Vec<Value>) -> Value>), -- runtime handler
}
```

## 7. Migration Compatibility

### 7.1 Legacy IR Preservation

During migration, the old `Act`, `Proc`, and `Workflow` AST variants remain in the IR but are
lowered to the unified `Computation` type before type checking and interpretation.

### 7.2 Dual Representation

A conforming implementation may maintain both representations during migration:

```rust
pub enum Expr {
    -- New unified representation
    Computation { ... },
    Handle { ... },
    Raise { ... },

    -- Legacy compatibility (deprecated)
    Act { ... },
    Do { ... },
    Proc { ... },
    Workflow { ... },
}
```

The legacy variants are lowered to `Computation` before semantic analysis.

## 8. Open Decisions

1. Whether the IR uses a single `Computation` type or keeps `Act`/`Proc`/`Workflow` as views.
2. Whether handler stacks are first-class IR values or runtime-only constructs.
3. Whether contract discharge status is stored in the IR or in a separate sidecar.
4. How row variables are represented in the IR (names, indices, or de Bruijn indices).
5. Whether effect aliases are expanded during lowering or preserved for diagnostics.

## 9. See Also

- [SPEC-098a: Current IR](SPEC-098a-CURRENT-IR.md) — what the IR looks like today
- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)

## 10. Changelog

- 2026-06-18: Created as target-state IR document. Defined unified `Computation` type, effect row representation, handler stack, and lowering rules.
