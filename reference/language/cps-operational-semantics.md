---
id: ref.language.cps-operational-semantics
title: CPS Operational Semantics
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
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
  code:
    - crates/ash-interp/src/cps.rs
  tests:
    - crates/ash-interp/tests/task_1591_cps_ir.rs
    - crates/ash-interp/tests/task_1592_cps_ir.rs
    - crates/ash-interp/tests/task_1593_cps_ir.rs
    - crates/ash-interp/tests/task_1594_cps_ir.rs
    - crates/ash-interp/tests/task_1595_cps_ir.rs
    - crates/ash-interp/tests/task_1596_cps_ir.rs
  examples:
    - crates/ash-interp/tests/task_1596_cps_ir.rs
refresh_trigger:
  - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md changes
  - crates/ash-interp/src/cps.rs changes
---

# CPS Operational Semantics

## Summary

The Ash CPS IR operational semantics defines how CPS terms evaluate to results. The current implementation uses **big-step semantics** — each rule describes the complete evaluation of a term in one step. Small-step semantics (reduction semantics) is planned for future work.

This document is a self-contained reference for programmers and agents who need to understand how Ash CPS IR terms evaluate. It references the canonical spec (SPEC-099b) but does not require reading it first.

## Big-step vs small-step

**Big-step semantics** (current): each rule maps a term directly to its final result. Good for proving correctness and understanding the complete evaluation of a term. Used by the Phase 159 interpreter.

**Small-step semantics** (deferred): each rule performs a single reduction step, producing a new term. Good for modeling concurrency, debugging, and step-by-step execution. Not yet implemented.

## Evaluation judgment

The big-step judgment has the form:

```text
⟨t, η, χ⟩ ⇓ r
```

Where:
- `t` is the term being evaluated
- `η` (eta) is the environment (immutable frame stack)
- `χ` (chi) is the handler chain (explicit frame stack)
- `r` is the result: either a value or a stuck state

Results are defined as:

```text
r ::= Atom(a)                    // final value
    | Stuck(Trap(r))           // trapped with reason
    | Stuck(UnhandledEffect(ε)) // unhandled effect
    | Stuck(InvalidCondition)  // non-boolean condition
    | Stuck(ExpectedLambda(v)) // call to non-lambda
    | Stuck(ExpectedContinuation(v)) // jump to non-continuation
```

## Core term rules

### LetVal

Evaluate the value, bind it in the environment, and continue with the body.

```text
⟨v, η⟩ ⇓ v'
-----------------------------------
⟨LetVal(x, v, t), η, χ⟩ ⇓ ⟨t, η[x ↦ v'], χ⟩
```

In Rust:

```rust
let evaluated = eval_value(value, env)?;
let new_env = env.clone().with_binding(name.clone(), evaluated);
eval_term(body, &new_env, chain)
```

### LetPrim

Evaluate arguments (resolving variables), apply the primitive operation, bind the result, and continue.

```text
⟨aᵢ, η⟩ ⇓ aᵢ'  for each i
eval_prim(⊙, [a₁', ..., aₙ']) = a'
-----------------------------------
⟨LetPrim(x, ⊙, [a₁, ..., aₙ], t), η, χ⟩ ⇓ ⟨t, η[x ↦ Atom(a')], χ⟩
```

In Rust:

```rust
let resolved: Vec<Atom> = args.iter().map(|a| eval_atom(a, env)).collect()?;
let result = eval_prim(op, &resolved)?;
let new_env = env.clone().with_binding(name.clone(), Value::Atom(result));
eval_term(body, &new_env, chain)
```

### LetCont

Create a continuation closure capturing the current environment, bind it, and continue.

```text
c = Cont { param: x, body: t₁, captured_env: η, row: ρ }
-----------------------------------
⟨LetCont(k, x, t₁, t₂), η, χ⟩ ⇓ ⟨t₂, η[k ↦ c], χ⟩
```

The `captured_env` is essential: when the continuation is later invoked via `Jump`, it runs in this captured environment, not the environment at the jump site. This preserves lexical scoping.

### Jump

Evaluate the argument, resolve the continuation, and execute the continuation body in the **captured** environment.

```text
⟨a, η⟩ ⇓ a'
lookup(κ, η) = Cont { param: x, body: t, captured_env: η', ... }
-----------------------------------
⟨Jump(κ, a, ρ), η, χ⟩ ⇓ ⟨t, η'[x ↦ Atom(a')], χ⟩
```

Note: the continuation body runs in `η'` (captured env), not `η` (current env).

### Call

Evaluate function and arguments, bind parameters, and execute the lambda body.

```text
⟨a_f, η⟩ ⇓ v_f
⟨aᵢ, η⟩ ⇓ aᵢ'  for each i
lookup(a_f, η) = Lam { params: [p₁, ..., pₙ], cont: k, body: t, ... }
lookup(κ, η) = c
η' = η[p₁ ↦ Atom(a₁'), ..., pₙ ↦ Atom(aₙ')]
η'' = η'[k ↦ c]  if k ∉ dom(η')
-----------------------------------
⟨Call(a_f, [a₁, ..., aₙ], κ, ρ), η, χ⟩ ⇓ ⟨t, η'', χ⟩
```

The non-overwriting binding rule (`if k ∉ dom(η')`) is critical for recursive functions with nested continuations. When a recursive call overwrites the lambda's continuation parameter `k`, any nested continuation that references `k` must still find the outer `k`, not the recursive call's `k`.

## Conditional rules

### If

```text
⟨a, η⟩ ⇓ Bool(true)
-----------------------------------
⟨If(a, t₁, t₂, ρ), η, χ⟩ ⇓ ⟨t₁, η, χ⟩

⟨a, η⟩ ⇓ Bool(false)
-----------------------------------
⟨If(a, t₁, t₂, ρ), η, χ⟩ ⇓ ⟨t₂, η, χ⟩

⟨a, η⟩ ⇓ v where v ≠ Bool(_)
-----------------------------------
⟨If(a, t₁, t₂, ρ), η, χ⟩ ⇓ Stuck(InvalidCondition)
```

Non-boolean conditions are an error.

## Handler rules

### Raise (handled)

```text
lookup_handler(ε, χ) = Some(Clause { op: ε, params: [p₁, ..., pₙ], resume: r, body: t, ... })
⟨aᵢ, η⟩ ⇓ aᵢ'  for each i
lookup(κ, η) = c
η' = η[p₁ ↦ Atom(a₁'), ..., pₙ ↦ Atom(aₙ')]
η'' = η'[r ↦ c]
-----------------------------------
⟨Raise(ε, [a₁, ..., aₙ], κ, ρ), η, χ⟩ ⇓ ⟨t, η'', χ⟩
```

Find the innermost handler for the effect, bind parameters and resume continuation, then execute the handler body.

### Raise (unhandled)

```text
lookup_handler(ε, χ) = None
-----------------------------------
⟨Raise(ε, [a₁, ..., aₙ], κ, ρ), η, χ⟩ ⇓ Stuck(UnhandledEffect(ε))
```

If no handler is found, the computation is stuck.

### Handle

```text
lookup(κ, η) = c
χ' = χ :: Shallow(Clause { op: ε, ... })
-----------------------------------
⟨Handle(Clause { op: ε, ... }, t, κ, ρ), η, χ⟩ ⇓ ⟨t, η[r ↦ c], χ'⟩
```

Install a shallow handler frame on the handler chain. Bind the resume continuation. Execute the body with the extended chain.

### Handler chain lookup

```text
lookup_handler(ε, []) = None

lookup_handler(ε, Shallow(Clause { op: ε', ... }) :: χ) =
  if ε == ε' then Some(Clause { op: ε', ... })
  else lookup_handler(ε, χ)

lookup_handler(ε, Provider { op: ε', ... } :: χ) =
  lookup_handler(ε, χ)  // provider frames don't have clauses
```

Handlers are searched from innermost (top of stack) to outermost. Provider frames are skipped.

### Shallow handler removal

Shallow handler frames are removed after handling a single effect. The frame is pushed when `Handle` is entered and is not re-pushed after `Raise` is handled. A second `Raise` of the same effect will not find the same handler.

### Provider frame persistence

Provider frames persist across resumes. They remain on the handler chain until explicitly removed or the computation completes.

### Resume construction

The resume continuation passed to a handler clause is the continuation captured at the point of the `Raise`. When the handler invokes `resume` with a value, execution continues from after the `Raise` with the provided value.

### One-shot resume

Resume continuations may be invoked at most once. Invoking a resume continuation more than once is undefined behavior in the current implementation.

## Recursion rules

### LetRec

```text
η' = η[x ↦ Null]
⟨v, η'⟩ ⇓ v'
η'' = η'[x ↦ v']
-----------------------------------
⟨LetRec(x, v, t), η, χ⟩ ⇓ ⟨t, η'', χ⟩
```

Bind the recursive name to `Null` as a placeholder, evaluate the value in the environment with the placeholder, then backfill with the actual value. This allows the value to reference itself recursively.

## Administrative rules

### RecordDischarge

```text
-----------------------------------
⟨RecordDischarge(d, t), η, χ⟩ ⇓ ⟨t, η, χ⟩
```

Passes through to its body. In the full implementation, it would track contract discharge metadata.

### Trap

```text
-----------------------------------
⟨Trap(r), η, χ⟩ ⇓ Stuck(Trap(r))
```

Immediately halts computation with the given reason.

## Row-checking rules

### Row validation

```text
validate_row(ρ) = Ok(())  if all items have distinct (namespace, name) pairs
validate_row(ρ) = Err(DuplicateItem(ns, n))  otherwise
```

Effect rows are validated for duplicate items. Same namespace+name with different kinds is still a duplicate.

## Example: factorial evaluation

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

**Trace:**

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

## Deferrals

The following are explicitly deferred:

- **Small-step semantics**: planned for future work; big-step is the current reference
- **Legacy AST lowering**: future phase
- **Lean 4 differential testing**: future phase
- **Bytecode compilation**: future phase
- **JIT compilation**: future phase
- **Mutual recursion**: single `LetRec` only
- **Full row polymorphism**: duplicate checking only
- **Effect aliases**: not implemented
- **Full contract discharge**: not implemented

## See also

- [CPS IR](cps-ir.md) — the intermediate representation types
- [CPS Interpreter](cps-interpreter.md) — how the interpreter implements these rules
- [SPEC-099b: Target Operational Semantics](../../docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — canonical formal semantics
- [PLAN-159: CPS IR Interpreter](../../docs/plan/PLAN-159-CPS-IR-INTERPRETER.md) — implementation plan
