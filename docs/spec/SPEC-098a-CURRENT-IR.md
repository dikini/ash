---
id: spec.ash.ir.current
title: Ash Intermediate Representation — Current State
description: Current IR types and lowering pipeline as of main HEAD
code_commit: e61f2792
kind: spec
audience: [human, agent]
authority: derived-from-code
status: active
stability: beta
owner: language
last_verified: 2026-06-18
verified_against:
  git_commit: e61f2792
  code:
    - crates/ash-core/src/ast.rs
    - crates/ash-core/src/value.rs
    - crates/ash-parser/src/lower.rs
    - crates/ash-interp/src/eval.rs
---

# SPEC-098a: Ash Intermediate Representation — Current State

**Status:** Active — records the live IR as of main HEAD
**Scope:** This document is the authority for the current AST, value types, and lowering pipeline.
It does not propose changes.
**Frozen against:** `e61f2792`

## 1. Summary

The current Ash IR is the core AST in `crates/ash-core/src/ast.rs`. It is produced by the parser
(`crates/ash-parser/src/lower.rs`) and consumed by the type checker and interpreter. The IR has
separate representations for:

- pure expressions;
- Act computations;
- Proc computations;
- Workflow computations.

There is no unified effect-row representation in the IR. Effect tracking is done through the
4-point `Effect` lattice and separate `Act`, `Proc`, and `Workflow` AST variants.

## 2. Current AST Types

### 2.1 Expression AST

```rust
pub enum Expr {
    Literal(Literal),
    Variable(Name),
    Call { target, arguments },
    Let { pattern, value, body },
    Match { scrutinee, arms },
    If { condition, then_branch, else_branch },
    Lambda { params, body },
    Record { fields },
    List { elements },
    Tuple { elements },
    FieldAccess { expr, field },
    IndexAccess { expr, index },
    Act { capability, action, arguments },
    Do { target, statements },
    WithError { expr, handler },
    Check { expr },
    Fail { message },
    -- ...
}
```

### 2.2 Workflow AST

```rust
pub enum Workflow {
    Observe { capability, pattern, continuation },
    Receive { mode, arms, control },
    Orient { expr, continuation },
    Propose { action_name, action_arguments, continuation },
    Decide { expr, policy, continuation },
    Check { obligation, continuation },
    Act { provider_name, action_name, arguments, guard, provenance, result_name, continuation },
    Call { target, arguments, continuation },
    Oblig { role, workflow },
    Let { pattern, expr, continuation },
    If { condition, then_branch, else_branch },
    Seq { first, second },
    ForEach { pattern, collection, body },
    Ret { expr },
    With { capability, workflow },
    Maybe { primary, fallback },
    Must { workflow },
    Set { capability, channel, value },
    Send { capability, channel, value },
    Spawn { workflow_type, init, pattern, continuation },
    Split { expr, pattern, continuation },
    Kill { target, continuation },
    Pause { target, continuation },
    Resume { target, continuation },
    CheckHealth { target, continuation },
    Oblige { name, span },
    CheckObligation { name, span },
    Yield { role, request, expected_response_type, continuation, span, resume_var },
    ProxyResume { value, value_type, correlation_id, span },
    Done,
}
```

### 2.3 Type AST

```rust
pub enum Type {
    Named(Name),
    Constructor { name, args },
    Tuple(Vec<Type>),
    Record(Vec<(Name, Type)>),
    Fn(Vec<Type>, Box<Type>),           -- pure function
    Fun(Vec<Type>, Box<Type>, Effect),   -- effectful callable
    Var(TypeVar),
    Associated { base, name },
    -- ...
}
```

## 3. Current Value Types

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Time(...),
    Ref(...),
    List(Vec<Value>),
    Record(Vec<(Name, Value)>),
    Variant { name, payload },
    Closure { params, body, env },
    Cap { name, effect },
    ProcessHandle(...),
    Proc(...),
    Instance(...),
    InstanceAddr(...),
    ControlLink(...),
    Stream(...),
    ActEnvToken(...),
}
```

## 4. Current Lowering Pipeline

### 4.1 Parser to Core

```text
surface AST (ash-parser/src/surface.rs)
    |
    v
lower.rs -- lowers surface constructs to core AST
    |
    v
core AST (ash-core/src/ast.rs)
    |
    v
type checker (ash-typeck/src/)
    |
    v
interpreter (ash-interp/src/eval.rs)
```

### 4.2 Lowering Rules

The lowering step converts:

- `do:Act { ... }` -> `Expr::Do { target: Act, statements }`
- `do:Proc { ... }` -> `Expr::Do { target: Proc, statements }`
- `do:Workflow { ... }` -> `Expr::Do { target: Workflow, statements }`
- `act { ... }` -> `Expr::Act { ... }` or `Expr::Do { target: Act, ... }`
- `workflow { ... }` -> `Workflow::...` variants
- `fn` definitions -> `Expr::Lambda` or top-level function bindings

## 5. Known Limitations

1. No effect-row type in the AST.
2. No unified `Eff<A>` representation.
3. Separate `Act`, `Proc`, and `Workflow` AST variants.
4. No effect item identity or namespace system.
5. No row polymorphism in the IR.
6. No contract effect nodes.
7. No handler stack representation.

## 6. See Also

- [SPEC-098b: Target IR Changes](SPEC-098b-TARGET-IR.md) — IR representation for effect rows
- [SPEC-001: Intermediate Representation](SPEC-001-IR.md) — older IR spec
- [SPEC-096a: Current Effect System](SPEC-096a-CURRENT-EFFECT-SYSTEM.md)
- [SPEC-097a: Current Type System](SPEC-097a-CURRENT-TYPE-SYSTEM.md)

## 7. Changelog

- 2026-06-18: Created as current-state IR document. Frozen against `e61f2792`. Added explicit description of current AST, value types, and lowering pipeline.
