# TASK-127: Type Check Constructors

## Status: ✅ Complete

## Description

Implement type checking for ADT constructors (e.g., `Some { value: 42 }`, `Point { x: 10, y: 20 }`).

## Specification Reference

- SPEC-020: ADT Types - Section 6.1

## Requirements

Type check constructor expressions:
```ash
let x = Some { value: 42 };           -- Type: Option<Int>
let p = Point { x: 10, y: 20 };       -- Type: Point
let n = None;                          -- Type: Option<T> (inference needed)
```

## TDD Steps

### Step 1: Add Constructor to Expr AST

**File**: `crates/ash-core/src/ast.rs`

Add to `Expr` enum:

```rust
pub enum Expr {
    // Existing...
    
    /// Constructor expression: Some { value: 42 }
    Constructor {
        name: Name,
        fields: Vec<(Name, Expr)>,
    },
}
```

### Step 2: Create Type Environment

**File**: `crates/ash-typeck/src/type_env.rs` (new or extend)

```rust
//! Type environment for tracking type definitions

use ash_core::ast::{TypeDef, VariantDef};
use ash_typeck::types::{Type, TypeVar, Variant};
use std::collections::HashMap;

/// Environment mapping type/constructor names to their definitions
pub struct TypeEnv {
    /// Type definitions by name
    type_defs: HashMap<Box<str>, TypeDef>,
    /// Constructor to type mapping
    constructors: HashMap<Box<str>, (Box<str>, usize)>,  // (type_name, variant_index)
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = Self {
            type_defs: HashMap::new(),
            constructors: HashMap::new(),
        };
        env.add_builtin_types();
        env
    }
    
    fn add_builtin_types(&mut self) {
        // Add Option<T>
        self.register_type(TypeDef {
            name: "Option".into(),
            params: vec![TypeVar(0)],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Some".into(),
                    fields: vec![("value".into(), TypeExpr::Var(TypeVar(0)))],
                },
                VariantDef {
                    name: "None".into(),
                    fields: vec![],
                },
            ]),
            visibility: Visibility::Public,
        });
        
        // Add Result<T, E>
        self.register_type(TypeDef {
            name: "Result".into(),
            params: vec![TypeVar(0), TypeVar(1)],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Ok".into(),
                    fields: vec![("value".into(), TypeExpr::Var(TypeVar(0)))],
                },
                VariantDef {
                    name: "Err".into(),
                    fields: vec![("error".into(), TypeExpr::Var(TypeVar(1)))],
                },
            ]),
            visibility: Visibility::Public,
        });
    }
    
    pub fn register_type(&mut self, def: TypeDef) {
        // Register constructor mappings
        match &def.body {
            TypeBody::Enum(variants) => {
                for (i, variant) in variants.iter().enumerate() {
                    self.constructors.insert(
                        variant.name.clone(),
                        (def.name.clone(), i)
                    );
                }
            }
            TypeBody::Struct(_) => {
                // Struct name is also its constructor
                self.constructors.insert(
                    def.name.clone(),
                    (def.name.clone(), 0)
                );
            }
            _ => {}
        }
        
        self.type_defs.insert(def.name.clone(), def);
    }
    
    pub fn lookup_constructor(&self, name: &str) -> Option<&(Box<str>, usize)> {
        self.constructors.get(name)
    }
    
    pub fn lookup_type(&self, name: &str) -> Option<&TypeDef> {
        self.type_defs.get(name)
    }
}
```

### Step 3: Implement Constructor Type Checking

**File**: `crates/ash-typeck/src/check_expr.rs` (extend)

```rust
use ash_core::ast::Expr;
use ash_typeck::types::{Type, TypeVar, Substitution};

/// Type check an expression
pub fn check_expr(
    env: &TypeEnv,
    var_env: &mut VarEnv,
    expr: &Expr,
) -> Result<(Type, Effect), TypeError> {
    match expr {
        // Existing cases...
        
        Expr::Constructor { name, fields } => {
            check_constructor(env, var_env, name, fields)
        }
    }
}

fn check_constructor(
    env: &TypeEnv,
    var_env: &mut VarEnv,
    name: &str,
    fields: &[(String, Expr)],
) -> Result<(Type, Effect), TypeError> {
    // Look up constructor
    let (type_name, variant_idx) = env.lookup_constructor(name)
        .ok_or_else(|| TypeError::UnknownConstructor(name.to_string()))?;
    
    let type_def = env.lookup_type(type_name)
        .ok_or_else(|| TypeError::UnknownType(type_name.to_string()))?;
    
    // Get expected field types
    let expected_fields = match &type_def.body {
        TypeBody::Enum(variants) => {
            &variants[*variant_idx].fields
        }
        TypeBody::Struct(struct_fields) => struct_fields,
        _ => return Err(TypeError::InvalidConstructor(name.to_string())),
    };
    
    // Build substitution for type parameters (fresh vars initially)
    let mut subst = Substitution::new();
    for param in &type_def.params {
        subst.insert(*param, Type::Var(TypeVar::fresh()));
    }
    
    // Check each field
    let mut total_effect = Effect::Epistemic;
    
    for (field_name, field_expr) in fields {
        // Find expected type for this field
        let expected_ty = expected_fields.iter()
            .find(|(n, _)| n == field_name)
            .map(|(_, ty)| ty)
            .ok_or_else(|| TypeError::UnknownField {
                constructor: name.to_string(),
                field: field_name.to_string(),
            })?;
        
        // Type check the field expression
        let (actual_ty, effect) = check_expr(env, var_env, field_expr)?;
        total_effect = total_effect.join(effect);
        
        // Unify with expected type (applying current substitution)
        let expected_subst = subst.apply(expected_ty);
        let field_subst = unify(&expected_subst, &actual_ty)
            .map_err(|e| TypeError::from(e))?;
        
        // Compose substitutions
        subst = subst.compose(&field_subst);
    }
    
    // Check all required fields are present
    for (field_name, _) in expected_fields {
        if !fields.iter().any(|(n, _)| n == field_name) {
            return Err(TypeError::MissingField {
                constructor: name.to_string(),
                field: field_name.to_string(),
            });
        }
    }
    
    // Build result type by applying substitution to type def
    let result_type = instantiate_type_def(type_def, &subst);
    
    Ok((result_type, total_effect))
}

/// Instantiate a type definition with a substitution
fn instantiate_type_def(def: &TypeDef, subst: &Substitution) -> Type {
    match &def.body {
        TypeBody::Enum(variants) => Type::Sum {
            name: def.name.clone(),
            type_params: def.params.clone(),
            variants: variants.iter().map(|v| Variant {
                name: v.name.clone(),
                fields: v.fields.iter()
                    .map(|(n, t)| (n.clone(), subst.apply(t)))
                    .collect(),
            }).collect(),
        },
        TypeBody::Struct(fields) => Type::Struct {
            name: def.name.clone(),
            type_params: def.params.clone(),
            fields: fields.iter()
                .map(|(n, t)| (n.clone(), subst.apply(t)))
                .collect(),
        },
        TypeBody::Alias(ty) => subst.apply(ty),
    }
}
```

### Step 4: Add Type Errors

**File**: `crates/ash-typeck/src/error.rs` (extend)

```rust
#[derive(Debug, Clone, Error)]
pub enum TypeError {
    // Existing errors...
    
    #[error("Unknown constructor: {0}")]
    UnknownConstructor(String),
    
    #[error("Unknown type: {0}")]
    UnknownType(String),
    
    #[error("Invalid constructor: {0}")]
    InvalidConstructor(String),
    
    #[error("Unknown field '{field}' in constructor '{constructor}'")]
    UnknownField { constructor: String, field: String },
    
    #[error("Missing required field '{field}' in constructor '{constructor}'")]
    MissingField { constructor: String, field: String },
}
```

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typecheck_some_constructor() {
        let env = TypeEnv::new();
        let mut var_env = VarEnv::new();
        
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![("value".into(), Expr::Literal(Value::Int(42)))],
        };
        
        let (ty, _) = check_expr(&env, &mut var_env, &expr).unwrap();
        
        // Should be Option<Int> (or Option<Var> if not fully resolved)
        assert!(matches!(ty, Type::Sum { name, .. } if name == "Option"));
    }

    #[test]
    fn test_typecheck_missing_field() {
        let env = TypeEnv::new();
        let mut var_env = VarEnv::new();
        
        // Some requires "value" field
        let expr = Expr::Constructor {
            name: "Some".into(),
            fields: vec![],  // Missing "value"
        };
        
        let result = check_expr(&env, &mut var_env, &expr);
        assert!(matches!(result, Err(TypeError::MissingField { .. })));
    }

    #[test]
    fn test_typecheck_unknown_constructor() {
        let env = TypeEnv::new();
        let mut var_env = VarEnv::new();
        
        let expr = Expr::Constructor {
            name: "UnknownVariant".into(),
            fields: vec![],
        };
        
        let result = check_expr(&env, &mut var_env, &expr);
        assert!(matches!(result, Err(TypeError::UnknownConstructor(..))));
    }
}
```

### Step 6: Run Tests

```bash
cargo test -p ash-typeck constructor -- --nocapture
```

## Completion Checklist

- [ ] `TypeEnv` for tracking type definitions
- [ ] Built-in Option and Result types registered
- [ ] Constructor lookup from name to type
- [ ] Field type checking with unification
- [ ] Generic type parameter inference
- [ ] Missing field detection
- [ ] Unknown constructor/field errors
- [ ] Unit tests for valid and invalid constructors
- [ ] Property tests for type inference
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

6 hours

## Dependencies

- TASK-121 (ADT Core Types)
- TASK-124 (Parse Type Definitions)

## Blocked By

- TASK-121
- TASK-124

## Blocks

- TASK-128 (Pattern Typing)
- TASK-131 (Constructor Evaluation)
