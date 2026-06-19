---
id: spec.ash.operational-semantics.target
title: Ash CPS IR Operational Semantics
description: Big-step operational semantics for the CPS IR interpreter
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-19
verified_against:
  specs:
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/plan/PLAN-159-CPS-IR-INTERPRETER.md
---

# SPEC-099b: Ash CPS IR Operational Semantics

**Status:** Draft — CPS IR operational semantics for the isolated prototype
**Scope:** This document defines the big-step operational semantics of the CPS IR interpreter implemented in Phase 159.
**Depends on:** SPEC-098b (Target IR), PLAN-159 (CPS IR Interpreter)

## §1 Syntax

### §1.1 Atoms

```text
a ::= Int(i) | Float(f) | String(s) | Bool(b) | Null | Var(x)
```

Atoms are primitive values or variable references. Variables are resolved in the environment.

### §1.2 Values

```text
v ::= Atom(a)
    | Lam { params: [x, ...], cont: k, body: t, row: ρ }
    | Cont { param: x, body: t, captured_env: η, row: ρ }
```

Values are inert data. `Lam` represents a function closure; `Cont` represents a continuation closure that captures its definition environment.

### §1.3 Terms

```text
t ::= LetVal { name: x, value: v, body: t }
    | LetPrim { name: x, op: ⊙, args: [a, ...], body: t }
    | LetCont { name: k, param: x, cont_body: t, body: t }
    | Jump { cont: κ, arg: a, row: ρ }
    | Call { func: a, args: [a, ...], cont: κ, row: ρ }
    | If { cond: a, then_branch: t, else_branch: t, row: ρ }
    | LetRec { name: x, value: v, body: t }
    | Raise { op: ε, args: [a, ...], resume: κ, row: ρ }
    | Handle { clause: h, body: t, cont: κ, row: ρ }
    | RecordDischarge { discharge: d, body: t }
    | Trap { reason: r }
```

Terms perform computation. All control flow is explicit via `Jump` and `Call`.

### §1.4 Continuation References

```text
κ ::= Label(k) | Var(x)
```

Labels are bound by `LetCont`; variables are bound in the environment.

### §1.5 Effect Rows

```text
ρ ::= EffectRow { items: [ι, ...] }

ι ::= EffectItem { namespace: ns, name: n, kind: τ }

τ ::= Capability | Role | Policy | Contract | Channel | Alias | Group
```

Effect rows track the effects a computation may perform. Rows are validated for duplicates.

## §2 Core Term Rules

### §2.1 LetVal

```text
eval(v, η) = v'
-----------------------------------
⟨LetVal(x, v, t), η, χ⟩ ⇓ ⟨t, η[x ↦ v'], χ⟩
```

Bind a value in the environment and continue.

### §2.2 LetPrim

```text
eval(aᵢ, η) = aᵢ'  for each i
eval_prim(⊙, [a₁', ..., aₙ']) = a'
-----------------------------------
⟨LetPrim(x, ⊙, [a₁, ..., aₙ], t), η, χ⟩ ⇓ ⟨t, η[x ↦ Atom(a')], χ⟩
```

Evaluate arguments (resolving variables), apply the primitive operation, bind the result, and continue.

### §2.3 LetCont

```text
c = Cont { param: x, body: t₁, captured_env: η, row: ρ }
-----------------------------------
⟨LetCont(k, x, t₁, t₂), η, χ⟩ ⇓ ⟨t₂, η[k ↦ c], χ⟩
```

Create a continuation closure capturing the current environment, bind it, and continue.

### §2.4 Jump

```text
eval(a, η) = a'
lookup(κ, η) = Cont { param: x, body: t, captured_env: η', ... }
-----------------------------------
⟨Jump(κ, a, ρ), η, χ⟩ ⇓ ⟨t, η'[x ↦ Atom(a')], χ⟩
```

Evaluate the argument, resolve the continuation, and execute the continuation body in the **captured** environment extended with the argument.

### §2.5 Call

```text
eval(a_f, η) = Var(x) or a_f
eval(aᵢ, η) = aᵢ'  for each i
lookup(func, η) = Lam { params: [p₁, ..., pₙ], cont: k, body: t, ... }
lookup(κ, η) = c
η' = η[p₁ ↦ Atom(a₁'), ..., pₙ ↦ Atom(aₙ')]
η'' = η'[k ↦ c]  if k ∉ dom(η')
-----------------------------------
⟨Call(a_f, [a₁, ..., aₙ], κ, ρ), η, χ⟩ ⇓ ⟨t, η'', χ⟩
```

Evaluate function and arguments. Create a new environment with parameters bound. Bind the continuation parameter **only if not already present** to preserve outer continuation references in nested continuations. Execute the lambda body.

### §2.6 Answer Type Discipline

All CPS terms are in **fixed answer type** discipline (Answer type): every term eventually reduces to a `Jump` to a continuation. There is no implicit return. The final result is produced by jumping to an exit continuation that traps with the result value.

## §3 Conditionals and Data Rules

### §3.1 If

```text
eval(a, η) = Bool(true)
-----------------------------------
⟨If(a, t₁, t₂, ρ), η, χ⟩ ⇓ ⟨t₁, η, χ⟩

eval(a, η) = Bool(false)
-----------------------------------
⟨If(a, t₁, t₂, ρ), η, χ⟩ ⇓ ⟨t₂, η, χ⟩

eval(a, η) ≠ Bool(_)
-----------------------------------
⟨If(a, t₁, t₂, ρ), η, χ⟩ ⇓ Stuck(InvalidCondition)
```

Evaluate the condition. If true, take the then branch; if false, take the else branch. Non-boolean conditions are an error.

### §3.2 RecordDischarge

```text
-----------------------------------
⟨RecordDischarge(d, t), η, χ⟩ ⇓ ⟨t, η, χ⟩
```

RecordDischarge is an administrative term that passes through to its body. In the full implementation, it would track contract discharge metadata.

## §4 Handler Rules

### §4.1 Raise (Handled)

```text
lookup_handler(ε, χ) = Some(Clause { op: ε, params: [p₁, ..., pₙ], resume: r, body: t, ... })
eval(aᵢ, η) = aᵢ'  for each i
lookup(κ, η) = c
η' = η[p₁ ↦ Atom(a₁'), ..., pₙ ↦ Atom(aₙ')]
η'' = η'[r ↦ c]
-----------------------------------
⟨Raise(ε, [a₁, ..., aₙ], κ, ρ), η, χ⟩ ⇓ ⟨t, η'', χ⟩
```

Find the innermost handler for the effect. Evaluate arguments, resolve the resume continuation, bind parameters and resume, then execute the handler body.

### §4.2 Raise (Unhandled)

```text
lookup_handler(ε, χ) = None
-----------------------------------
⟨Raise(ε, [a₁, ..., aₙ], κ, ρ), η, χ⟩ ⇓ Stuck(UnhandledEffect(ε))
```

If no handler is found, the computation is stuck with an unhandled effect error.

### §4.3 Handle

```text
lookup(κ, η) = c
χ' = χ :: Shallow(Clause { op: ε, ... })
-----------------------------------
⟨Handle(Clause { op: ε, ... }, t, κ, ρ), η, χ⟩ ⇓ ⟨t, η[r ↦ c], χ'⟩
```

Install a shallow handler frame on the handler chain. Bind the resume continuation in the environment. Execute the body with the extended handler chain.

### §4.4 Shallow Handler Removal

Shallow handler frames are removed after handling a single effect. The handler frame is pushed when `Handle` is entered and is not re-pushed after `Raise` is handled. This means a second `Raise` of the same effect will not find the same handler.

### §4.5 Provider Frame Persistence

Provider frames persist across resumes. When a provider frame is installed, it remains on the handler chain until explicitly removed or the computation completes.

### §4.6 Resume Construction

The resume continuation `c` passed to a handler clause is the continuation captured at the point of the `Raise`. When the handler invokes `resume` with a value, execution continues from the point after the `Raise` with the result value.

### §4.7 One-Shot Resume

Resume continuations are one-shot: they may be invoked at most once. Invoking a resume continuation more than once is undefined behavior in the current implementation.

### §4.8 Row Transformation

When a handler is installed, the effect row of the body is transformed to include the handler's effect item. When a handler handles an effect, the effect item is removed from the row. Provider frames add their effect item to the row and it persists until the provider frame is removed. This is the row transformation rule.

## §5 Recursion Rules

### §5.1 LetRec

```text
η' = η[x ↦ Null]
v' = eval(v, η')
η'' = η'[x ↦ v']
-----------------------------------
⟨LetRec(x, v, t), η, χ⟩ ⇓ ⟨t, η'', χ⟩
```

Bind the recursive name to `Null` as a placeholder, evaluate the value in the environment with the placeholder, then backfill the placeholder with the actual value. This allows the value to reference itself recursively.

### §5.2 Recursive Call Example

```text
letrec fact = (lam [n] k
  letprim is_zero = eq n 0 in
  if is_zero then
    (jump k 1)
  else
    letprim n_minus_1 = sub n 1 in
    letcont k_mul [result]
      (letprim prod = mul n result in (jump k prod))
    in (call fact [n_minus_1] k_mul))
in (call fact [5] exit)
```

The `LetRec` binds `fact` to the lambda. The lambda body references `fact` recursively. The continuation parameter `k` is preserved across recursive calls by the non-overwriting binding rule in `Call`.

## §6 Advanced and Row-Checker Rules

### §6.1 Trap

```text
-----------------------------------
⟨Trap(r), η, χ⟩ ⇓ Stuck(Trap(r))
```

Trap immediately halts computation with the given reason.

### §6.2 Row Validation

```text
validate_row(ρ) = Ok(())  if all items in ρ have distinct (namespace, name) pairs
validate_row(ρ) = Err(DuplicateItem(ns, n))  otherwise
```

Effect rows are validated for duplicate items. Two items are duplicates if they share the same namespace and name, regardless of kind.

### §6.3 Handler Chain Lookup

```text
lookup_handler(ε, []) = None

lookup_handler(ε, Shallow(Clause { op: ε', ... }) :: χ) =
  if ε.item == ε'.item then Some(Clause { op: ε', ... })
  else lookup_handler(ε, χ)

lookup_handler(ε, Provider { op: ε', ... } :: χ) =
  lookup_handler(ε, χ)  // provider frames don't have clauses
```

Handlers are searched from the innermost (top of stack) to outermost. Provider frames are skipped during clause lookup.

## §7 Worked Example: Factorial in CPS

```text
letcont exit [v] (trap return) in
letrec fact = (lam [n] k
  letprim is_zero = eq n 0 in
  if is_zero then
    (jump k 1)
  else
    letprim n_minus_1 = sub n 1 in
    letcont k_mul [result]
      (letprim prod = mul n result in (jump k prod))
    in (call fact [n_minus_1] k_mul))
in (call fact [5] exit)
```

**Execution trace:**

1. `exit` bound as continuation with body `trap return`
2. `fact` bound to lambda via `LetRec` (placeholder → backfill)
3. `Call fact [5] exit`: `n=5`, `k=exit`
4. `is_zero = eq 5 0 = false`
5. `If` takes else branch
6. `n_minus_1 = sub 5 1 = 4`
7. `k_mul` bound as continuation capturing env with `n=5`, `k=exit`
8. `Call fact [4] k_mul`: `n=4`, `k=k_mul` (but `exit` preserved for `k_mul` body)
9. ... recursion continues until `n=0` ...
10. Base case: `jump k 1` where `k` is the innermost continuation
11. Continuations unwind, multiplying results, until `exit` is reached with `120`

## §8 Deferrals

The following features are explicitly deferred outside PLAN-159 scope:

- Legacy AST lowering to CPS IR
- Lean 4 differential testing
- Bytecode compilation
- JIT compilation
- Mutual recursion (single `LetRec` only)
- Full row polymorphism (scaffold only)
- Effect aliases
- Full contract discharge

## §9 See Also

- [SPEC-098b: Target IR](SPEC-098b-TARGET-IR.md) — CPS IR syntax and types
- [PLAN-159: CPS IR Interpreter](../plan/PLAN-159-CPS-IR-INTERPRETER.md) — implementation plan
- [PLAN-INDEX](../plan/PLAN-INDEX.md) — task tracking

## §10 Changelog

- 2026-06-19: Rewrote with actual CPS IR semantics matching Phase 159 implementation. Added §2-7 with concrete rules for all term forms. Documented continuation capture, handler chain semantics, and LetRec backfill.
