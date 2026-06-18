---
id: spec.ash.ir.target
title: Ash Intermediate Representation — Target State
description: Target CPS IR with unified effect rows, three-layer grammar, and operation-typed raise/handle
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

**Status:** Draft — target CPS IR for unified effect rows
**Scope:** This document defines the IR representation we want Ash to have.
It is a goal-state living document that will be refined as implementation progresses.
**Depends on:** SPEC-095b (Target Grammar), SPEC-096b (Target Effect System), SPEC-097b (Target Type System)

## 1. Summary

The target IR is a **CPS (Continuation-Passing Style)** intermediate representation with
three syntactic layers: atoms, values, and tail computations. Every non-atomic computation
is a tail term that eventually jumps, calls, raises, or handles. There is no direct return.

Key design decisions:

1. **Three-layer grammar**: `Atom` (variables, literals, labels), `Value` (atoms, lambdas,
   records, tuples), `Term` (let-bindings, calls, jumps, conditionals, raise, handle).
2. **Call vs Jump**: Ordinary function calls (`Call`) are separate from continuation
   invocation (`Jump`). A continuation is not a CPS function that takes another continuation.
3. **Answer type discipline**: Every CPS term is typed under a fixed answer type `Ans`.
   A continuation `Cont<A, Ans, ρ>` consumes an `A` and produces an `Ans`. A CPS function
   called with that continuation must also produce `Ans`.
4. **Row composition**: The total effect row of a call is the union of the callee's body
   row and the continuation's row. They are not conflated.
5. **Operation-typed raise/handle**: `Raise` names an operation with argument types and a
   result type. The resume continuation has type `Cont<OpResult, Ans, ρ_resume>`.
6. **Rows are requirements**: Their discharge is kind-specific: capabilities, channels,
   process effects, and failures may appear as raised operations matched by `Handle` frames,
   while roles, policies, contracts, resources, and evidence use static, evidence, ownership,
   or boundary mechanisms.
7. **Backward compatibility**: Legacy AST variants are lowered to CPS during migration.

### 1.1 Continuation Representation

The IR supports two representations for continuations:

1. **Labels**: Named continuation targets defined outside the term grammar. Labels are
   bound at function entry or at `LetCont` declarations. They are not first-class values
   and cannot be stored in data structures or passed as ordinary arguments.
2. **Continuation closures**: `Cont` values that capture their environment. These are
   first-class and can be passed as arguments, but they are linear/affine: they can be
   invoked at most once.

For this IR slice, the default representation is **labels**. Continuation closures are
used only when a continuation must be passed as a value (e.g., to a handler clause or
a higher-order function). The spec uses `Atom` for continuation references, which may be
`Label` or `Var` depending on the representation choice.

```text
Atom ::= ... | Label(LabelId) | Var(Name)
```

A `Label` is a static continuation target. A `Var` may name a `Lam` (CPS function) or a
`Cont` (continuation closure). The type checker distinguishes them by type.

**Label declarations:**

Labels are declared at function scope or via `LetCont`:

```text
Term ::= ... | LetCont { name: LabelId, param: Param, body: Term, rest: Term }
```

`LetCont` binds a label `name` with parameter `param` to `body`. The label is visible in
`rest` and can be jumped to from anywhere in `rest`. Labels are not values — they are
static control-flow targets. A `Jump { cont: Label(name), arg: v }` transfers control to
the label's body with `v` bound to the label's parameter.

**Continuation closure values:**

When a continuation must be passed as a value (e.g., as a resume parameter), it is a
`Cont` value:

```text
Value ::= ... | Cont { param: Param, body: Term, env: Env, row: EffectRow }
```

A `Cont` value is a closure that captures its environment. It is invoked by `Jump` and
consumes its argument to produce the answer. `Cont` values are linear/affine: they can be
invoked at most once. The type checker enforces this via affine typing.

## 2. Core Grammar

The target IR has three syntactic layers. This separation is required for CPS soundness.

### 2.1 Atom

An atom is a reference that needs no evaluation.

```text
Atom ::= Var(Name)
       | Lit(Literal)
       | Label(LabelId)
       | PrimName(PrimOp)
       | ConstructorName(Name)
```

Atoms appear in argument position, constructor fields, and primitive operations.

### 2.2 Value

A value is an atom or a value constructor that does not perform effects.

```text
Value ::= Atom
        | Lam { params: Vec<Param>, cont_param: Param, body: Term, row: EffectRow }
        | Cont { param: Param, body: Term, env: Env, row: EffectRow }
        | Record { fields: Vec<(Name, Atom)> }
        | Tuple { elems: Vec<Atom> }
        | DischargeMarker { discharge: ContractDischarge }
```

A `Lam` is a CPS function value. It is not a tail computation — it is a value that can be
bound to a variable, passed as an argument, or stored in a data structure. The body of a
`Lam` is a `Term` (a tail computation), not a `Value`.

### 2.3 Term

A term is a tail computation that eventually jumps, calls, raises, or handles. Every term
is evaluated under an answer type `Ans`.

```text
Term ::= LetVal { name: Name, value: Value, body: Term }
        | LetPrim { name: Name, op: PrimOp, args: Vec<Atom>, body: Term }
        | LetCont { name: LabelId, param: Param, body: Term, rest: Term }
        | Call { func: Atom, args: Vec<Atom>, cont: Atom, row: EffectRow }
        | Jump { cont: Atom, arg: Atom, row: EffectRow }
        | If { cond: Atom, then_branch: Term, else_branch: Term, row: EffectRow }
        | Raise { op: EffectOp, args: Vec<Atom>, resume: Atom, row: EffectRow }
        | Handle { clause: HandlerClause, body: Term, cont: Atom, row: EffectRow }
        | RecordDischarge { discharge: ContractDischarge, body: Term }
        | Trap { reason: TrapReason }
```

**Key invariants:**

1. Every term eventually reaches a `Call`, `Jump`, `Raise`, `Handle`, or `If` that transfers
   control. There is no "return" — the answer type is produced by the outermost continuation.
2. A `Jump` invokes a continuation with a single argument. The continuation is not a CPS
   function that takes another continuation; it is a `Cont` value that consumes the argument
   and produces the answer.
3. A `Call` invokes an ordinary CPS function (`Lam`) with arguments and a continuation.
   The callee's body is a term that must eventually `Jump` to the provided continuation.
4. `LetVal`, `LetPrim`, and `LetCont` bind values, primitive results, and labels to names.
   They are administrative normal form (ANF) bindings. Every intermediate result is named.
5. `RecordDischarge` is an administrative term that records contract discharge status.
   It is a no-op at runtime but preserves metadata for audit and evidence caching.
6. `Trap` is an unrecoverable abort. It does not resume and is outside ordinary row
   accounting. `TrapReason` is diagnostic metadata and does not contribute an effect row.
   The row of a term containing `Trap` is `{}` (bottom row). Recoverable failures must use
   `Raise { op: EffectOp { item: Failure(...), ... }, ... }` and are row-accounted.

### 2.4 Answer Type Discipline

Every CPS term is typed under a fixed answer type `Ans` for its region:

```text
Γ ⊢ atom : A
Γ ⊢ value : A
Γ ⊢ term ! Ans, ρ
```

A continuation has type:

```text
Cont<A, Ans, ρk>  -- consumes A, produces Ans, with row ρk
```

A CPS function called with a continuation must produce the same `Ans`:

```text
f : CpsFn { params: [A], cont: Cont<B, Ans, ρk>, body_row: ρf, total_row: ρf ∪ ρk }
```

The answer type is fixed for a compilation region (e.g., a function, a workflow, or a module
entry point). It is not polymorphic unless the region explicitly supports answer-type
polymorphism.

## 3. CPS Types

### 3.1 CPS Function Type

A CPS function type separates the callee's body row from the continuation's row:

```rust
pub struct CpsFn {
    pub params: Vec<Type>,           -- ordinary parameters
    pub cont: Box<Cont>,             -- continuation type
    pub answer: Type,                -- fixed answer type for the region
    pub body_row: EffectRow,         -- effects of the function body itself
    pub total_row: EffectRow,        -- body_row ∪ cont_row (computed, not stored separately)
}

pub struct Cont {
    pub arg: Type,                   -- argument type
    pub answer: Type,                -- must match the region's answer type
    pub row: EffectRow,              -- effects of the continuation
}
```

A surface function `fn f(x: A) -> {ρf} B { ... }` lowers to a CPS function:

```text
f : ∀Ans ρk. CpsFn {
    params: [A],
    cont: Cont<B, Ans, ρk>,
    answer: Ans,
    body_row: ρf,
    total_row: ρf ∪ ρk
}
```

The callee's body row `ρf` and the continuation's row `ρk` are distinct. The total row of
the call is their union. A pure function called with an effectful continuation does not
become intrinsically effectful — the total call context has the union of both.

### 3.2 Continuation Type

A continuation is not a CPS function that takes another continuation. It is a one-shot
consumer of a value that produces the answer:

```text
Cont<A, Ans, ρ>  -- A -> {ρ} Ans
```

In the IR, a continuation is referenced by an atom: either a `Label` for a static
continuation target, or a `Var` naming a `Cont` closure value. It is invoked by `Jump`:

```text
Jump { cont: k, arg: v, row: ρ }
```

The `row` on `Jump` is the continuation's row `ρk`, not the caller's body row.

### 3.3 Effect Row Type

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

## 4. Effect Item Identity

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

See SPEC-096b and SPEC-097b for the taxonomy and semantic definitions.

### 4.1 Contract Discharge Status

Contract effects carry discharge status in the IR or in a sidecar:

```rust
pub struct ContractDischarge {
    pub contract: ContractEffect,
    pub mode: DischargeMode,
    pub evidence: Option<EvidenceRef>,
    pub source_span: Span,
}

pub enum DischargeMode {
    Static,      -- discharged by type checker / prover
    Evidence,    -- discharged by proof / test / law evidence
    Dynamic,     -- discharged by runtime contract handler
}
```

A contract effect cannot be silently erased from a row without recording its discharge mode.
The IR must preserve this information for audit, diagnostics, and evidence caching.

## 5. Raise and Handle

### 5.1 Effect Operation

`Raise` names an operation with a signature:

```rust
pub struct EffectOp {
    pub item: EffectItem,
    pub args: Vec<Type>,
    pub result: Type,
}

pub enum TrapReason {
    ContractViolation(ContractEffect),
    UnhandledEffect(EffectOp),
    Panic(String),
}
```

The resume continuation has type `Cont<OpResult, Ans, ρ_resume>`.

### 5.2 Raise

```rust
pub struct Raise {
    pub op: EffectOp,
    pub args: Vec<Atom>,
    pub resume: Atom,           -- resume: Cont<OpResult, Ans, ρ_resume>
    pub row: EffectRow,
}
```

The `row` on `Raise` is the **operation row** `ρ_op`: the effect of the operation request
itself. The total row of the term containing the `Raise` is `ρ_op ∪ ρ_resume`, where
`ρ_resume` is the row of the resume continuation. The type checker computes the total term
row; the `Raise.row` field records only the operation's local row.

For example, a capability call `Raise { op: cap db.read, resume: k }` has:
- `op_row = {cap db.read}`
- `resume.row = ρk`
- `term_row = {cap db.read} ∪ ρk`

**Capability discharge layering:** For capability operations, `Raise` is the operational
request form. The requirement row is discharged by an admitted provider/authority. In the
CPS IR, that provider may be represented as a `HandlerFrame` installed by the runtime boundary,
or as ambient authority that directly services the `Raise` without an explicit handler. Both
paths preserve the "rows are requirements, not grants" rule: the row records what is needed,
not what is available.

### 5.3 Handler Clause

```rust
pub struct HandlerClause {
    pub op: EffectOp,
    pub params: Vec<Param>,
    pub resume: Param,          -- resume: Cont<OpResult, Ans, ρ_resume>
    pub body: Box<Term>,
    pub row: EffectRow,
}
```

The handler clause body is a term that must eventually `Jump` to `resume` or `Jump` to the
outer continuation. The `resume` parameter is one-shot: after the handler body jumps to it,
it is consumed.

**One-shot enforcement:** The IR enforces one-shot resume via **linear/affine typing**.
The `resume` parameter has an affine type: it can be used at most once in the handler body.
If the handler body duplicates `resume` (e.g., stores it in a data structure, passes it to
another function, or jumps to it twice), the type checker rejects the term. For the initial
implementation, a simpler runtime check that traps on second use is acceptable as a
stopgap, but the target is static affine typing.

Affine use is control-flow/path sensitive: a continuation may appear in multiple mutually
exclusive branches, but at most one dynamic invocation may occur.

### 5.4 Handle

```rust
pub struct Handle {
    pub clause: HandlerClause,
    pub body: Box<Term>,
    pub cont: Atom,             -- current continuation for normal completion: Cont<A, Ans, ρ_cont>
    pub row: EffectRow,         -- residual row after handling
}
```

The `row` on `Handle` is the **local residual body row** after removing the handled
operation and adding handler effects. The total row of the `Handle` term is
`Handle.row ∪ ρ_cont`, where `ρ_cont` is the row of `Handle.cont`. This mirrors the
same local-vs-total separation used by `Raise`.

### 5.5 Handler Row Transformation

A `Handle` node transforms the row of its body, but only for **raised operations** (see
§5.6). Ambient-discharge items (roles, policies, contracts, evidence, resources) are not
removed by `Handle`; they are discharged by their respective kind-specific mechanisms.

For a raised operation:

```text
body row: {op, ... | r}
handler row: {handler_effects}
Handle { op, ... } row: {handler_effects, ... | r}
```

The handled operation `op` is removed from the row. The handler's own effects are added.
Unhandled operations in the row remain. This rule applies only to operations from the
"raised operations" category in §5.6.

### 5.6 Effect Handling vs Ambient Discharge

Not every `EffectItem` is a resumable algebraic operation. The IR distinguishes:

- **Raised operations**: Capability calls, channel ops, process ops, failures — these are
  runtime requests that are matched by `Handle` frames.
- **Ambient discharge**: Roles, policies, contracts, resources, evidence — these are discharged by
  static checking, evidence proofs, or runtime boundary checks, not by `Handle` frames.

A capability call lowers to `Raise` with a `Capability` operation. A role admission is not
raised; it is checked statically or at the workflow boundary. A policy effect is not raised;
it is evaluated by a policy handler at an explicit boundary. A resource effect is not raised;
it is discharged through ownership, borrow, split, join, or provenance tracking at the
runtime boundary.

This aligns with SPEC-096b's "rows are requirements, not grants" rule.

## 6. Lowering Pipeline

### 6.1 Target Lowering

```text
surface AST (with effect rows)
    |
    v
lower.rs -- lowers to unified CPS IR
    |
    v
core CPS AST (Atom/Value/Term with EffectRow)
    |
    v
type checker (with row discharge and answer type discipline)
    |
    v
interpreter (with continuation chain and handler frames)
```

### 6.2 CPS Lowering Rules

| Surface | CPS Target IR |
|---------|---------------|
| `fn f(x: A) -> {ρ} B { body }` | `Lam { params: [x], cont_param: k, body: [lowered body], row: ρ }` |
| `f(x)` | `Call { func: f, args: [x], cont: k, row: ρ_total }` |
| `return v` | `Jump { cont: k, arg: v, row: ρk }` |
| `let x = v in e` | `LetVal { name: x, value: [v], body: [e] }` |
| `let x = a + b in e` | `LetPrim { name: x, op: Add, args: [a, b], body: [e] }` |
| `if c then t else e` | `If { cond: c, then_branch: [t], else_branch: [e], row: ρ }` |
| `handle E with { ... }` | `Handle { clause: C, body: [lowered], cont: k, row: ρ_residual }` |
| `raise E(args)` | `Raise { op: O, args: [lowered], resume: k, row: ρ_op }` |

## 7. CPS Lowering Examples

### 7.1 Pure Function (Fully Normalized)

```ash
fn add(a: Int, b: Int) -> Int { a + b }
```

Lowers to:

```text
add =
  Lam { params: [a, b], cont_param: k, row: {},
    body:
      LetPrim { name: sum, op: Add, args: [a, b],
        body:
          Jump { cont: k, arg: sum, row: {} } } }
```

Key points:
- `a + b` is a primitive operation, bound via `LetPrim`.
- The result is returned by `Jump` to the continuation `k`.
- No `App` or `Call` — the function body is a straight-line sequence of `LetPrim` and `Jump`.

### 7.2 Direct Capability Operation (Fully Normalized)

```ash
fn get_user_name(id: Int) -> {cap db.read} String {
    db.read("users", id)
}
```

Lowers to:

```text
get_user_name =
  Lam { params: [id], cont_param: k, row: {cap db.read},
    body:
      Raise {
        op: EffectOp { item: Capability(db.read), args: [String, Int], result: String },
        args: ["users", id],
        resume: k,
        row: {cap db.read} } }
```

Key points:
- A capability call is a `Raise`, not a `Call` or `App`.
- The operation signature `(String, Int) -> String` is explicit in the `EffectOp`.
- The resume continuation is the function's own continuation `k`.
- The row `{cap db.read}` is the requirement that must be discharged by ambient authority.

### 7.3 Dynamic Contract Discharge (Fully Normalized)

A contract checked at runtime with dynamic discharge:

```ash
fn safe_divide(a: Int, b: Int) -> {requires {b != 0}} Int {
    a / b
}
```

Lowers to a dynamic contract check followed by the operation:

```text
safe_divide =
  Lam { params: [a, b], cont_param: k, row: {requires {b != 0}},
    body:
      LetPrim { name: ok, op: Neq, args: [b, 0],
        body:
          If { cond: ok,
            then_branch:
              LetPrim { name: result, op: Div, args: [a, b],
                body:
                  RecordDischarge {
                    discharge: ContractDischarge {
                      contract: Contract(requires {b != 0}),
                      mode: Dynamic,
                      evidence: None,
                      source_span: ... },
                    body:
                      Jump { cont: k, arg: result, row: {} } },
            else_branch:
              Trap { reason: ContractViolation(requires {b != 0}) },
            row: {} } } }
```

Key points:
- The source function advertises the pre-discharge contract requirement `{requires {b != 0}}`.
- After the dynamic check records `ContractDischarge`, the residual row of the continuation
  is `{}`.
- The contract predicate `b != 0` is a primitive operation (`Neq`), bound via `LetPrim`.
- On success: the contract is discharged with `mode: Dynamic` and recorded in the IR.
- On failure: the computation traps with `Trap { reason: ContractViolation(...) }`. A trap
  is an unrecoverable abort that does not resume. It is outside ordinary row accounting.
- The `RecordDischarge` node is a no-op at runtime but preserves the discharge status for
  audit and evidence caching. It does not appear in the effect row after discharge.
- This example is consistent with §5.6: contracts are discharged by boundary checks, not by
  `Handle` frames.

**Alternative failure semantics:** If the contract violation is recoverable, the failure
branch can `Raise` a `Failure` effect instead of trapping. The choice between trap and recovery
is a policy decision, not an IR invariant. If recovery is used, the function row must include
the failure effect.

## 8. Handler Patterns (Schematic Examples)

The following patterns show how common control-flow constructs are expressed as CPS handler
combinations. These examples are **schematic** — they use helper notation (e.g., `success()`,
`Err(...)`, arithmetic in atom position) that is not part of the core grammar. They illustrate
the design direction without claiming to be fully normalized target IR.

### 8.1 Retry Pattern

A retry boundary re-executes a computation up to a maximum number of attempts:

```ash
fn fetch_with_retry(url: String) -> {cap http.get} Result<String, NetworkError> {
    retry max_attempts: 3, backoff_ms: 1000 {
        http.get(url)
    } handle {
        NetworkError => retry;
        _ => fail UnrecoverableError;
    }
}
```

Schematic CPS lowering:

```text
fetch_with_retry = Lam { params: [url], cont_param: k, row: {cap http.get},
    body: LetVal { name: retry_loop,
        value: Lam { params: [attempts_left], cont_param: k_retry, row: {cap http.get},
            body: If { cond: attempts_left > 0,
                then_branch:
                    Raise {
                        op: EffectOp { item: Capability(http.get), args: [String], result: Result<String, NetworkError> },
                        args: [url],
                        resume: k_body,
                        row: {cap http.get} }
                    -- k_body is a continuation that checks result and retries
                else_branch: Jump { cont: k, arg: Err(UnrecoverableError), row: {} } } },
        body: Call { func: retry_loop, args: [3], cont: k, row: {cap http.get} } } }
```

Key points:
- `retry_loop` is a recursive CPS function.
- The body is re-invoked on each retry by `Call` to `retry_loop`.
- The capability operation is a `Raise`, not a `Call`.
- The schematic uses `Err(...)` and `>` in atom position — these would be `LetPrim` bindings
  in fully normalized IR.

### 8.2 Rollback / Compensation Pattern

A compensation pair executes a primary action and, on failure, runs an undo action:

```ash
fn transfer_funds(from: Account, to: Account, amount: Int)
    -> {cap db.read, cap db.write} Result<Unit, TxError>
{
    compensate {
        do { reserve <- payment.reserve(from, amount); return reserve }
    } undo {
        payment.release(from, amount);
    } in {
        do {
            transfer <- payment.transfer(from, to, amount);
            return transfer
        }
    }
}
```

Schematic CPS lowering:

```text
transfer_funds = Lam { params: [from, to, amount], cont_param: k, row: {cap db.read, cap db.write},
    body: Raise {
        op: EffectOp { item: Capability(payment.reserve), args: [Account, Int], result: Result<Unit, TxError> },
        args: [from, amount],
        resume: k_reserve,
        row: {cap db.write} }
}

k_reserve(reserve_result):
  If { cond: is_ok(reserve_result),
    then_branch:
      Raise {
        op: EffectOp { item: Capability(payment.transfer), args: [Account, Account, Int], result: Result<Unit, TxError> },
        args: [from, to, amount],
        resume: k_transfer,
        row: {cap db.write} }
    else_branch:
      Jump { cont: k, arg: Err(TxError), row: {} } }

k_transfer(transfer_result):
  If { cond: is_ok(transfer_result),
    then_branch:
      Jump { cont: k, arg: Ok(()), row: {} }
    else_branch:
      Raise {
        op: EffectOp { item: Capability(payment.release), args: [Account, Int], result: () },
        args: [from, amount],
        resume: k_release,
        row: {cap db.write} } }

k_release(_):
  Jump { cont: k, arg: Err(TxError), row: {} }
```

Key points:
- The undo action is inlined into the failure continuation `k_transfer`.
- On failure: `Raise` to `payment.release`, then `Jump` to `k` with error.
- The schematic uses `is_ok(...)` and `Ok(...)` in atom position — these would be `LetPrim`
  bindings in fully normalized IR.

### 8.3 Transactional Memory Pattern

A transaction boundary intercepts read and write effects:

```ash
fn update_balance(account: Account, delta: Int)
    -> {cap db.read, cap db.write} Result<Unit, TxError>
{
    transaction {
        let balance = db.read(account);
        db.write(account, balance + delta);
    }
}
```

Schematic CPS lowering:

```text
update_balance = Lam { params: [account, delta], cont_param: k, row: {cap db.read, cap db.write},
    body: LetVal { name: tx_log, value: new_log(),
        body: Handle {
            clause: HandlerClause {
                op: EffectOp { item: Capability(db.read), args: [Account], result: Int },
                params: [key],
                resume: resume,
                body: If { cond: tx_log.has_write(key),
                    then_branch: Jump { cont: resume, arg: tx_log.get_write(key), row: {} },
                    else_branch: LetPrim { name: value, op: StoreRead, args: [key],
                        body: LetPrim { name: _, op: LogAddRead, args: [tx_log, key, value],
                          body: Jump { cont: resume, arg: value, row: {} } } },
                row: {} },
            body: Handle {
                clause: HandlerClause {
                    op: EffectOp { item: Capability(db.write), args: [Account, Int], result: () },
                    params: [key, value],
                    resume: resume,
                    body: LetPrim { name: _, op: LogAddWrite, args: [tx_log, key, value],
                        body: Jump { cont: resume, arg: (), row: {} } },
                    row: {} },
                body: Raise {
                    op: EffectOp { item: Capability(db.read), args: [Account], result: Int },
                    args: [account],
                    resume: k_read,
                    row: {cap db.read} }
                -- k_read continues with balance, then writes, then validates
                cont: k, row: {tx_log_effects} },
            cont: k, row: {tx_log_effects} } } }
```

Key points:
- Two nested `Handle` frames intercept `db.read` and `db.write`.
- The `tx_log` is a value threaded through the handler closures.
- The schematic uses `tx_log.has_write(key)` and `tx_log.get_write(key)` in atom position —
  these would be `LetPrim` bindings in fully normalized IR.
- The transaction boundary itself is not a separate effect item; it is nested handlers over
  capability operations.

### 8.4 Summary of Handler Patterns

| Pattern | Core Mechanism | IR Nodes Used |
|---------|---------------|---------------|
| Retry | Recursive CPS wrapper | `Lam`, `Raise`, `Call`, `If`, `LetVal` |
| Rollback | Undo actions in failure continuations | `Raise`, `If`, `Jump`, `LetVal` |
| Transaction | Nested `Handle` frames intercepting `Raise` | `Handle`, `Raise`, `Lam`, `If`, `LetVal`, `LetPrim` |

All three patterns are built from the core CPS nodes. No special-purpose constructs.

**Note on example rows:** Unless stated otherwise, examples assume the top-level continuation
`k` has empty row `{}`. In production IR, continuation rows carry the effects of the rest of
the computation.

## 9. Laziness and Evaluation Strategy in CPS (Pseudo-IR)

This section explores how call-by-name and call-by-need evaluation can be expressed in the
CPS IR. The examples below are **pseudo-IR** — they use notation that is not yet part of the
core grammar (e.g., mutable environment cells, `Option`, field access, inline continuation
closures). A fully normalized term must first bind each thunk closure and each inline
continuation closure to a name, and must lower memo-cell reads/writes through the chosen
primitive/effect model.

### 9.1 Thunks in CPS

A thunk is a zero-parameter CPS function that delays evaluation:

```text
Thunk<A> = Lam {
  params: [],
  cont_param: k : Cont<A, Ans, ρk>,
  body: Term ! Ans, ρ,
  row: ρ
}
```

When forced with `Call { func: thunk, args: [], cont: k, row: ρ }`, it evaluates its body and
invokes `k` with the result.

### 9.2 Call-by-Name

In call-by-name, arguments are wrapped in thunks and passed unevaluated. The function
forces each thunk at each use site.

**Pseudo-IR example:**

```text
lazy_if = Lam { params: [cond, then, else], cont_param: k, row: r,
    body: Call { func: cond, args: [],           -- force cond
        cont: Lam { params: [cond_val], cont_param: k_cond, row: r,
            body: If { cond: cond_val,
                then_branch: Call { func: then, args: [], cont: k, row: r },
                else_branch: Call { func: else, args: [], cont: k, row: r },
                row: r } } } }
```

Key points:
- Arguments are `Lam { params: [] }` — thunks.
- The caller does not evaluate arguments before the call.
- The callee decides when to force each thunk by `Call` with empty args.
- Effects inside the thunk fire at the force site, not at the call site.

### 9.3 Call-by-Need (Memoized)

In call-by-need, a thunk is forced once, then its result is cached.

**Pseudo-IR example:**

```text
memo_thunk = CpsClosure {
    params: [],
    cont_param: k,
    env: { computed: Bool, value: Option<A>, body_thunk: Thunk<A> },
    body: If { cond: env.computed,
        then_branch: LetPrim { name: cached, op: UnwrapOption, args: [env.value],
            body: Jump { cont: k, arg: cached, row: ρk } },
        else_branch: Call { func: env.body_thunk, args: [],
            cont: Lam { params: [v], cont_param: k_body, row: r,
                body: LetPrim { name: _, op: SetCell, args: [env.computed, true],
                    body: LetPrim { name: _, op: SetCell, args: [env.value, Some(v)],
                        body: Jump { cont: k, arg: v, row: {} } } } } } },
    row: r
}
```

Key points:
- The thunk is a `CpsClosure` with a mutable environment cell.
- On first force: evaluate `body`, store result, return it.
- On subsequent force: return cached result directly.
- The pseudo-IR uses `env.computed`, `env.value.unwrap()`, and `SetCell` — these would be
  lowered to primitive operations or memory effects in fully normalized IR.

### 9.4 Effect Row Implications (Design Space)

The effect row of a lazy function depends on which thunks are forced:

```text
fn f(x: Thunk<{cap db.read} String>) -> {cap db.read} Int
```

If `x` is forced inside `f`, the row includes `cap db.read`. If `x` is never forced, the row
does not. The type checker must approximate this (e.g., assume all thunks may be forced, or
track force sites).

For call-by-need, the memoization itself is a stateful operation. It could be tracked as an
effect:

```text
memo Thunk<A>  -- reads/writes a memo cell
```

Or it could be treated as a pure runtime primitive (invisible to the effect system).

### 9.5 Open Questions for Future Specs

1. **Surface syntax:** How are thunks created and forced? Explicit syntax (`Thunk { expr }`,
   `force x`) or implicit (`lazy expr`, `x`)?
2. **Type system:** Is `Thunk<A>` a distinct type constructor? Can it appear in effect rows?
3. **Effect tracking:** Are `force` and `memo` explicit effect items, or implicit runtime
   operations?
4. **Pattern matching:** Are patterns lazy or strict by default? Can patterns force thunks?
5. **Data constructors:** Can constructor fields be lazy? How is this annotated?
6. **Interaction with `do`:** Is `do` notation strict or lazy? Can lazy thunks appear in `do`
   blocks?
7. **Memoization scope:** Is memoization per-thunk (global) or per-evaluation-context (local)?

### 9.6 Summary

| Strategy | IR Representation | Force Mechanism | Memoization |
|----------|-------------------|-------------------|-------------|
| Call-by-value | Value passed directly | N/A | N/A |
| Call-by-name | `Lam { params: [] }` | `Call { func: thunk, args: [] }` | None |
| Call-by-need | `CpsClosure` with memo cell | `Call { func: thunk, args: [] }` | Env cell |

Laziness is a calling convention in the CPS IR. No special nodes needed. The design space for
surface syntax, type system integration, and effect tracking is left to future specs.

## 10. Handler Stack as CPS Continuation Chain

In CPS, handlers are not a separate stack data structure. They are continuation frames that
wrap the "next" continuation. A `HandlerFrame` is a `Cont` value that intercepts matching
`Raise` nodes.

### 10.1 Handler Frame

```rust
pub struct HandlerFrame {
    pub clause: HandlerClause,
    pub next: Box<Cont>,      -- the next continuation in the chain
    pub parent: HandlerChain, -- the rest of the chain beyond this frame
}

pub enum HandlerChain {
    Empty,                          -- end of chain
    Frame(Box<HandlerFrame>),       -- another handler frame
    Cont(Box<Cont>),                -- ordinary continuation
}
```

A handler frame participates in the current continuation chain. Normal `Jump`s pass through
to `next`. `Raise` dispatch walks the chain and may select the frame by matching `clause.op`.

### 10.2 Handle Operational Semantics

`Handle` installs a handler frame around the body by introducing a fresh continuation
atom for the frame:

```text
eval(Handle { clause, body, cont = k }) under chain H
  = let h = fresh_label() in
    eval(body[h / current_cont]) under chain HandlerFrame { clause, next: k, parent: H }
```

The body is lowered with `current_cont` bound to the fresh handler label `h`. Only the
distinguished current continuation is rewritten to `h`; arbitrary continuation atoms captured
from outer scopes are not rewritten. The handler frame intercepts `Raise` nodes that target
the current continuation. If the body completes normally (by `Jump` to `h`), the handler frame
forwards to `k`.

### 10.3 Raise Operational Semantics

`Raise` walks the current continuation chain to find a matching handler. The `resume` atom
on `Raise` is the captured raise-site continuation, including any handler frames that remain
active after the operation result is produced. Handler dispatch uses the chain `H` only to find
the matching frame.

```text
eval(Raise { op, args, resume = k_resume }) under chain H
  = find first matching HandlerFrame in H

If found (HandlerFrame { clause, next, parent }):
  - Build resume continuation:
      resume = Cont {
        arg: clause.op.result,
        body: Jump { cont: k_resume, arg: arg, row: k_resume.row },
        env: capture_env(k_resume),
        row: k_resume.row
      }
  - Evaluate clause.body under chain parent with args and resume

If not found:
  - Trap { reason: UnhandledEffect(op) }
```

**Handler body evaluation:** When a matching frame is selected, the handler body evaluates
under the chain outside the matching frame (`parent`). The selected frame is not active
while its own handler body runs. Effects raised by the handler body dispatch through `parent`
and any frames outside it. If recursive self-handling is desired, the handler must explicitly
reinstall itself.

**Resume construction:** The resume continuation is a `Cont` value that, when invoked with a
value `v`, jumps to `k_resume` (the original continuation from the raise site) with `v`.
The resume captures the environment from `k_resume`. The resume is one-shot: after the
handler jumps to it, the `Cont` is consumed.

**Nested handler behavior:**
- The resume continues with the original raise-site continuation `k_resume`.
- The matching handler frame is removed from the chain: the resume jumps directly to
  `k_resume`, which is the continuation that was current at the raise site.
- Outer handlers (closer to the root) are preserved because `k_resume` may itself be a
  handler frame or may be wrapped by outer handlers.
- Inner handlers (installed after the raise site) are part of `k_resume` and are preserved.
- The matching handler itself is not reinstalled: the resume bypasses it entirely.

### 10.4 Handler Dispatch

When a `Raise` node is evaluated:

1. Walk the current continuation chain from the current continuation.
2. Find the first `HandlerFrame` whose `clause.op` matches the raised `op`.
3. If found: invoke the handler body with the effect arguments and a resume continuation
   that reconstructs the rest of the chain.
4. If not found: evaluation traps with `Trap { reason: UnhandledEffect(op) }`.

If no matching handler is found, evaluation reaches `Trap { reason: UnhandledEffect(op) }`.
This is distinct from authority discharge failure. Missing capability authority is rejected
before the operation provider runs and is reported as `MissingAuthority` / `CapabilityDenied`,
not `UnhandledEffect`.

**Ambient authority as provider frames:** For the CPS IR operational model, ambient authority
is represented by provider frames installed at the runtime boundary. A `Raise` is serviced only
by a matching provider or handler frame; if no frame exists, evaluation traps with
`UnhandledEffect(op)`. There is no separate "ambient authority directly services the Raise"
path in the IR semantics — the runtime boundary installs the necessary frames before
execution begins.

## 11. Migration and IR Evolution

### 11.1 Two IRs During Migration

During the migration period, the compiler maintains **two IRs**:

1. **Legacy IR**: The current AST with `Act`, `Proc`, `Workflow`, `Do`, and other
   migration-era artifacts. This is what the current parser produces.
2. **Target IR (CPS)**: The new CPS form with `Atom`, `Value`, `Term`, `Call`, `Jump`, etc.

The lowering pipeline is:

```text
surface AST (legacy syntax)
    |
    v
Legacy IR (Act/Proc/Workflow/Do variants)
    |
    v
lower_to_cps.rs -- single pass lowering
    |
    v
Target IR (CPS: Atom/Value/Term)
    |
    v
type checker, optimizer, code generator
```

All semantic analysis operates on the Target IR only. The Legacy IR is a transient
representation that exists only between parsing and the CPS lowering pass.

### 11.2 Migration Completion Flag

Migration is complete when:

- The parser produces Target IR directly (no Legacy IR variants);
- The `lower_to_cps` pass becomes an identity function on Target IR;
- No code path constructs `Act`, `Proc`, `Workflow`, or `Do` AST nodes;
- The Legacy IR variants are removed from the `Expr` enum.

At that point, the compiler has a single IR: the CPS form. The presence of a non-identity
lowering pass is a clear flag that migration is still in progress.

### 11.3 Legacy IR to CPS Lowering Rules

During migration, legacy AST variants are lowered to CPS:

```text
legacy Expr::Act { ... }      -> CPS Lam { cont_param: k, body: [lowered], row: Act_profile }
legacy Expr::Do { target, ... } -> CPS Lam { cont_param: k, body: [lowered], row: target_profile }
legacy Expr::Proc { ... }     -> CPS Lam { cont_param: k, body: [lowered], row: Proc_profile }
legacy Expr::Workflow { ... } -> CPS Lam { cont_param: k, body: [lowered], row: Workflow_profile }
legacy Type::Fn { ... }       -> CPS CpsFn { params, cont: Cont { arg: ret, answer: Ans }, answer: Ans, body_row: {}, total_row: ρk }
legacy Type::Fun { ... }      -> CPS CpsFn { params, cont: Cont { arg: ret, answer: Ans }, answer: Ans, body_row: effect, total_row: effect ∪ ρk }
```

### 11.4 Dual Representation (Temporary)

A conforming implementation may maintain both representations during migration:

```rust
pub enum Expr {
    -- CPS representation (target)
    Lam { ... },
    Call { ... },
    Jump { ... },
    Raise { ... },
    Handle { ... },
    If { ... },
    LetVal { ... },
    LetPrim { ... },

    -- Legacy compatibility (to be removed after migration)
    Act { ... },
    Do { ... },
    Proc { ... },
    Workflow { ... },
}
```

The legacy variants are always lowered to CPS before semantic analysis. They are never
optimized, interpreted, or code-generated directly.

## 12. CPS Optimization Opportunities

The CPS form enables several standard optimizations:

1. **Contification**: Identify functions that are always called with a known continuation and
   inline the continuation into the function body.
2. **Administrative normal form (ANF)**: All intermediate values are named `LetVal` or
   `LetPrim` bindings, making dataflow analysis straightforward.
3. **Effect row propagation**: Effect rows flow through continuations, enabling precise
   effect-based dead code elimination and inlining decisions.
4. **Handler frame simplification**: Nested handlers for the same effect can be merged or
   reordered if their rows are compatible.
5. **Tail call optimization**: Every `Call` to a known function and every `Jump` to a
   continuation is a tail call by construction in CPS.

## 13. Open Decisions

1. Whether to use explicit labels or closures for continuations (labels enable better
   contification and compilation to machine code; closures are simpler for interpretation).
   **Current choice:** Labels are the default representation. Closures are used only when
   a continuation must be passed as a value. This is an implementation choice, not a
   semantic open question.
2. Whether the CPS IR is the canonical IR or an intermediate layer between a higher-level IR
   and a lower-level IR.
3. How to represent mutually recursive CPS functions (`LetRec` or fixpoint combinator).
4. Whether contract discharge status is stored in the IR or in a separate sidecar.
5. How row variables are represented in the CPS IR (names, indices, or de Bruijn indices).
6. Whether effect aliases are expanded during CPS lowering or preserved for diagnostics.
7. Whether to support direct-style fragments within CPS for performance-critical pure code.

## 14. See Also

- [SPEC-098a: Current IR](SPEC-098a-CURRENT-IR.md) — what the IR looks like today
- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)

## 15. Changelog

- 2026-06-18: Major revision after review. Split grammar into Atom/Value/Term. Added `Call`
  vs `Jump` distinction. Added answer type discipline. Separated callee/continuation/total
  rows. Made `Raise`/`Handle` operation-typed. Added contract discharge status. Rewrote
  examples into fully normalized core examples and schematic handler patterns. Marked
  laziness section as pseudo-IR.
