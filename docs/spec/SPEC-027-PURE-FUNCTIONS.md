# SPEC-027: Pure Functions

## Status: Draft

## 1. Overview

Define `fn` as a first-class construct for pure computation in Ash. Functions are total
(modulo panic), deterministic, effect-free transformations from inputs to outputs. They bypass
the capability system, the effect lattice, and the workflow lifecycle machinery.

## 2. Syntax

### 2.1 Function Definition

```
[pub] fn <name>[<type_params>](<params>) [-> <return_type>]
    [requires: <predicate>]
    [ensures: <predicate>]
{
    <body>
}
```

Components:
- Visibility: optional `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` (same as types/workflows)
- Name: identifier
- Type parameters: optional `<T, U, ...>`
- Parameters: `(name: Type, ...)` (no `cap` types allowed)
- Return type: optional `-> Type` (inferred if omitted)
- Contract: optional `requires`/`ensures` clauses (arithmetic subset only, see SPEC-028)
- Body: sequence of let-bindings followed by a tail expression (the return value)

### 2.2 Function Body

The body is a sequence of statements. The last expression in the body is the return value
(tail-expression return). No `ret` keyword is used in fn bodies.

```
Body ::= Statement* Expr

Statement ::= let <pattern> = Expr ;

Expr ::= Literal
       | Variable
       | BinaryOp
       | UnaryOp
       | FunctionCall
       | Constructor
       | MatchExpr
       | IfExpr
       | PanicExpr
       | Block
```

**AST Alignment Note:** The surface grammar above corresponds to these AST variants in surface.rs:
- `MatchExpr` → `Expr::Match { scrutinee, arms, span }`
- `IfExpr` → `Expr::If { condition, then_branch, else_branch, span }` (NEW - required for fn)
- `PanicExpr` → `Expr::Panic { message, span }` (NEW - required for fn)
- `Block` → `Expr::Block { statements, tail_expr, span }` (NEW - required for fn)

**Implementation Requirement:** These three AST variants (`Expr::If`, `Expr::Panic`, `Expr::Block`) must be added to the `Expr` enum in surface.rs. They are distinct from:
- `Workflow::If` (produces workflow steps, not values)
- `Expr::IfLet` (pattern matching if, already exists)
- `Workflow::Block` (workflow block statement)

### 2.3 Match Expression

```
MatchExpr ::= match Expr { MatchArm [, MatchArm]* }

MatchArm ::= Pattern => { Body }
           | Pattern => Expr
```

Each arm produces a value. The arm body's last expression is the arm's value (tail-expression
return, same as fn body).

### 2.4 If Expression

```
IfExpr ::= if Expr then { Body } else { Body }
         | if Expr then { Body }
```

When `else` is omitted, the type is `()` (unit). Both branches must produce values of the same
type. Nested `else if` is syntactic sugar for nested `IfExpr`. When `else` is omitted, both
branches conceptually produce `Unit`. See SPEC-003 for the Unit type definition.

### 2.5 Panic

```
PanicExpr ::= panic StringLiteral
```

Immediately aborts the computation. Type-checked as returning any type (diverges).

### 2.6 Function Call

```
FunctionCall ::= name(args)
              | module::name(args)
```

Module-qualified fn calls use `::` (double colon), matching the existing module path convention in SPEC-009 and SPEC-012. This is distinct from capability calls, which use `:` (single colon) for `provider:action(args)`.

## 3. Type System

### 3.1 Function Type

```
FnType ::= Fn(<param_types>) -> <return_type>
```

With generics:
```
FnType ::= Fn<T, U>(<param_types>) -> <return_type>
```

The `Fun(T) -> U` surface syntax from the std/ files is revised to use the standard Ash form:
`Fn(T) -> U`.

**AST Coverage:** The `Type` enum in surface.rs must include:
```rust
pub enum Type {
    // ... existing variants ...
    /// Function type: Fn(T, U) -> V
    Fn {
        /// Type parameters for generic functions (optional)
        type_params: Vec<Name>,
        /// Parameter types
        params: Vec<Type>,
        /// Return type
        return_type: Box<Type>,
    },
}
```

**Parser Work:** Implement `parse_fn_type` to handle:
- `Fn(Int) -> Int` - simple function type
- `Fn(Int, String) -> Bool` - multiple parameters
- `Fn<T>(T) -> T` - generic function type
- Function types as parameter types: `fn map(f: Fn(T) -> U) -> List<U>`
- Function types in type constructors: `Option<Fn(Int) -> Int>`

**Type System Work:** The type checker must:
- Distinguish `Type::Fn` (pure fn type) from any existing `Type::Fun` (which may carry effects)
- Support unification of function types at generic instantiation sites
- Check that fn values assigned to fn type annotations match in parameter count and type

### 3.2 Function Type and Effect Neutrality

The fn type does not carry an effect slot. fn types are pure by construction:

```
FnType ::= (Type*) -> Type
```

This is distinct from the existing `Type::Fun` which may carry an effect annotation. A fn type never has an effect; fn calls are effect-neutral in all contexts.

When a workflow calls a fn, the call contributes no effect level -- it is equivalent to an `Orient` (epistemic) step. Under the current four-grade lattice (Epistemic..Operational), fn calls within a workflow body are classified as Epistemic.

### 3.3 Type Inference

Return types may be inferred from the body if omitted from the definition. Parameter types
must be explicitly annotated (no Hindley-Milner inference on parameters).

### 3.4 Generic Functions

```
fn map<T, U>(opt: Option<T>, f: Fn(T) -> U) -> Option<U> { ... }
```

Type parameters are instantiated at call sites by unification with argument types. Recursion is allowed. fn definitions may reference themselves or other fn definitions. Termination analysis for fn contracts (`ensures`) proving is deferred.

### 3.5 Purity Checking

The type checker validates fn bodies by classifying every Expr node:

**Allowed (pure):** Literal, Variable, FieldAccess, IndexAccess, Unary, Binary, Match, IfLet, Constructor, IfExpr

**Rejected unconditionally:** Policy (and all PolicyExpr variants), CheckObligation, and any construct that references capabilities, obligations, or the workflow lifecycle.

**Resolved by callee:** Expr::Call -- pure when the callee resolves to a fn definition; rejected when the callee is a capability. Expr::InterfaceMethodCall -- same resolution logic applies.

Additionally, the following keywords are rejected in fn bodies: `ret`, `act`, `observe`, `orient`, `propose`, `decide`, `receive`, `send`, `spawn`, `oblige`, `check`, `maybe`, `must`, `attempt`, `retry`, `timeout`, `yield`, `resume`. `cap` parameter types are rejected.

### 3.6 Error Conditions

The type checker produces the following errors for fn purity violations:

- `E0xxx`: `ret` in fn body -- fn uses tail-expression return, not `ret`
- `E0xxx`: capability parameter in fn definition -- fn parameters cannot be `cap` types
- `E0xxx`: effectful expression in fn body -- `[construct]` is not allowed in pure fn bodies
- `E0xxx`: call to capability from fn body -- `[name]` is a capability, not a function
- `E0xxx`: non-exhaustive match -- not all constructors of `[type]` are covered
- `E0xxx`: non-value-producing branch -- if/else branches produce incompatible types

### 3.7 Exhaustiveness

Match expressions must be exhaustive. The type checker verifies that all constructors of the
matched type are covered, considering wildcard patterns.

## 4. Operational Semantics

### 4.1 Evaluation

fn evaluation is standard call-by-value with tail-expression return:

```
(LET)
  E ⊢ e : v
  E[x↦v] ⊢ rest : v'
  ──────────────────────────────
  E ⊢ let x = e; rest : v'

(TAIL)
  E ⊢ e : v
  ──────────────────────────────
  E ⊢ e : v

(MATCH)
  E ⊢ e : v
  v matches arm_i
  E ⊢ arm_i.body : v'
  ──────────────────────────────
  E ⊢ match e { ..., arm_i => body, ... } : v'

(IF-TRUE)
  E ⊢ cond : true
  E ⊢ then_body : v
  ──────────────────────────────
  E ⊢ if cond then then_body else else_body : v

(IF-FALSE)
  E ⊢ cond : false
  E ⊢ else_body : v
  ──────────────────────────────
  E ⊢ if cond then then_body else else_body : v

(CALL)
  fn f(x₁: τ₁, ..., xₙ: τₙ) -> τ { body }
  E ⊢ a₁ : τ₁ ... E ⊢ aₙ : τₙ
  [x₁↦a₁, ..., xₙ↦aₙ] ⊢ body : v
  ──────────────────────────────
  E ⊢ f(a₁, ..., aₙ) : v

(PANIC)
  ──────────────────────────────
  E ⊢ panic msg : ⊥
```

### 4.2 No Effect Tracking

fn evaluation produces no Effect, no Trace, no Provenance. The semantic domains for fn are:

```
FnResult ::= Value | Panic
```

Contrast with workflow:
```
WorkflowOutcome ::= Return(Value, Effect, Trace, ObligationState, Provenance)
                  | Reject(Error, Effect, Trace, ObligationState, Provenance)
```

### 4.3 Fn Panic in Workflow Context

When a workflow calls a fn that panics, the panic propagates as a runtime failure. The workflow's CompletionPayload is:

```
CompletionPayload {
    result: Err(RuntimeFailure(reason: "panic: <message>")),
    obligations: ...,
    provenance: ...,
    effects: ...
}
```

The panic is not a policy violation or obligation violation -- it is an unrecoverable runtime failure that terminates the calling workflow. Supervisors and control link monitors observe it as a normal workflow failure.

### 4.4 No Lifecycle

fn has no PID, no mailbox, no control links, no suspension, no resumption. It is an immediate
synchronous computation.

## 5. Module Integration

### 5.1 Definition Kind

`fn` is a top-level module definition alongside `workflow`, `capability`, `type`, `use`, `mod`:

```
Definition ::= ...
             | FnDef(FnDef)
```

### 5.2 Visibility

`pub fn` exports the function from the module. Non-pub functions are module-private.

### 5.3 Use and Import

```
use path::{function_name};
use path::{map as map_opt};
```

Functions are imported the same way as types and capabilities.

## 6. Relationship to Workflows

| Aspect | fn | workflow |
|--------|-----|----------|
| Effects | None | Epistemic..Operational |
| Capabilities | Cannot declare or use | Declares and uses via cap params |
| Lifecycle | None (synchronous call) | Spawn, mailbox, control links |
| Return | Tail expression (last expr) | `ret expr` |
| Contracts | requires/ensures (arithmetic) | requires/ensures (full) + obligations |
| Continuations | None (scope-based) | Continuation chains |
| Provenance | None | Full audit trail |
| Composition | fn -> fn | workflow -> fn, workflow -> cap |

## 7. Examples

### 7.1 Pure helpers

```ash
fn is_some<T>(opt: Option<T>) -> Bool {
    match opt {
        Some { value: _ } => true,
        None => false
    }
}

fn unwrap_or<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some { value: v } => v,
        None => default
    }
}
```

### 7.2 Recursive function

```ash
fn length<T>(list: List<T>) -> Int {
    match list {
        [] => 0,
        [_, ..rest] => 1 + length(rest)
    }
}
```

### 7.3 With contract

```ash
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
{
    a / b
}
```

### 7.4 Called from workflow

```ash
workflow process_path(raw: String, fs: cap Fs) -> Option<String> {
    let path = from_string(raw);
    if is_absolute(path) then {
        let content = act fs:read_to_string(path);
        ret Some { value: content };
    } else {
        ret None;
    }
}
```

(Note: `is_absolute(path)` is a fn call returning Bool. The workflow `if ... then ... else ...` evaluates the condition as an expression and branches.)

## 8. Not in Scope

The following are explicitly deferred:
- Workflow implementing capability (`implements Cap` clause) -- see DESIGN-020 D7/D8
- Proxy collapse into workflow -- see DESIGN-020 D8
- Higher-kinded types for generic function composition
- Termination checking for recursive functions
- Recursion is allowed in fn bodies. Termination checking and analysis is deferred to future work -- this does not restrict fn from being recursive semantically.
- Tail-call optimization guarantees
- Dependent types for contract proving
