# SPEC-020: Algebraic Data Types (ADTs)

## Status: Draft

## 1. Overview

This specification defines the design for Algebraic Data Types (ADTs) in Ash, including sum types (enums), product types (structs/tuples), and generics. ADTs enable:

- **Option<T>** for nullable values and control link tracking
- **Result<T, E>** for error handling  
- **User-defined enums** for state machines and domain modeling
- **Pattern matching** with exhaustiveness checking

## 2. Motivation

Current limitations:
- No way to express "value or nothing" (nullable types)
- No way to express "value or error" (error handling)
- No user-defined enums for state representation
- Limited pattern matching (no variant patterns)

ADTs solve these while maintaining type safety through exhaustiveness checking.

## 3. Design Goals

1. **Familiar syntax**: Similar to Rust/Haskell ML-family languages
2. **Zero-cost abstraction**: No runtime overhead for Option/Result
3. **Compile-time safety**: Exhaustiveness checking prevents runtime errors
4. **Ergonomics**: Good error messages, helpful type inference

## 4. Type Definitions

### 4.1 Core Type Extensions

```rust
pub enum Type {
    // Existing variants...
    Int, String, Bool, Null, Time, Ref, List, Record, Cap, Fun, Var,
    
    /// Sum type: enum with variants
    /// e.g., `Option<T> = Some { value: T } | None`
    Sum {
        name: Box<str>,
        type_params: Vec<TypeVar>,
        variants: Vec<Variant>,
    },
    
    /// Struct type: product with named fields
    /// e.g., `Point = { x: Int, y: Int }`
    Struct {
        name: Box<str>,
        type_params: Vec<TypeVar>,
        fields: Vec<(Box<str>, Type)>,
    },
    
    /// Type constructor application
    /// e.g., `Option<Int>` or `Result<T, Error>`
    Constructor {
        name: Box<str>,
        args: Vec<Type>,
    },
}

pub struct Variant {
    pub name: Box<str>,
    pub fields: Vec<(Box<str>, Type)>,
}
```

### 4.2 Type Definition AST

```rust
pub enum Definition {
    // Existing...
    TypeDef(TypeDef),
}

pub struct TypeDef {
    pub name: Box<str>,
    pub params: Vec<TypeVar>,
    pub body: TypeBody,
    pub visibility: Visibility,
}

pub enum TypeBody {
    /// type Point = { x: Int, y: Int }
    Struct(Vec<(Box<str>, Type)>),
    
    /// type Status = Pending | Processing { ... }
    Enum(Vec<Variant>),
    
    /// type PointTuple = (Int, Int)
    Tuple(Vec<Type>),
    
    /// type Name = String
    Alias(Type),
}

pub enum Visibility {
    Public,
    Crate,
    Private,
}
```

## 5. Surface Syntax

### 5.1 Enum Definition

```ash
-- Simple enum (C-like)
type Status = Pending | Processing | Completed;

-- Enum with data
type TaskResult =
    | Success { value: Value, timestamp: Time }
    | Failure { error: String, retryable: Bool };

-- Generic enum
type Option<T> =
    | Some { value: T }
    | None;

type Result<T, E> =
    | Ok { value: T }
    | Err { error: E };

-- Recursive type
type List<T> =
    | Cons { head: T, tail: List<T> }
    | Nil;
```

### 5.2 Struct Definition

```ash
-- Named struct
type Point = { x: Int, y: Int };

-- Generic struct
type Pair<T, U> = { first: T, second: U };

-- Tuple struct (anonymous fields)
type PointTuple = (Int, Int);
```

### 5.3 Pattern Matching

```ash
-- Match expression
match opt {
    Some { value: x } => x * 2,
    None => 0
}

-- Pattern in let
let Some { value: config } = load_config() else {
    ret Err { error: "Failed to load config" }
};

-- Nested patterns
match result {
    Ok { value: Some { value: x } } => x,
    Ok { value: None } => 0,
    Err { error: e } => { act log with e; 0 }
}

-- Wildcard and bindings
match status {
    Processing => "Working...",
    _ => "Done"
}

-- Record patterns (bind by field name)
let point = { x: 10, y: 20 };
let { x, y } = point;  -- x = 10, y = 20
```

### 5.4 If-Let Sugar

```ash
-- If-let as syntactic sugar for match
if let Some { value: x } = opt then {
    act log with "Got value: " ++ x;
} else {
    act log with "No value";
}

-- Expands to:
match opt {
    Some { value: x } => {
        act log with "Got value: " ++ x;
    },
    _ => {
        act log with "No value";
    }
}
```

## 6. Type Checking

### 6.1 Constructor Typing

```
(CONSTRUCTOR)
  lookup_env(Γ, C) = (Variant { fields }, ParentType)
  fields = [f1:τ1, ..., fn:τn]
  ∀i. Γ ⊢ ei : σi
  unify(σi, apply_subst(τi, subst)) = ok
  ─────────────────────────────────────────────────────────────
  Γ ⊢ C { f1: e1, ..., fn: en } : apply_subst(ParentType, subst)
```

### 6.2 Pattern Typing

```
(PATTERN-VAR)
  ─────────────────────────────────────────────────────────────
  Γ ⊢ x : τ ⊣ Γ, x:τ

(PATTERN-VARIANT)
  lookup(τ_scrutinee, C) = Variant { fields: [f1:τ1, ..., fn:τn] }
  ∀i. Γ ⊢ pi : τi ⊣ Γi
  ─────────────────────────────────────────────────────────────
  Γ ⊢ C { f1: p1, ..., fn: pn } : τ_scrutinee ⊣ Γ1 ∪ ... ∪ Γn

(PATTERN-RECORD)
  τ_scrutinee = Struct { fields: [f1:τ1, ..., fn:τn] }
  ∀i. Γ ⊢ pi : τi ⊣ Γi
  ─────────────────────────────────────────────────────────────
  Γ ⊢ { f1: p1, ..., fn: pn } : τ_scrutinee ⊣ Γ1 ∪ ... ∪ Γn
```

### 6.3 Exhaustiveness Checking

Algorithm based on pattern matrix analysis:

```rust
/// Check if patterns cover all cases for type
type Coverage =
    | Covered
    | Missing(Vec<Pattern>)  -- Witness patterns not covered

fn check_exhaustive(
    patterns: Vec<Pattern>,
    scrutinee_type: Type,
) -> Coverage {
    // Build pattern matrix
    let matrix = PatternMatrix::new(patterns);
    
    // Check for uncovered cases
    match find_uncovered(&matrix, &scrutinee_type) {
        None => Coverage::Covered,
        Some(witnesses) => Coverage::Missing(witnesses),
    }
}
```

**Example error:**
```
error[E042]: Non-exhaustive pattern match
  --> example.ash:10:3
   |
10 |   match opt {
11 |     Some { value: x } => x
   |     ^^^^^^^^^^^^^^^^^ pattern doesn't cover `None`
   |
   = help: Add a `None` arm:
   |
11 |     Some { value: x } => x,
12 |     None => todo!(),
   |
```

### 6.4 Generic Type Instantiation

```rust
/// Instantiate generic type with concrete arguments
fn instantiate(
    type_def: TypeDef,
    args: Vec<Type>,
) -> Type {
    assert_eq!(type_def.params.len(), args.len());
    
    let subst = Substitution::from_pairs(
        type_def.params.iter().zip(args.iter())
    );
    
    subst.apply_to_type_def(&type_def)
}

// Example:
// instantiate(Option<T>, [Int]) => Option<Int>
//   where T is substituted with Int throughout
```

## 7. Runtime Representation

### 7.1 Value Extensions

```rust
pub enum Value {
    // Existing...
    Int(i64), String(String), Bool(bool), /* ... */
    
    /// Tuple value
    Tuple(Vec<Value>),
    
    /// Struct value
    Struct {
        typ: Box<str>,
        fields: Vec<(Box<str>, Value)>,
    },
    
    /// Enum variant
    Variant {
        typ: Box<str>,      // Type name (e.g., "Option")
        variant: Box<str>,  // Variant name (e.g., "Some")
        fields: Vec<(Box<str>, Value)>,
    },
}
```

### 7.2 Memory Layout

```rust
// Option<T> layout (optimized):
// - Some { value }: tag = 1, payload = T
// - None: tag = 0, no payload
// 
// Null pointer optimization:
// Option<Box<T>> is same size as Box<T>
// (None = null pointer, Some = valid pointer)

// Result<T, E> layout:
// - Ok { value }: tag = 0, payload = T
// - Err { error }: tag = 1, payload = E
// Size = max(sizeof(T), sizeof(E)) + tag
```

## 8. Control Link Integration

### 8.1 Spawn Returns Composite

```ash
spawn worker with { init: args } as w;

-- w: Instance<Worker> (composite)
-- Contains: InstanceAddr<Worker> + Option<ControlLink<Worker>>

let (w_addr, w_ctrl) = split w;

-- w_addr: InstanceAddr<Worker> - opaque handle for communication
-- w_ctrl: Option<ControlLink<Worker>> - initially Some { value: link }
```

### 8.2 Control Link Transfer

```ash
-- Before transfer
if let Some { value: link } = w_ctrl then {
    send_control supervisor with link;
    -- After send_control, w_ctrl is logically None
}

-- Cannot use w_ctrl after transfer (tracked by type system)
-- w_addr remains usable
```

### 8.3 Type Checking Transfer

The type checker must track when an Option is consumed:

```rust
/// Variable state tracking for linearity
enum VarState {
    Available(Type),
    Consumed,  // e.g., after transfer
}

/// Environment tracks variable states
struct LinearEnv {
    bindings: HashMap<Name, VarState>,
}

/// Type check a match that consumes a variable
fn check_match(
    env: &mut LinearEnv,
    scrutinee: &Expr,
    arms: &[MatchArm],
) -> Result<Type, TypeError> {
    let scrut_ty = check_expr(env, scrutinee)?;
    
    // Check each arm, tracking consumption
    for arm in arms {
        let mut arm_env = env.branch();
        check_pattern(&mut arm_env, &arm.pattern, &scrut_ty)?;
        check_expr(&mut arm_env, &arm.body)?;
    }
    
    // Merge arm environments
    env.merge_branches(/* ... */);
    
    Ok(/* common result type */)
}
```

## 9. Standard Library

### 9.1 Option Module

```ash
pub type Option<T> =
    | Some { value: T }
    | None;

-- Predicates
pub fn is_some<T>(opt: Option<T>) -> Bool;
pub fn is_none<T>(opt: Option<T>) -> Bool;

-- Extraction
pub fn unwrap<T>(opt: Option<T>) -> T;  -- Panics if None
pub fn unwrap_or<T>(opt: Option<T>, default: T) -> T;
pub fn unwrap_or_else<T>(opt: Option<T>, f: Fun() -> T) -> T;

-- Transformation
pub fn map<T, U>(opt: Option<T>, f: Fun(T) -> U) -> Option<U>;
pub fn map_or<T, U>(opt: Option<T>, default: U, f: Fun(T) -> U) -> U;
pub fn and_then<T, U>(opt: Option<T>, f: Fun(T) -> Option<U>) -> Option<U>;

-- Filtering
pub fn filter<T>(opt: Option<T>, pred: Fun(T) -> Bool) -> Option<T>;

-- Boolean operations
pub fn and<T>(opt: Option<T>, other: Option<T>) -> Option<T>;
pub fn or<T>(opt: Option<T>, other: Option<T>) -> Option<T>;
pub fn xor<T>(opt: Option<T>, other: Option<T>) -> Option<T>;
```

### 9.2 Result Module

```ash
pub type Result<T, E> =
    | Ok { value: T }
    | Err { error: E };

-- Predicates
pub fn is_ok<T, E>(res: Result<T, E>) -> Bool;
pub fn is_err<T, E>(res: Result<T, E>) -> Bool;

-- Extraction
pub fn unwrap<T, E>(res: Result<T, E>) -> T;  -- Panics if Err
pub fn unwrap_or<T, E>(res: Result<T, E>, default: T) -> T;
pub fn unwrap_err<T, E>(res: Result<T, E>) -> E;  -- Panics if Ok

-- Transformation
pub fn map<T, E, U>(res: Result<T, E>, f: Fun(T) -> U) -> Result<U, E>;
pub fn map_err<T, E, F>(res: Result<T, E>, f: Fun(E) -> F) -> Result<T, F>;
pub fn and_then<T, E, U>(res: Result<T, E>, f: Fun(T) -> Result<U, E>) -> Result<U, E>;

-- Error handling
pub fn or_else<T, E>(res: Result<T, E>, f: Fun(E) -> Result<T, E>) -> Result<T, E>;
```

## 10. Property Tests

```rust
// Type safety: Well-typed ADT programs don't get stuck
proptest! {
    #[test]
    fn prop_adt_type_safety(e in arbitrary_adt_expr()) {
        if type_check(&e).is_ok() {
            let result = eval(&e);
            assert!(!result.is_stuck());
        }
    }
}

// Exhaustiveness: All well-typed matches have a matching arm
proptest! {
    #[test]
    fn prop_exhaustive_match_never_fails(
        v in arbitrary_value(),
        arms in arbitrary_exhaustive_arms()
    ) {
        let result = eval_match(v, arms);
        assert!(result.is_ok(), "Exhaustive match should never fail");
    }
}

// Pattern matching roundtrip
proptest! {
    #[test]
    fn prop_construct_deconstruct(v in arbitrary_variant()) {
        let constructed = construct_variant(&v);
        let pattern = variant_to_pattern(&v);
        let matched = match_pattern(&pattern, &constructed);
        assert!(matched.is_ok());
        
        // Verify bindings
        let bindings = matched.unwrap();
        assert_eq!(reconstruct_from_bindings(&bindings, &v.typ), v);
    }
}

// Generic instantiation preserves semantics
proptest! {
    #[test]
    fn prop_generic_substitution(
        def in arbitrary_type_def(),
        args in arbitrary_type_args()
    ) {
        let instantiated = instantiate(&def, &args);
        
        // Type should be fully concrete (no free vars)
        assert!(!has_free_vars(&instantiated));
        
        // Original def params should be substituted
        for (param, arg) in def.params.iter().zip(args.iter()) {
            assert!(!contains_var(&instantiated, param));
        }
    }
}
```

## 11. Implementation Phases

### Phase 1: Core Types (TASK-121 to TASK-123)
- Add Type::Sum, Type::Struct, Type::Constructor
- Add Value::Variant, Value::Struct, Value::Tuple
- Update unification for new types

### Phase 2: Parser (TASK-124 to TASK-126)
- Parse type definitions
- Parse variant constructors
- Parse match expressions and patterns

### Phase 3: Type Checker (TASK-127 to TASK-130)
- Type check constructors
- Type check patterns
- Exhaustiveness checking
- Generic type instantiation

### Phase 4: Interpreter (TASK-131 to TASK-133)
- Evaluate constructors
- Pattern matching engine
- Match expression evaluation

### Phase 5: Integration (TASK-134 to TASK-135)
- Spawn returns Option<ControlLink>
- Control link transfer semantics

### Phase 6: Standard Library (TASK-136)
- Option and Result modules

## 12. Related Documents

- SPEC-003: Type System
- PLAN-020: ADT Implementation Plan
- TASK-121 through TASK-136: Individual implementation tasks
