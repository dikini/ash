# SPEC-028: Function Constraint System

## Status: Draft

## 1. Overview

Define the constraint vocabulary for `fn` contracts and the evolution path from the current
arithmetic-only constraints to a richer system including string predicates, Z3 compile-time
proving, and future dependent constraints.

This spec extends the contract mechanism defined in SPEC-022 (Workflow Typing) to cover fn
contracts as a strict subset, and defines the staged growth of the constraint vocabulary.

## 2. fn Contract Syntax

```
[pub] fn name(params) -> Type
    requires: predicate₁, predicate₂, ...
    ensures: predicate₁, predicate₂, ...
{
    body
}
```

`requires` and `ensures` clauses are optional. When present, they must satisfy the fn-specific
restriction: only arithmetic/value predicates are allowed.

## 3. fn Contract Restrictions

fn contracts MAY use:
- `Arithmetic { var, constraint }` -- integer comparison constraints
- (Future) `StringConstraint { var, constraint }` -- string predicates
- (Future) `Compound` -- conjunction/disjunction of the above

fn contracts MUST NOT use:
- `HasCapability { cap, min_effect }` -- fn has no capabilities
- `HasRole(role)` -- fn has no authority context
- `oblige`/`check` obligations -- fn has no lifecycle

The type checker rejects these requirement types inside fn contract clauses with a dedicated
error:

```
error[E0xxx]: invalid fn contract requirement
  --> example.ash:2:5
   |
 2 |     requires: HasCapability(Fs, Operational)
   |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: fn contracts cannot reference capabilities
```

### 3.1 Resolved Call Targets

At the call site, fn calls and capability calls are distinguished by the call target's definition kind:
- `name(args)` resolves to a fn if `name` is bound to a fn definition (via import or local scope)
- `provider:action(args)` is always a capability call (single colon, parsed in workflow context)
- `module::name(args)` resolves to a fn if `module::name` is bound to a fn (double colon, parsed in expression context)

The resolver determines the callee kind before type checking. If a fn call targets a capability or vice versa, it is a type error.

## 4. Current Constraint Vocabulary (Stage 1)

### 4.1 ArithConstraint

```rust
pub enum ArithConstraint {
    Gt(i64),      // value > n
    Lt(i64),      // value < n
    Gte(i64),     // value >= n
    Lte(i64),     // value <= n
    Eq(i64),      // value == n
    Range { min: i64, max: i64 },  // min <= value <= max
}
```

Checking: runtime comparison against concrete values at the call site.

Example:
```ash
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result >= 0
{
    a / b
}
```

Note: `b != 0` desugars to `Arithmetic { var: "b", constraint: NotEq(0) }`. The current
ArithConstraint does not include `NotEq` -- this should be added as part of the fn work.

### 4.2 Postcondition Predicates

```rust
pub enum PostPredicate {
    Eq(String, String),                          // expr1 == expr2
    ResultSatisfies(ArithConstraint),            // result satisfies constraint
    StateAssertion(String),                      // state predicate
}
```

For fn, `ResultSatisfies` is the primary postcondition form.

> **Note:** `PostPredicate::StateAssertion` is not applicable to fn contracts. Fn contracts are value-only and have no lifecycle or state context. StateAssertion is excluded from fn contract postconditions.

## 5. Stage 2: ValueConstraint (Near Term)

Extend the constraint vocabulary to include string predicates:

```rust
pub enum ValueConstraint {
    Integer(ArithConstraint),
    String(StringConstraint),
    Bool(BoolConstraint),
}

pub enum StringConstraint {
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    MinLength(i64),
    MaxLength(i64),
    Equals(String),
    NotEquals(String),
    Matches(String),   // regex pattern
}

pub enum BoolConstraint {
    IsTrue,
    IsFalse,
}
```

### 5.1 Usage

```ash
fn is_absolute(path: PathBuf) -> Bool
    requires: string::length(path) > 0
{
    string::starts_with(path, "/")
}
```

Checking: still runtime, but richer vocabulary. The `requires` clause checks string predicates
against known values.

### 5.2 Compound Predicates

```ash
fn validate_port(port: Int) -> Bool
    requires: port >= 1 && port <= 65535
{
    true
}
```

Conjunction (`&&`) and disjunction (`||`) compose constraints. These lower to
`And(Vec<ValueConstraint>)` and `Or(Vec<ValueConstraint>)`.

> **Note:** `And`/`Or` are not new `Requirement` variants at Stage 2. They are internal lowering forms that decompose compound predicates into primitive ones for the existing `Requirement::Arithmetic` check path. A future stage may introduce them as explicit AST variants.

## 6. Stage 3: Z3 Compile-Time Proving (Feature-Gated)

Behind the existing `smt` feature flag, encode constraints as Z3 assertions for compile-time
proving.

### 6.1 Encoding

```
ArithConstraint::Gt(n)    → Z3 Int: var > n
ArithConstraint::Range    → Z3 Int: min <= var <= max
StringConstraint::StartsWith(s) → Z3 String: str.prefixof(var, s)
StringConstraint::Contains(s)   → Z3 String: str.contains(var, s)
Compound::And(cs)         → Z3 Bool: AND(encoding(c) for c in cs)
```

Z3's mixed theory solver handles integer + string constraints in the same context.

### 6.2 Proving at Call Sites

When a fn has `requires: n >= 0` and the call site provides `n = 5`, Z3 proves the constraint
trivially. When the call site provides a symbolic value, Z3 checks whether the caller's context
implies the callee's precondition.

### 6.3 Failure Reporting

```
error[E0xxx]: fn precondition may not hold
  --> example.ash:15:5
   |
15 |     let result = safe_div(x, y);
   |                   ^^^^^^^^^^^^^
   |
   = note: requires: b != 0
   = note: available: no facts about y
   = help: add a requires clause to the caller or prove y != 0 at this call site
```

## 7. Stage 4: Dependent Constraints (Future)

When the type system tracks collection sizes:

```ash
fn nth<T>(list: List<T>, n: Int) -> T
    requires: n >= 0 && n < len(list)
{
    list[n]
}
```

This requires:
- Sized types: `List<T, N>` where `N` is a natural number
- Z3 encoding of `len` as an uninterpreted function or dependent pair
- Significant type system extension -- deferred

## 8. ensures Proving

### 8.1 Current (Runtime Check)

At fn return, the `ensures` predicate is evaluated against the return value. If it fails,
a runtime error is raised.

### 8.2 With Z3 (Compile Time)

The fn body is symbolically executed to produce a postcondition:
- `a + b` where `a >= 0` and `b >= 0` → Z3 proves `result >= 0`
- If the body has branches, both branches must satisfy the ensures clause

This is a best-effort analysis. If Z3 times out or returns Unknown, the compiler falls back
to runtime checking and emits a warning.

## 9. Interaction with Workflow Contracts

Workflow contracts (SPEC-022) use the full Requirement vocabulary including HasCapability,
HasRole, and obligations. fn contracts are a strict subset.

A workflow that calls a fn inherits the fn's preconditions as call-site requirements:

```ash
workflow process(fs: cap Fs, path: String) -> Option<String>
    requires: string::length(path) > 0
{
    -- safe_div's requires is checked here with known values
    let result = safe_div(x, y);
    ...
}
```

The workflow's own contract checker verifies that fn preconditions hold at each call site.

## 10. Additions to ArithConstraint

The following are needed for fn contracts and should be added in Stage 1:

```rust
pub enum ArithConstraint {
    // Existing
    Gt(i64),
    Lt(i64),
    Gte(i64),
    Lte(i64),
    Eq(i64),
    Range { min: i64, max: i64 },
    // New
    NotEq(i64),           // value != n  (needed for div-by-zero)
    Modulo { div: i64, rem: i64 },  // value % div == rem
}
```
