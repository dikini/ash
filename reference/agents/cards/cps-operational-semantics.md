---
id: ref.agents.cps-operational-semantics
title: CPS Operational Semantics Agent Card
kind: agent-card
audience: [agent]
authority: derivative
canonical_page: ref.language.cps-operational-semantics
canonical_page_path: reference/language/cps-operational-semantics.md
status: current
stability: alpha
owner: language
last_verified: 2026-06-20
verified_against:
  git_commit: b7d6137f
refresh_trigger:
  - reference/language/cps-operational-semantics.md changes
---

# CPS Operational Semantics Agent Card

## Retrieval tags

operational-semantics, big-step, small-step, eval, judgment, rule, letval, letprim, letcont, jump, call, if, raise, handle, letrec, trap, record-discharge, handler-chain, resume, continuation, environment, stuck, deferral

## Stale-claim warnings

- Only **big-step semantics** is implemented. Do not claim small-step semantics exists.
- Do not claim legacy AST lowering, Lean differential testing, bytecode, or JIT.
- Do not claim native multi-binding `LetRec`. Phase 160 documents tuple-of-lambdas mutual recursion over single-binding `LetRec`.
- Do not claim full row polymorphism. Only duplicate validation exists.
- Effect aliases are not implemented.
- Full contract discharge is not implemented.

## Quick facts

- **Location**: `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md` (canonical spec)
- **Reference**: `reference/language/cps-operational-semantics.md` (this page's canonical)
- **Implementation**: `crates/ash-interp/src/cps.rs`
- **Test files**: `crates/ash-interp/tests/task_1591_cps_ir.rs` through `task_1596_cps_ir.rs`, plus `task_1616*_cps_ir_*.rs`
- **Plan**: `docs/plan/PLAN-159-CPS-IR-INTERPRETER.md`, `docs/plan/PLAN-160-CPS-IR-RUNTIME-EXPANSION.md`

## When to use this card

Use this card when:
- Understanding how a CPS term evaluates
- Debugging interpreter behavior against formal rules
- Adding new term forms or primitive operations
- Writing or reviewing operational semantics documentation
- Comparing implementation against formal rules
- Planning small-step semantics (future work)

## Key invariants

1. Big-step judgment: `⟨t, η, χ⟩ ⇓ r`
2. Every term eventually reduces to a `Jump` (fixed answer type discipline)
3. `Jump` uses the continuation's `captured_env`, not the current env
4. `Call` only binds the continuation parameter if not already present
5. `LetRec` uses placeholder/backfill: `Null` → evaluate → backfill
6. Handler chain search is innermost-first; provider frames are skipped
7. Shallow handlers are removed after one use; providers persist
8. Resume continuations are one-shot

## Rule reference

| Term | Rule | Key side condition |
|------|------|-------------------|
| `LetVal` | Bind evaluated value, continue | `eval(v, η) = v'` |
| `LetPrim` | Resolve args, apply prim, bind, continue | `eval_prim(⊙, args) = a'` |
| `LetCont` | Create cont with captured env, bind, continue | `captured_env = η` |
| `Jump` | Eval arg, resolve cont, run in captured env | Uses `η'`, not `η` |
| `Call` | Eval func, bind params, non-overwrite cont | `k ∉ dom(η')` |
| `If` | Eval cond, choose branch | Non-bool = error |
| `LetRec` | Placeholder → eval → backfill | `η[x ↦ Null]` first |
| `Raise` | Find handler, bind params+resume, execute | `lookup_handler(ε, χ)` |
| `Handle` | Push shallow frame, bind resume, execute | `χ' = χ :: Shallow(...)` |
| `Trap` | Halt with reason | `Stuck(Trap(r))` |

## Common debugging patterns

### Checking a rule application

When a test fails, trace the evaluation step by step:

```rust
// Add to eval_term or specific eval_* function
eprintln!("eval: {:?}", term);
eprintln!("env keys: {:?}", env.bindings.keys().collect::<Vec<_>>());
eprintln!("chain frames: {}", chain.frames.len());
```

### Verifying continuation capture

```rust
// After LetCont, check the captured env
if let Value::Cont { captured_env, .. } = cont_value {
    eprintln!("captured env keys: {:?}", captured_env.bindings.keys());
}
```

### Checking handler chain state

```rust
eprintln!("chain frames: {}", chain.frames.len());
for (i, frame) in chain.frames.iter().enumerate() {
    match frame {
        HandlerFrame::Shallow { clause } => {
            eprintln!("  [{}] Shallow: {:?}", i, clause.op.item);
        }
        HandlerFrame::Provider { op, .. } => {
            eprintln!("  [{}] Provider: {:?}", i, op.item);
        }
    }
}
```

## Related cards

- [CPS IR](cps-ir.md) — the intermediate representation
- [CPS Interpreter](cps-interpreter.md) — implementation details
- [The Ash Tower](../language/tower.md) — effect tower

## Edit preflight

Before modifying operational semantics:
1. Check `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md` for canonical rules
2. Write a failing test first (TDD)
3. Update both the spec and the reference page
4. Run `cargo test -p ash-interp --test task_159X_cps_ir` for relevant tests
5. Run `cargo clippy -p ash-interp --all-targets -- -D warnings`
6. Update this card if adding new rules or invariants

## Future work

- Small-step semantics: planned but not scheduled
- Relationship between big-step and small-step: big-step is the reference; small-step will be proven equivalent
- Concurrency semantics: deferred until Proc/process semantics is defined
