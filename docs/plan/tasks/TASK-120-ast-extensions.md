# TASK-120: AST Extensions for ADTs

## Status: ✅ Complete

## Description

Add missing AST nodes and type variants required for all ADT functionality. This task extracts the AST-related work from TASK-121 (which was marked complete but only had type representations done).

## Specification Reference

- SPEC-020: ADT Types - Section 4.2, 5.3, 5.4

## Why This Task Exists

During review of Phase 17, we discovered:
- TASK-121 marked "complete" but only `Type` enum extensions were done
- TASK-122 (values) was complete
- TASK-123 (unification) was already implemented in the codebase
- The AST nodes (`Pattern::Variant`, `Expr::Constructor`, etc.) were **never added**

This task fills that gap as a prerequisite for all remaining Phase 17 tasks.

## Requirements

### 1. Pattern Extensions (`crates/ash-core/src/ast.rs`)

Add `Variant` variant to `Pattern` enum:

```rust
/// Pattern for destructuring
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
        fields: Option<Vec<(Name, Pattern)>>,  // None for unit variants
    },
}
```

### 2. Expression Extensions (`crates/ash-core/src/ast.rs`)

Add to `Expr` enum:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    // Existing variants remain...
    Literal(Value),
    Variable(Name),
    FieldAccess { expr: Box<Expr>, field: Name },
    IndexAccess { expr: Box<Expr>, index: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Call { function: Box<Expr>, arguments: Vec<Expr> },
    List(Vec<Expr>),
    Record(Vec<(Name, Expr)>),
    Tuple(Vec<Expr>),
    
    /// NEW: Constructor expression: Some { value: 42 }
    Constructor {
        name: Name,
        fields: Vec<(Name, Expr)>,
    },
    
    /// NEW: Match expression: match scrutinee { arms... }
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    
    /// NEW: If-let expression: if let pat = expr then { ... } else { ... }
    IfLet {
        pattern: Pattern,
        expr: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}
```

### 3. Match Arm (`crates/ash-core/src/ast.rs`)

Add `MatchArm` struct:

```rust
/// Match arm: pattern => expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}
```

### 4. Type Definition AST (`crates/ash-core/src/ast.rs`)

Add type definition structures:

```rust
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
```

### 5. Type Extensions (`crates/ash-typeck/src/types.rs`)

Add to `Type` enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Existing variants...
    Int, String, Bool, Null, Time, Ref, 
    List(Box<Type>), Record(Vec<(Box<str>, Type)>), 
    Cap { name: Box<str>, effect: Effect },
    Fun(Vec<Type>, Box<Type>, Effect),
    Var(TypeVar),
    Sum { name: Box<str>, type_params: Vec<TypeVar>, variants: Vec<Variant> },
    Struct { name: Box<str>, type_params: Vec<TypeVar>, fields: Vec<(Box<str>, Type)> },
    Constructor { name: Box<str>, args: Vec<Type> },
    
    /// NEW: Instance type (composite of addr + control link)
    Instance {
        workflow_type: Box<str>,
    },
    
    /// NEW: Opaque instance address
    InstanceAddr {
        workflow_type: Box<str>,
    },
    
    /// NEW: Control link (affine - must be used exactly once)
    ControlLink {
        workflow_type: Box<str>,
    },
}
```

## TDD Steps

### Step 1: Create Failing Compilation Test

**File**: `crates/ash-core/src/ast_adt_tests.rs` (new, or in ast.rs)

```rust
#[cfg(test)]
mod adt_ast_tests {
    use super::*;

    #[test]
    fn test_pattern_variant_exists() {
        // This should compile if Pattern::Variant exists
        let _pat = Pattern::Variant {
            name: "Some".to_string(),
            fields: Some(vec![("value".to_string(), Pattern::Variable("x".to_string()))]),
        };
    }

    #[test]
    fn test_expr_constructor_exists() {
        // This should compile if Expr::Constructor exists
        let _expr = Expr::Constructor {
            name: "Some".to_string(),
            fields: vec![("value".to_string(), Expr::Literal(Value::Int(42)))],
        };
    }

    #[test]
    fn test_expr_match_exists() {
        // This should compile if Expr::Match exists
        let _expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable("opt".to_string())),
            arms: vec![],
        };
    }

    #[test]
    fn test_expr_if_let_exists() {
        // This should compile if Expr::IfLet exists
        let _expr = Expr::IfLet {
            pattern: Pattern::Wildcard,
            expr: Box::new(Expr::Variable("x".to_string())),
            then_branch: Box::new(Expr::Literal(Value::Null)),
            else_branch: Box::new(Expr::Literal(Value::Null)),
        };
    }

    #[test]
    fn test_match_arm_exists() {
        // This should compile if MatchArm exists
        let _arm = MatchArm {
            pattern: Pattern::Wildcard,
            body: Expr::Literal(Value::Null),
        };
    }

    #[test]
    fn test_type_def_exists() {
        // This should compile if TypeDef exists
        let _type_def = TypeDef {
            name: "Option".to_string(),
            params: vec!["T".to_string()],
            body: TypeBody::Enum(vec![]),
            visibility: Visibility::Public,
        };
    }

    #[test]
    fn test_visibility_exists() {
        // This should compile if Visibility enum exists
        let _vis = Visibility::Public;
        let _vis = Visibility::Private;
    }
}
```

### Step 2: Run Compilation Check

```bash
cargo check -p ash-core
cargo check -p ash-typeck
```

Expected: FAIL - types don't exist yet.

### Step 3: Implement AST Extensions

Modify `crates/ash-core/src/ast.rs`:
1. Add `Pattern::Variant` to Pattern enum
2. Add `Expr::Constructor`, `Expr::Match`, `Expr::IfLet` to Expr enum
3. Add `MatchArm` struct
4. Add `TypeDef`, `TypeBody`, `VariantDef` structs
5. Add `Visibility` enum
6. Add `TypeExpr` enum

Modify `crates/ash-typeck/src/types.rs`:
1. Add `Type::Instance`
2. Add `Type::InstanceAddr`
3. Add `Type::ControlLink`

### Step 4: Update Pattern.bindings() method

Extend `Pattern::bindings()` and `collect_bindings()` to handle `Variant`:

```rust
fn collect_bindings(&self, result: &mut Vec<Name>) {
    match self {
        // Existing cases...
        
        Pattern::Variant { fields, .. } => {
            if let Some(fields) = fields {
                for (_, p) in fields {
                    p.collect_bindings(result);
                }
            }
        }
        
        // ... rest
    }
}
```

### Step 5: Update Pattern.is_refutable()

Add Variant to refutable check:

```rust
pub fn is_refutable(&self) -> bool {
    match self {
        Pattern::Variable(_) | Pattern::Wildcard => false,
        Pattern::Tuple(_) | Pattern::Record(_) | Pattern::List(_, _) | Pattern::Literal(_) | Pattern::Variant { .. } => {
            true
        }
    }
}
```

### Step 6: Run Tests

```bash
cargo test -p ash-core adt_ast_tests -- --nocapture
cargo check -p ash-typeck
```

Expected: PASS

### Step 7: Property Tests

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_pattern_variant_roundtrip(name in "[A-Z][a-zA-Z0-9]*") {
            let pat = Pattern::Variant {
                name: name.clone(),
                fields: None,
            };
            // Verify we can create and match on it
            assert!(matches!(pat, Pattern::Variant { name: n, .. } if n == name));
        }

        #[test]
        fn prop_expr_constructor_roundtrip(name in "[A-Z][a-zA-Z0-9]*") {
            let expr = Expr::Constructor {
                name: name.clone(),
                fields: vec![],
            };
            assert!(matches!(expr, Expr::Constructor { name: n, .. } if n == name));
        }
    }
}
```

### Step 8: Commit

```bash
git add crates/ash-core/src/ast.rs crates/ash-typeck/src/types.rs
git commit -m "feat(ast): add ADT AST extensions - Pattern::Variant, Expr::Constructor, MatchArm, TypeDef (TASK-120)"
```

## Completion Checklist

- [ ] `Pattern::Variant` added with `name` and `fields`
- [ ] `Expr::Constructor` added with `name` and `fields`
- [ ] `Expr::Match` added with `scrutinee` and `arms`
- [ ] `Expr::IfLet` added with `pattern`, `expr`, `then_branch`, `else_branch`
- [ ] `MatchArm` struct added with `pattern` and `body`
- [ ] `TypeDef` struct added with `name`, `params`, `body`, `visibility`
- [ ] `TypeBody` enum with `Struct`, `Enum`, `Alias` variants
- [ ] `VariantDef` struct with `name` and `fields`
- [ ] `Visibility` enum with `Public`, `Crate`, `Private`
- [ ] `TypeExpr` enum for surface syntax types
- [ ] `Type::Instance` added to type system
- [ ] `Type::InstanceAddr` added to type system
- [ ] `Type::ControlLink` added to type system
- [ ] `Pattern::bindings()` updated for Variant
- [ ] `Pattern::is_refutable()` updated for Variant
- [ ] Unit tests for all new AST nodes
- [ ] Property tests for variant/constructor roundtrip
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [ ] Documentation comments on all new types

## Estimated Effort

3 hours

## Dependencies

None - this is the foundation task.

## Blocked By

Nothing

## Blocks

- TASK-124 (Parse Type Definitions) - needs TypeDef, TypeBody, Visibility
- TASK-125 (Parse Match Expressions) - needs Pattern::Variant, MatchArm
- TASK-126 (Parse If-Let) - needs Expr::IfLet
- TASK-127 (Type Check Constructors) - needs Expr::Constructor
- TASK-128 (Type Check Patterns) - needs Pattern::Variant
- TASK-131 (Constructor Evaluation) - needs Expr::Constructor
- TASK-132 (Pattern Matching Engine) - needs Pattern::Variant
- TASK-133 (Match Evaluation) - needs Expr::Match, Expr::IfLet
- TASK-134 (Spawn Option ControlLink) - needs Type::Instance, Type::InstanceAddr, Type::ControlLink
