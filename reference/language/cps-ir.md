---
id: ref.language.cps-ir
title: Ash CPS IR
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-06-19
verified_against:
  git_commit: b7d6137f
  specs:
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
    - docs/plan/PLAN-159-CPS-IR-INTERPRETER.md
  code:
    - crates/ash-core/src/cps.rs
    - crates/ash-core/src/sexp.rs
  tests:
    - crates/ash-interp/tests/task_1590_cps_ir.rs
    - crates/ash-interp/tests/task_1591_cps_ir.rs
    - crates/ash-interp/tests/task_1592_cps_ir.rs
    - crates/ash-interp/tests/task_1593_cps_ir.rs
    - crates/ash-interp/tests/task_1594_cps_ir.rs
    - crates/ash-interp/tests/task_1595_cps_ir.rs
    - crates/ash-interp/tests/task_1596_cps_ir.rs
    - crates/ash-interp/tests/task_1598_cps_ir.rs
    - crates/ash-interp/tests/task_1599_cps_ir.rs
  examples:
    - crates/ash-interp/tests/task_1596_cps_ir.rs
refresh_trigger:
  - crates/ash-core/src/cps.rs changes
  - crates/ash-interp/src/cps.rs changes
  - docs/spec/SPEC-098b-TARGET-IR.md changes
  - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md changes
---

# Ash CPS IR

## Summary

The Ash CPS IR (Continuation Passing Style Intermediate Representation) is the target intermediate representation for the Ash language. All computation is in CPS form: every term performs an explicit control transfer via `Jump` or `Call`, and there is no implicit return. Values are inert data; terms perform computation.

This representation is the interface between the compiler frontend (which lowers Ash source to CPS IR) and the backend (which interprets, compiles to bytecode, or JITs the IR).

## Why CPS?

CPS makes control flow explicit. Every function takes an extra continuation argument, and every function call specifies where to continue after the call returns. This simplifies:

- **Effect handling**: effects are just calls to the runtime with a resume continuation
- **Tail call optimization**: every call is a tail call by construction
- **Debugging and introspection**: the continuation stack is always explicit
- **Compilers**: no need to manage a separate return stack

## Core concepts

### Values vs terms

A `Value` is inert data. It can be bound to variables but does not perform computation. A `Term` performs computation and eventually transfers control.

| Category | Examples | Role |
|----------|----------|------|
| `Value` | `Atom(Int(42))`, `Lam { ... }`, `Cont { ... }` | Data that can be bound |
| `Term` | `LetVal`, `LetPrim`, `Jump`, `Call`, `If` | Computation that transfers control |

### Atoms

Atoms are primitive values or variable references. They are the leaves of the value tree.

```rust
pub enum Atom {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Var(Name),    // variable reference — resolved in environment
}
```

### Values

Values are inert data that can be bound to variables.

```rust
pub enum Value {
    Atom(Atom),
    Lam {           // function closure
        params: Vec<Name>,
        cont: Name,
        body: Box<Term>,
        row: EffectRow,
    },
    Cont {          // continuation closure
        param: Name,
        body: Box<Term>,
        captured_env: Env,
        row: EffectRow,
    },
}
```

The `captured_env` in `Cont` is essential: when a continuation is invoked, it runs in the environment from its definition point, not the invocation point. This preserves lexical scoping for nested continuations.

### Terms

Terms are the computation nodes. Every term eventually reduces to a `Jump` to a continuation.

```rust
pub enum Term {
    LetVal { name, value, body },           // bind a value
    LetPrim { name, op, args, body },       // bind primitive result
    LetCont { name, param, cont_body, body }, // bind a continuation
    Jump { cont, arg, row },                // transfer control
    Call { func, args, cont, row },         // call function with continuation
    If { cond, then_branch, else_branch, row }, // conditional
    LetRec { name, value, body },            // recursive binding
    Raise { op, args, resume, row },         // raise an effect
    Handle { clause, body, cont, row },    // install handler
    RecordDischarge { discharge, body },    // administrative pass-through
    Trap { reason },                         // halt with reason
}
```

### Continuation references

A continuation reference is either a static label (bound by `LetCont`) or a variable (bound in the environment).

```rust
pub enum ContRef {
    Label(Name),    // static label
    Var(Name),      // variable reference
}
```

### Effect rows

Effect rows track the effects a computation may perform. Each item is a triple of `(namespace, name, kind)`.

```rust
pub struct EffectRow {
    pub items: Vec<EffectItem>,
}

pub struct EffectItem {
    pub namespace: Name,
    pub name: Name,
    pub kind: EffectItemKind,  // Capability, Role, Policy, Contract, Channel, Alias, Group
}
```

Rows are validated for duplicate `(namespace, name)` pairs. Same namespace+name with different kinds is still a duplicate.

## Serialization

The CPS IR supports two serialization formats:

### JSON (machine-oriented)

```rust
use serde_json;
let json = serde_json::to_string(&term)?;
let term: Term = serde_json::from_str(&json)?;
```

### S-expressions (human-oriented)

```rust
use ash_core::sexp::{term_to_string, string_to_term};
let sexp = term_to_string(&term)?;     // "(LetVal (name . \"x\") ...)"
let term = string_to_term(&sexp)?;     // parse back
```

S-expressions use the `.cps` file extension and are suitable for hand-authoring test fixtures.

## Example: factorial in CPS

Here's the factorial function expressed in CPS IR:

```rust
// letrec fact = (lam [n] k
//   letprim is_zero = eq n 0 in
//   if is_zero then
//     (jump k 1)
//   else
//     letprim n_minus_1 = sub n 1 in
//     letcont k_mul [result]
//       (letprim prod = mul n result in (jump k prod))
//     in (call fact [n_minus_1] k_mul))
// in (call fact [5] exit)
```

In S-expression form:

```lisp
(letrec fact
  (lam (n) k
    (letprim is_zero eq ((var n) (int 0))
      (if (var is_zero)
        (jump (var k) (int 1))
        (letprim n_minus_1 sub ((var n) (int 1))
          (letcont k_mul (result)
            (letprim prod mul ((var n) (var result))
              (jump (var k) (var prod)))
            (call (var fact) ((var n_minus_1)) (label k_mul)))))))
  (call (var fact) ((int 5)) (label exit)))
```

Key observations:
- `LetRec` binds `fact` to the lambda, allowing recursive calls
- `LetCont` creates `k_mul`, a continuation that multiplies the result
- The lambda's continuation parameter `k` is preserved across recursive calls
- Every primitive operation is bound with `LetPrim` before use

## Common patterns

### Identity function

```rust
Term::LetVal {
    name: "id".to_string(),
    value: Value::Lam {
        params: vec!["x".to_string()],
        cont: "k".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("k".to_string()),
            arg: Atom::Var("x".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    },
    body: Box::new(Term::Call {
        func: Atom::Var("id".to_string()),
        args: vec![Atom::Int(42)],
        cont: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    }),
}
```

### Conditional with exit continuation

```rust
Term::If {
    cond: Atom::Bool(true),
    then_branch: Box::new(Term::Jump {
        cont: ContRef::Label("exit".to_string()),
        arg: Atom::Int(1),
        row: EffectRow::default(),
    }),
    else_branch: Box::new(Term::Jump {
        cont: ContRef::Label("exit".to_string()),
        arg: Atom::Int(0),
        row: EffectRow::default(),
    }),
    row: EffectRow::default(),
}
```

### Effect raising and handling

```rust
// Define an effect operation
let op = EffectOp {
    item: EffectItem {
        namespace: "db".to_string(),
        name: "read".to_string(),
        kind: EffectItemKind::Capability,
    },
    arg_types: vec!["String".to_string()],
    result_type: "String".to_string(),
};

// Install a handler
Term::Handle {
    clause: HandlerClause {
        op: op.clone(),
        params: vec!["table".to_string()],
        resume: "resume".to_string(),
        body: Box::new(Term::Jump {
            cont: ContRef::Var("resume".to_string()),
            arg: Atom::String("users".to_string()),
            row: EffectRow::default(),
        }),
        row: EffectRow::default(),
    },
    body: Box::new(Term::Raise {
        op: op.clone(),
        args: vec![Atom::String("users".to_string())],
        resume: ContRef::Label("exit".to_string()),
        row: EffectRow::default(),
    }),
    cont: ContRef::Label("exit".to_string()),
    row: EffectRow::default(),
}
```

## Runtime environment

The `Env` type is an immutable frame stack with parent-chain lookup:

```rust
pub struct Env {
    pub bindings: HashMap<Name, Value>,
    pub parent: Option<Box<Env>>,
}
```

Lookup searches the current frame first, then the parent chain. This gives lexical scoping without mutation.

## Handler chain

The `HandlerChain` is an explicit stack of handler frames. The runtime searches from innermost (top) to outermost (bottom) when resolving an effect.

```rust
pub struct HandlerChain {
    pub frames: Vec<HandlerFrame>,
}

pub enum HandlerFrame {
    Shallow { clause: HandlerClause },
    Provider { op: EffectOp, handler: Name },
}
```

Shallow handlers are removed after handling a single effect. Provider frames persist across resumes.

## Known limitations

- Mutual recursion is not supported (single `LetRec` only)
- Full row polymorphism is not implemented (only duplicate validation)
- Effect aliases are not supported
- Full contract discharge is not implemented
- Bytecode compilation and JIT are deferred to future phases

## See also

- [CPS Interpreter](cps-interpreter.md) — how the interpreter evaluates CPS IR
- [The Ash Tower](tower.md) — the effect tower (Pure, Act, Proc, Workflow)
- [SPEC-098b: Target IR](../../docs/spec/SPEC-098b-TARGET-IR.md) — canonical IR specification
- [SPEC-099b: Target Operational Semantics](../../docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — formal semantics
- [PLAN-159: CPS IR Interpreter](../../docs/plan/PLAN-159-CPS-IR-INTERPRETER.md) — implementation plan
