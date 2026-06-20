---
id: spec.ash.core-language
title: Ash Core Language — Canonical IR
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
---

# SPEC-099: Ash Core Language — Canonical IR

**Status:** Draft — formal specification of the minimal core language
**Scope:** The canonical IR that surface syntax desugars to. Core Ash lowers mechanically to the target CPS IR in SPEC-098b.
**Depends on:** SPEC-096b (Target Effect System), SPEC-097b (Target Type System), SPEC-098b (Target IR)

## 1. Summary

Core Ash is the canonical intermediate representation between surface syntax and CPS IR. It is a typed lambda calculus with effect rows, refinement predicates, explicit raised operations for the operation-like effect kinds, and enough structure for the type checker to attach contract and law evidence.

Core Ash is not the user-facing surface language. Surface forms such as `do`, `handle ... with`, `observe`, comprehensions, type-class constraints, laws, and properties are either desugared to core terms or recorded as compile-time metadata that can produce core refinements or discharge metadata.

Design goals:

1. **Minimal:** small enough to reason about formally.
2. **Mechanical lowering:** every core term has a direct lowering to SPEC-098b CPS IR.
3. **Dumb-executor compatible:** after lowering, a simple CPS executor can run the program by evaluating terms and dispatching `Raise`/`Handle`; it does not need to understand surface sugar, proof search, or law/property inference.
4. **Optimization-friendly:** administrative-normal form, explicit data dependencies, and explicit effect rows.
5. **Expressive:** full enough for Ash's type rows, dynamic contract discharge, refinement obligations, and compile-time evidence.

## 2. Syntactic layers

Core Ash has three layers:

- **Atoms** name things that need no evaluation.
- **Values** construct data or callable closures without performing effects.
- **Expressions** perform computation.

This mirrors SPEC-098b's `Atom` / `Value` / `Term` split, but Core Ash remains one step above CPS. Core expressions are mostly direct-style; continuation references appear only where the core has already exposed continuation control, such as command-pattern handler bodies.

## 3. Types

```text
Type ::= BaseType
       | Type -> {Row} Type              -- function with requirement row
       | RefinementType
       | ContType
       | TupleType
       | RecordType
       | TypeApp
       | TypeVar

BaseType ::= Int | String | Bool | Unit

RefinementType ::= Type | Predicate
ContType       ::= Cont<Type, Ans, Row, Multiplicity>
Multiplicity   ::= Affine | MultiShotPure
TupleType      ::= (Type, ...)
RecordType     ::= { Label: Type, ... }
TypeApp        ::= TypeName<Type, ...>
TypeVar        ::= Name

Row ::= {}
      | { RowItem, ... }
      | { RowItem, ... | RowVar }
      | { RowVar }

RowItem ::= cap Path[.Operation]
          | resource Path Mode
          | role Path
          | policy Path
          | contract ContractItem
          | channel Path Mode Type
          | proc Operation
          | fail [Type]
          | evidence Path
          | EffectGroupRef

Predicate ::= CorePredicateExpression
```

Rows are requirement rows, not authority grants. A closed row has no tail. An open row has a row variable tail.

`ContractViolation` is **not** a `RowItem`. Contract failures are represented by contract discharge metadata plus either `Trap { reason: ContractViolation(...) }` for unrecoverable failure or an explicit `fail`/`FailureEffect` path when a surface construct chooses recoverable failure behavior. This keeps SPEC-099 aligned with SPEC-096b and SPEC-098b: contracts are ambient-discharge items, not arbitrary raised operations.

### 3.1 Refinement predicates

A refinement type `T | P` means "a value of type `T` for which predicate `P` holds." The predicate is well-formed only if it type-checks as `Bool` in the refinement environment.

Refinement predicates may come from three sources:

1. static Hoare clauses (`requires`, `ensures`, `invariant`);
2. compile-time law evidence lowered to a refinement predicate;
3. explicit refinement annotations.

The core language contains the refinement predicate and its evidence reference, not the proof procedure that produced it. SMT solving, external proof checking, QuickCheck, and SmallCheck are separate compile-time passes.

### 3.2 Evidence references

A refinement or discharge record may carry an optional evidence reference:

```text
RefinementEvidence ::= {
    source: EvidenceSource,
    status: EvidenceStatus,
    predicate: Predicate,
    diagnostic: Option<DiagnosticId>
}

EvidenceSource ::= HoareClause | Law(NamePath) | ExternalProof(NamePath) | Assumption(NamePath)
EvidenceStatus ::= Proven | Disproved | Unknown | Statistical
```

Only `Proven` evidence may harden a type as a static refinement. `Disproved` evidence is a compile-time error. `Unknown` evidence may be demoted to a dynamic contract check. `Statistical` evidence is advisory and must not be used as a hard refinement.

Laws and properties are not core terms. A proven law may produce `RefinementEvidence { source: Law(...), status: Proven, ... }`; a property may produce `status: Statistical`, which can inform diagnostics but cannot satisfy a required refinement.

### 3.3 Continuation types

Core only exposes continuations when the source form intentionally exposes continuation control, such as Frank-style command-pattern handling. A continuation has the same semantic type as in SPEC-098b:

```text
Cont<A, Ans, ρ, Affine>
```

It consumes an `A`, produces the current answer type `Ans`, and may perform row `ρ` when resumed. Handler resume continuations are affine by default. The type checker must reject duplicate use, storage in ordinary data, or passing through unrestricted function arguments unless a future multi-shot rule proves the continuation pure and explicitly marks it `MultiShotPure`.

`MultiShotPure` is included here only as a type-shape hook for the design direction in `multi-shot-continuations.md`. It is not required for the initial canonical Core Ash implementation.

## 4. Atoms and values

```text
Atom ::= Var(Name)
       | Lit(Literal)
       | PrimName(PrimOp)
       | ConstructorName(Name)

ContRef ::= Label(LabelId) | Var(Name)

Value ::= Atom
        | Lam { params: Vec<Param>, body: Expr, row: Row }
        | Record { fields: Vec<(Name, Atom)> }
        | Tuple { elems: Vec<Atom> }
        | DischargeMarker { discharge: ContractDischarge }

Param ::= Name: Type
```

A `Lam` is direct-style in core. During lowering, the CPS pass adds a continuation parameter and lowers the body into a SPEC-098b `Term`.

`ContRef` is not a general atom. It may appear only in continuation positions such as `Jump`, or as the reference named by a handler resume parameter after lowering binds that parameter to a continuation closure.

`DischargeMarker` is administrative metadata. It records contract/evidence discharge information and lowers to SPEC-098b's `DischargeMarker` value or `RecordDischarge` term shape. It is not an ordinary user value and must not be stored or pattern-matched as program data.

`PrimOp` includes primitive arithmetic, comparison, field projection, tuple projection, constructor-tag tests, and other compiler-known pure operations. Pattern matching lowers through these primitives unless a future core revision admits `Match` directly.

## 5. Expressions

```text
Expr ::= Atom
       | LetVal { name: Name, ty: Type, value: Value, body: Expr }
       | LetRec { name: Name, ty: Type, value: Value, body: Expr }
       | LetPrim { name: Name, op: PrimOp, args: Vec<Atom>, body: Expr }
       | LetCall { name: Name, func: Atom, args: Vec<Atom>, body: Expr }
       | If { cond: Atom, then_branch: Expr, else_branch: Expr }
       | Call { func: Atom, args: Vec<Atom> }
       | Jump { cont: ContRef, arg: Atom }
       | Raise { op: EffectOp, args: Vec<Atom> }
       | Handle { clause: HandlerClause, body: Expr }
       | RecordDischarge { discharge: ContractDischarge, body: Expr }
       | Trap { reason: TrapReason }

HandlerClause ::= {
    op: EffectOp,
    params: Vec<Param>,
    resume: Param,        -- resume: Cont<OpResult, Ans, ρ_resume, Affine>
    body: Expr,
    row: Row              -- local row of handler body, excluding resume and outer cont rows
}

EffectOp ::= CapabilityOp | ChannelOp | ProcessOp | FailureOp

CapabilityOp ::= cap Path.Operation
ChannelOp    ::= channel Path Mode
ProcessOp    ::= proc Operation
FailureOp    ::= fail [Type]

ContractDischarge ::= {
    contract: ContractItem,
    mode: DischargeMode,
    evidence: Option<RefinementEvidence>,
    source_span: Option<SourceSpan>
}

DischargeMode ::= Static | Evidence | Dynamic

TrapReason ::= ContractViolation(ContractItem)
             | UnhandledEffect(EffectOp)
             | Panic(String)
             | NonExhaustiveMatch
```

### 5.1 Expression invariants

1. `Call`, `Raise`, and `Handle` are direct-style core forms. Their CPS continuation fields are synthesized during lowering.
2. `Jump` is already a continuation-level operation. Its target is a `ContRef`, not a general atom.
3. Handler `resume` parameters have continuation types and affine use rules.
4. Every intermediate non-atomic value is named with `LetVal`, `LetRec`, or `LetPrim`.
5. `Call` arguments and primitive arguments are atoms.
6. `Raise` names a typed operation. The operation declaration determines its argument types, result type, and local effect row.
7. `Handle` only handles raised operation kinds. It does not discharge ambient contract, evidence, role, policy, or resource items.
8. `RecordDischarge` records contract/evidence discharge metadata. It has no runtime behavior beyond preserving audit and diagnostic information.
9. `Trap` is an unrecoverable diagnostic abort. It is not row-accounted. Recoverable contract behavior must be modeled explicitly as `fail` if the language surface chooses recoverability.

## 6. Hoare clauses and dynamic contracts

### 6.1 Static Hoare clauses

Static Hoare clauses lower to refinement predicates plus discharge metadata:

```ash
-- Surface, proposed:
fn safe_div(a: Int, b: Int) -> Int
    requires: b != 0
    ensures: result * b == a
{
    a / b
}

-- Core shape:
Lam {
    params: [a: Int, b: Int | b != 0],
    row: {},
    body: RecordDischarge { discharge: ..., body: ... }
} : (Int, Int | b != 0) -> {} (Int | result * b == a)
```

The type checker checks refinement well-formedness and emits proof obligations. Proof search is not part of core evaluation.

### 6.2 Gradual verification

Static verification uses this result discipline:

| Proof result | Core consequence | Diagnostic consequence |
|--------------|------------------|------------------------|
| Proven | keep the refinement as static evidence; record `DischargeMode::Static` or `Evidence` | no runtime check required |
| Disproved | reject the program | emit an error with counterexample/evidence |
| Unknown | demote to dynamic contract check | emit a warning unless policy requires an error |
| Statistical | advisory only | emit informational diagnostic; do not harden the type |

### 6.3 Dynamic contract checks

A dynamic Hoare check lowers to a runtime predicate test plus contract discharge metadata. If the predicate fails, the default unrecoverable behavior is `Trap { reason: ContractViolation(contract) }`. A surface construct may instead choose an explicit recoverable failure path by lowering to `Raise { op: fail ..., ... }` and row-accounting the corresponding `fail` item.

```text
-- Pseudocode core shape:
RecordDischarge {
    discharge: ContractDischarge { mode: Dynamic, contract: requires(P), ... },
    body: If {
        cond: not(P),
        then_branch: Trap { reason: ContractViolation(requires(P)) },
        else_branch: body
    }
}
```

A function with a dynamic contract check does not automatically gain a special `ContractViolation` row item. If the check is unrecoverable, it traps and remains outside ordinary row accounting. If the check is recoverable, the lowering must use an explicit `fail` effect and include that failure item in the row.

## 7. Laws, properties, and evidence

Laws and properties are compile-time metadata. They do not appear in the core expression grammar.

A law may produce evidence:

```ash
-- Surface metadata, proposed:
law associativity<A> {
    forall x, y, z: List<A>.
    append(append(x, y), z) == append(x, append(y, z))
}
```

If the verifier proves the law, the compiler may produce evidence such as:

```text
RefinementEvidence {
    source: Law(std::list::append_associative),
    status: Proven,
    predicate: associative(std::list::append),
    diagnostic: None
}
```

That evidence may be attached to a refinement:

```ash
-- Surface, proposed:
fn fold<A>(xs: List<A>, init: A, op: (A, A) -> A) -> A
    requires: associative(op)
{
    ...
}

-- Core type shape:
op: ((A, A) -> {} A) | associative(op)
```

A disproved law is a compile-time error. An unknown law may produce a warning and optional dynamic or test obligations. Statistical property evidence is advisory; it must not satisfy a hard refinement.

Evidence that affects auditability or diagnostics lowers to `ContractDischarge` metadata, `DischargeMarker`, `RecordDischarge`, or sidecar records keyed by the relevant core item. It does not lower to ordinary user-visible `Evidence` values.

## 8. Effect operations

### 8.1 Effect declarations

This Core Ash draft does not specify arbitrary user-defined resumable algebraic effects. SPEC-096b explicitly leaves those out of scope.

Upper-layer declarations may still provide user-friendly names for known operation kinds. For example, a surface `Console` declaration can lower to capability operations:

```ash
-- Surface, proposed upper-layer declaration:
effect Console
  fun print(msg: String) : Unit lowers_to cap stdout.write
  fun println(msg: String) : Unit lowers_to cap stdout.write
```

The core operation identities remain known SPEC-096b/SPEC-098b operation kinds:

```text
EffectOp(cap stdout.write) : (String) -> Unit, row {cap stdout.write}
```

A future spec may add arbitrary user-defined resumable effects. That extension must update SPEC-096b, SPEC-097b, SPEC-098b, and this spec together.

### 8.2 Raise

```text
Raise { op: cap stdout.write, args: ["hello"] }
```

During CPS lowering, `Raise` receives:

- `resume`: the current continuation;
- `row`: the local operation row from the operation declaration;
- argument arity and type checks from the operation signature.

### 8.3 Handle

```text
Handle {
    clause: {
        op: cap stdout.write,
        params: [msg: String],
        resume: resume_k,
        row: {},
        body: Jump { cont: resume_k, arg: () }
    },
    body: action
}
```

The clause matches an operation by canonical operation identity. The clause body receives operation arguments and an affine resume continuation. Koka-style surface handlers can hide the continuation behind `resume`; Frank-style command patterns can bind it explicitly as `k`. Both lower to this core shape only for operation kinds that SPEC-098b can represent as `EffectOp`.

## 9. Diagnostics and user-facing errors

Every compiler and runtime phase that can reject or warn must produce a structured diagnostic. This is separate from contract discharge behavior: diagnostics explain what went wrong to the user; contract discharge metadata records how a contract was checked or failed.

```text
Diagnostic ::= {
    id: DiagnosticId,
    severity: Error | Warning | Info,
    phase: Parser | Lowering | TypeChecker | ContractVerifier | LawVerifier | Runtime,
    code: DiagnosticCode,
    message: String,
    primary_span: Option<SourceSpan>,
    labels: Vec<DiagnosticLabel>,
    notes: Vec<String>,
    help: Vec<String>,
    evidence: Option<RefinementEvidence>
}
```

Rules:

1. Parser, lowering, type-checker, and verifier failures emit compile-time diagnostics.
2. Runtime contract failures may produce a runtime diagnostic record and then trap or raise an explicit failure effect, depending on the lowering choice.
3. Disproved static refinements and disproved laws are `Error` diagnostics.
4. Unknown proof obligations that demote to dynamic checks are `Warning` diagnostics unless the build profile requires static proof.
5. Statistical evidence from properties is `Info` or `Warning`, depending on project policy.
6. Diagnostics must preserve source spans through surface desugaring and core lowering.

This diagnostic shape is part of the core-language contract because refinements, law evidence, dynamic contract checks, parser errors, lowering errors, and runtime failures all need a common way to explain errors and warnings.

## 10. Comonads

Comonadic operations are ordinary core functions. Core Ash does not need a special `observe` expression.

```text
type Stream<A> = { head: A, tail: Stream<A> }

extract : Stream<A> -> {} A
extend  : (Stream<A> -> {} B, Stream<A>) -> {} Stream<B>
```

Surface `observe` desugars to `extend`:

```ash
-- Surface, proposed:
observe s { let x = head; let y = tail.head; (x + y) / 2.0 }

-- Core:
Call extend [
    Lam { params: [ctx: Stream<Float>], row: {}, body: ... },
    s
]
```

## 11. Pattern matching

Core Ash does not require a primitive `match` expression in this draft. Surface pattern matching lowers to decision trees using:

- constructor tag tests (`PrimOp::IsConstructor`);
- field and tuple projections;
- nested `If` expressions;
- `Trap` for unreachable branches after exhaustiveness checking, or compile-time diagnostics for non-exhaustive matches.

Example:

```ash
-- Surface, proposed:
match xs {
    Nil => 0,
    Cons(head, tail) => 1 + length(tail)
}
```

lowers to core shaped like:

```text
If IsConstructor(xs, Nil) then
    0
else if IsConstructor(xs, Cons) then
    LetPrim head = ConstructorField(xs, 0) in
    LetPrim tail = ConstructorField(xs, 1) in
    let len_tail = Call length [tail] in
    Call add [1, len_tail]
else
    Trap { reason: NonExhaustiveMatch }
```

The example is schematic: ANF conversion names nested calls before they appear as arguments.

## 12. Lowering to CPS IR

### 12.1 Translation notation

This spec writes `⟦e⟧` for "the CPS lowering of core expression `e`." It does not mean a list literal.

### 12.2 Mechanical translation

| Core | CPS IR |
|------|--------|
| `LetVal { name, value, body }` | `LetVal { name, value: ⟦value⟧, body: ⟦body⟧ }` |
| `LetRec { name, value, body }` | `LetRec { name, value: ⟦value⟧, body: ⟦body⟧ }` |
| `LetPrim { name, op, args, body }` | `LetPrim { name, op, args, body: ⟦body⟧ }` |
| `LetCall { name, func, args, body }` | `LetCont { name: fresh_k, param: name, cont_body: ⟦body⟧, body: Call { func, args, cont: fresh_k, row: call_total_row } }` |
| `If { cond, then, else }` | `If { cond, then_branch: ⟦then⟧, else_branch: ⟦else⟧, row }` |
| `Call { func, args }` | `Call { func, args, cont: current_cont, row: call_total_row }` |
| `Jump { cont, arg }` | `Jump { cont, arg, row: cont_row }` |
| `Raise { op, args }` | `Raise { op, args, resume: current_cont, row: op_row }` |
| `Handle { clause, body }` | `Handle { clause: ⟦clause⟧, body: ⟦body⟧, cont: current_cont, row: handle_local_residual_row }` |
| `RecordDischarge { discharge, body }` | `RecordDischarge { discharge, body: ⟦body⟧ }` |
| `Trap { reason }` | `Trap { reason }` |
| `Lam { params, body, row }` | `Lam { params, cont_param: fresh_k, body: ⟦body⟧ under fresh_k, row }` |
| `DischargeMarker { discharge }` | `DischargeMarker { discharge }` |

### 12.3 Field synthesis

Core is direct-style. The lowering pass synthesizes CPS fields as follows:

- `current_cont` is the continuation for the current direct-style context.
- `Call.row` is the union of the callee's body row and `current_cont`'s row, as specified by SPEC-098b.
- `Raise.resume` is `current_cont`.
- `Raise.row` is the local operation row of the raised operation.
- `Handle.cont` is `current_cont`.
- `Handle.row` is the **local residual body row** after removing the handled operation and adding handler effects. It excludes `Handle.cont`'s row. The total row of the lowered `Handle` term is `Handle.row ∪ row(Handle.cont)`, exactly as in SPEC-098b §5.4.
- `If.row` is the local union of branch rows, excluding any continuation effects already accounted separately by the enclosing CPS context.
- `Jump.row` is the row of the target continuation.

These fields are deterministic products of the typing environment and the current lowering context. A dumb CPS executor receives fully materialized SPEC-098b terms and does not need to recompute them.

### 12.4 Continuation introduction

Every core function lowers to a CPS lambda with a continuation parameter:

```text
-- Core:
fn add(a: Int, b: Int) -> {} Int { a + b }

-- CPS shape:
add = Lam { params: [a, b], cont_param: k, row: {}, body:
    LetPrim { name: sum, op: Add, args: [a, b], body:
        Jump { cont: k, arg: sum, row: row(k) }
    }
}
```

Every non-tail call introduces a continuation label or continuation closure:

```text
-- Core:
LetCall { name: x, func: add, args: [1, 2], body }

-- CPS shape:
LetCont { name: k_x, param: x, cont_body: ⟦body⟧, body:
    Call { func: add, args: [1, 2], cont: Label(k_x), row: ... }
}
```

`LetCont` is not a core source construct in this draft. It is introduced by CPS lowering.

## 13. Relationship to surface syntax

| Surface | Core |
|---------|------|
| `do { x <- f; return g }` | direct-style calls and raises with ANF bindings |
| `handle action with { op -> body }` | `Handle { clause, body: action }` for representable raised operations |
| Frank-style command pattern `<op args -> k>` | `Handle` clause with explicit affine `resume` binding |
| `observe s { ... }` | `Call extend [Lam(...), s]` |
| `[f(x,y) | x <- xs, y <- ys]` | calls to `flatMap` and `map` |
| co-comprehension syntax | calls to `extend`, `zipWith`, or library combinators |
| `fn sort<A: Ord>(...)` | dictionary argument, e.g. `ord_dict: OrdDict<A>` |
| `law ...` | compile-time metadata; may produce refinement or discharge evidence |
| `property ...` | compile-time/test metadata; may produce advisory evidence |

Dictionary lowering is an upper-layer-to-Core translation note. It is not a commitment that Core Ash itself has type classes, higher-kinded types, or an ad-hoc-polymorphism solver.

Core Ash does not include surface sugar as primitive syntax. It includes only the constructs needed for type checking, evidence attachment, discharge metadata, and CPS lowering.

## 14. Relationship to other specs

| Spec | Role |
|------|------|
| SPEC-095b (Target Grammar) | Surface syntax with sugar |
| SPEC-096b (Target Effect System) | Effect taxonomy and discharge |
| SPEC-097b (Target Type System) | Row polymorphism, subtyping, refinement obligations |
| **SPEC-099 (Core Language)** | **Canonical IR (this spec)** |
| SPEC-098b (Target IR) | CPS IR lowering target |

## 15. Open questions

1. Should a future core revision admit `Match` directly for better diagnostics and optimization, while still lowering to decision trees?
2. Should `Trap` remain in core, or should it be introduced only by lowering after exhaustiveness and reachability checks?
3. Should `RefinementEvidence` remain type/discharge metadata only, or should some evidence become inspectable through a separate reflection API?
4. How much of the diagnostic shape belongs in SPEC-099 versus a dedicated diagnostics spec?
5. If arbitrary user-defined resumable effects are added later, which exact EffectItem namespace and handler-dispatch semantics do they lower to?

## Changelog

- 2026-06-20: Created formal specification for Core Ash — the canonical IR between surface syntax and CPS IR.
- 2026-06-20: Clarified CPS field synthesis, dynamic contract discharge, law evidence lowered to refinements, diagnostics, pattern-match desugaring, and `LetCont` introduction during lowering.
- 2026-06-20: Reconciled SPEC-099 with SPEC-098b/SPEC-096b by removing `ContractViolation` as a row item/raised operation, making `Handle.row` local residual only, specifying affine continuation typing for handler resumes, treating user-defined resumable effects as out of scope, and lowering evidence to discharge metadata or sidecar records rather than ordinary values.
