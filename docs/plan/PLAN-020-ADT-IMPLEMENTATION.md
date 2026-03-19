# Phase 17: Algebraic Data Types Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use task-development-using-tdd to implement each task following TDD + Sub-Agent workflow from AGENTS.md.

**Goal:** Implement full Algebraic Data Types (ADTs) for Ash including sum types, product types, generics, pattern matching with exhaustiveness checking, and Option/Result standard library.

**Architecture:** Extend existing type system in `ash-typeck`, add runtime values in `ash-core`, implement parser support in `ash-parser`, and add evaluation in `ash-interp`. Follow SPEC-020 operational semantics.

**Tech Stack:** Rust 2024, winnow parser, proptest for property testing, thiserror for errors.

---

## Phase 17 Task Status Overview

### Already Complete (from previous work)

| Task | Description | Status |
|------|-------------|--------|
| TASK-121a | Type::Sum, Type::Struct, Type::Constructor | ✅ Complete |
| TASK-121b | Variant struct, TypeVar, Substitution | ✅ Complete |
| TASK-122 | Value::Variant, Value::Struct, Value::Tuple | ✅ Complete |
| TASK-123 | ADT Unification (constructor unification, occurs_in) | ✅ Complete |
| TASK-120 | AST Extensions for ADTs | ✅ Complete |
| TASK-124 | Parse Type Definitions | ✅ Complete |
| TASK-125 | Parse Match Expressions | ✅ Complete |
| TASK-126 | Parse If-Let Expressions | ✅ Complete |
| TASK-127 | Type Check Constructors | ✅ Complete |
| TASK-128 | Type Check Patterns | ✅ Complete |
| TASK-129 | Generic Type Instantiation | ✅ Complete |

### Discovery: TASK-123 Already Implemented

Upon review, constructor unification is **already implemented** in `crates/ash-typeck/src/types.rs`:

```rust
// Constructor unification - names must match, args must unify
(Type::Constructor { name: n1, args: a1 }, Type::Constructor { name: n2, args: a2 }) => {
    if n1 != n2 { return Err(UnifyError::Mismatch(t1.clone(), t2.clone())); }
    // ... unifies args recursively
}
```

The `occurs_in` function also handles `Type::Constructor`, `Type::Sum`, and `Type::Struct`.

### Missing Prerequisites

| Missing Component | Location | Blocking |
|-------------------|----------|----------|
| Pattern::Variant | ast.rs | TASK-125, TASK-128, TASK-132 |
| Expr::Constructor | ast.rs | TASK-124, TASK-127, TASK-131 |
| Expr::Match, Expr::IfLet | ast.rs | TASK-125, TASK-126, TASK-133 |
| MatchArm struct | ast.rs | TASK-125, TASK-128 |
| TypeDef, TypeBody, VariantDef | ast.rs | TASK-124, TASK-127 |
| Visibility enum | ast.rs | TASK-124 |
| Type::Instance, Type::InstanceAddr, Type::ControlLink | types.rs | TASK-134 |

---

## Phase 17 Task Dependencies (Corrected)

```
Phase 17 Tasks (16 total, 3 complete, 13 to implement)

ALREADY COMPLETE:
├── TASK-121a: ADT Type Representations ✅
├── TASK-121b: TypeVar, Variant, Substitution ✅
├── TASK-122: ADT Runtime Values ✅
└── TASK-123: ADT Unification ✅ (already implemented)

Phase 0: AST Extensions (NEW - Prerequisite for all)
└── TASK-120: AST Extensions for ADTs (was part of 121)
    ├── Add Pattern::Variant
    ├── Add Expr::Constructor, Expr::Match, Expr::IfLet
    ├── Add MatchArm, TypeDef, TypeBody, VariantDef
    ├── Add Visibility enum
    └── Add Type::Instance, Type::InstanceAddr, Type::ControlLink

Phase 1: Parser Foundation
├── TASK-124: Parse Type Definitions (depends: 120)
│   └── Blocks: 125, 126, 127
└── TASK-125: Parse Match Expressions (depends: 120, 124)
    └── Blocks: 128, 132, 133

Phase 2: Parser Sugar
└── TASK-126: Parse If-Let (depends: 125)
    └── Blocks: 133

Phase 3: Type Checker Core
├── TASK-129: Generic Instantiation (depends: 120)
│   └── Blocks: 127, 128, 130
├── TASK-127: Type Check Constructors (depends: 120, 124)
│   └── Blocks: 128, 131, 134, 136
└── TASK-128: Type Check Patterns (depends: 120, 125, 127)
    └── Blocks: 132

Phase 4: Type Checker Advanced
└── TASK-130: Exhaustiveness Checking (depends: 128, 129)
    └── Blocks: none

Phase 5: Interpreter Core
├── TASK-131: Constructor Evaluation (depends: 122, 127)
│   └── Blocks: 132
└── TASK-132: Pattern Matching Engine (depends: 128, 131)
    └── Blocks: 133

Phase 6: Interpreter Sugar
└── TASK-133: Match Evaluation (depends: 126, 132)
    └── Blocks: none

Phase 7: Control Link Integration
├── TASK-134: Spawn Option ControlLink (depends: 120, 127)
│   └── Blocks: 135, 136
└── TASK-135: Control Link Transfer (depends: 134)
    └── Blocks: none

Phase 8: Standard Library
└── TASK-136: Option/Result Library (depends: 127, 128, 134)
    └── Blocks: none
```

---

## Implementation Order (Corrected)

### Wave 0: AST Foundation (COMPLETE ✅)
**TASK-120**: AST Extensions for ADTs ✅
- Add `Pattern::Variant` to `crates/ash-core/src/ast.rs`
- Add `Expr::Constructor`, `Expr::Match`, `Expr::IfLet` to `crates/ash-core/src/ast.rs`
- Add `MatchArm`, `TypeDef`, `TypeBody`, `VariantDef`, `Visibility` to `crates/ash-core/src/ast.rs`
- Add `Type::Instance`, `Type::InstanceAddr`, `Type::ControlLink` to `crates/ash-typeck/src/types.rs`

### Wave 1: Parser (COMPLETE ✅)
1. **TASK-124**: Parse Type Definitions ✅
2. **TASK-125**: Parse Match Expressions ✅
3. **TASK-126**: Parse If-Let ✅

### Wave 2: Type Checker Core (COMPLETE ✅)
4. **TASK-129**: Generic Instantiation ✅
5. **TASK-127**: Type Check Constructors ✅
6. **TASK-128**: Type Check Patterns ✅

### Wave 3: Type Checker Advanced (Sequential)
7. **TASK-130**: Exhaustiveness Checking (after 128, 129)

### Wave 4: Interpreter (Sequential)
8. **TASK-131**: Constructor Evaluation (after 127)
9. **TASK-132**: Pattern Matching Engine (after 128, 131)
10. **TASK-133**: Match Evaluation (after 126, 132)

### Wave 5: Control Link (Sequential)
11. **TASK-134**: Spawn Option ControlLink (after 120, 127)
12. **TASK-135**: Control Link Transfer (after 134)

### Wave 6: Standard Library (Sequential)
13. **TASK-136**: Option/Result Library (after 127, 128, 134)

---

## Task Details

### TASK-120: AST Extensions for ADTs

**Status:** 🟡 Ready to Start (Prerequisite for all other tasks)

**Files:**
- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-typeck/src/types.rs`

**Description:**
Add missing AST nodes and type variants that are prerequisites for all other ADT tasks. This was partially included in TASK-121 but not completed.

**TDD Steps:**

**Step 1: Write failing compilation tests**

Create `crates/ash-core/src/ast_adt.rs` (or extend ast.rs):

```rust
//! ADT-related AST extensions

use serde::{Deserialize, Serialize};

/// Type definition in source code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: Name,
    pub params: Vec<TypeVar>,
    pub body: TypeBody,
    pub visibility: Visibility,
}

/// Body of a type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeBody {
    /// type Point = { x: Int, y: Int }
    Struct(Vec<(Name, TypeExpr)>),
    
    /// type Status = Pending | Processing { ... }
    Enum(Vec<VariantDef>),
    
    /// type Name = String
    Alias(TypeExpr),
}

/// Variant definition for enums
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantDef {
    pub name: Name,
    pub fields: Vec<(Name, TypeExpr)>,
}

/// Visibility modifier
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Crate,
    Private,
}

/// Surface syntax type expression (to be resolved)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    Named(Name),
    Constructor { name: Name, args: Vec<TypeExpr> },
    Tuple(Vec<TypeExpr>),
    Record(Vec<(Name, TypeExpr)>),
}

/// Match arm: pattern => expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}
```

Extend `Pattern` enum:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Variable(Name),
    Tuple(Vec<Pattern>),
    Record(Vec<(Name, Pattern)>),
    List(Vec<Pattern>, Option<Name>),
    Wildcard,
    Literal(Value),
    
    /// Variant pattern: Some { value: x } or just Some (unit variant)
    Variant {
        name: Name,
        fields: Option<Vec<(Name, Pattern)>>,
    },
}
```

Extend `Expr` enum:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    // Existing variants...
    Literal(Value),
    Variable(Name),
    
    /// Constructor expression: Some { value: 42 }
    Constructor {
        name: Name,
        fields: Vec<(Name, Expr)>,
    },
    
    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    
    /// If-let expression (sugar for match)
    IfLet {
        pattern: Pattern,
        expr: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    
    // ... rest
}
```

Extend `Type` enum in `crates/ash-typeck/src/types.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Existing variants...
    
    /// Instance type (composite of addr + control link)
    Instance {
        workflow_type: Box<str>,
    },
    
    /// Opaque instance address
    InstanceAddr {
        workflow_type: Box<str>,
    },
    
    /// Control link (affine - must be used exactly once)
    ControlLink {
        workflow_type: Box<str>,
    },
}
```

**Step 2: Run tests**

```bash
cargo check -p ash-core
cargo check -p ash-typeck
```

Expected: Should compile with new types.

**Step 3: Commit**

```bash
git add crates/ash-core/src/ast.rs crates/ash-typeck/src/types.rs
git commit -m "feat(ast): ADT AST extensions - Pattern::Variant, Expr::Constructor, TypeDef (TASK-120)"
```

---

[Remaining task details for TASK-124 through TASK-136 remain unchanged from original plan...]

---

## Updated Task Status in PLAN-INDEX

After completing the review, update `docs/plan/PLAN-INDEX.md`:

```markdown
## Phase 17: Algebraic Data Types (Weeks 26-30)

| Task | Description | Spec | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-121](tasks/TASK-121-adt-core-types.md) | ADT core type representations | SPEC-020 | 6 | ✅ Complete (types only) |
| [TASK-122](tasks/TASK-122-adt-runtime-values.md) | Runtime values for ADTs | SPEC-020 | 5 | ✅ Complete |
| [TASK-123](tasks/TASK-123-adt-unification.md) | Unification with constructors | SPEC-020 | 4 | ✅ Complete (already implemented) |
| [TASK-120](tasks/TASK-120-ast-extensions.md) | AST Extensions for ADTs | SPEC-020 | 3 | ✅ Complete |
| [TASK-124](tasks/TASK-124-parse-type-definitions.md) | Parse type definitions | SPEC-020 | 6 | ✅ Complete |
| [TASK-125](tasks/TASK-125-parse-match-expressions.md) | Parse match and patterns | SPEC-020 | 5 | ✅ Complete |
| [TASK-126](tasks/TASK-126-parse-if-let.md) | Parse if-let expressions | SPEC-020 | 3 | ✅ Complete |
| [TASK-127](tasks/TASK-127-type-check-constructors.md) | Type check constructors | SPEC-020 | 6 | ✅ Complete |
| [TASK-128](tasks/TASK-128-type-check-patterns.md) | Type check patterns | SPEC-020 | 8 | ✅ Complete |
| [TASK-129](tasks/TASK-129-generic-instantiation.md) | Generic type instantiation | SPEC-020 | 6 | ✅ Complete |
| [TASK-130](tasks/TASK-130-exhaustiveness-checking.md) | Exhaustiveness checking | SPEC-020 | 8 | 🟡 Ready |
| [TASK-131](tasks/TASK-131-constructor-evaluation.md) | Constructor evaluation | SPEC-020 | 4 | 🟡 Ready |
| [TASK-132](tasks/TASK-132-pattern-matching-engine.md) | Pattern matching engine | SPEC-020 | 6 | 🟡 Ready |
| [TASK-133](tasks/TASK-133-match-evaluation.md) | Match expression evaluation | SPEC-020 | 5 | 🟡 Ready |
| [TASK-134](tasks/TASK-134-spawn-option-control-link.md) | Spawn returns Option<ControlLink> | SPEC-020 | 6 | 🟡 Ready |
| [TASK-135](tasks/TASK-135-control-link-transfer.md) | Control link transfer semantics | SPEC-020 | 5 | 🟡 Ready |
| [TASK-136](tasks/TASK-136-option-result-library.md) | Option and Result modules | SPEC-020 | 6 | 🟡 Ready |

**Phase 17 Deliverable**: Full ADT support with Option<T>, Result<T,E>, pattern matching, and exhaustiveness checking
**Note**: TASK-120 added as prerequisite; TASK-123 was already implemented
```

---

## Quality Gates

### Per Task
- [ ] Property tests written first (RED)
- [ ] Tests fail appropriately
- [ ] Implementation passes tests (GREEN)
- [ ] Code reviewed for simplification opportunities
- [ ] `cargo fmt` passes
- [ ] `cargo clippy --all-targets` passes
- [ ] Documentation comments added
- [ ] CHANGELOG.md updated

### Pre-Implementation Verification
- [ ] Verify TASK-121/122/123 are complete by running: `cargo test -p ash-typeck` and `cargo test -p ash-core`
- [ ] Document any gaps found during verification

---

## References

- SPEC-020: ADT Types Specification
- docs/design/ADT_TYPE_SYSTEM.md
- AGENTS.md: TDD + Sub-Agent workflow
