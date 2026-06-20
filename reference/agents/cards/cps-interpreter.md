---
id: ref.agents.cps-interpreter
title: CPS Interpreter Agent Card
kind: agent-card
audience: [agent]
authority: derivative
canonical_page: ref.runtime.cps-interpreter
canonical_page_path: reference/runtime/cps-interpreter.md
status: current
stability: alpha
owner: runtime
last_verified: 2026-06-20
verified_against:
  git_commit: b7d6137f
refresh_trigger:
  - reference/runtime/cps-interpreter.md changes
---

# CPS Interpreter Agent Card

## Retrieval tags

cps, interpreter, eval, eval_term, eval_letval, eval_letprim, eval_letcont, eval_jump, eval_call, eval_if, eval_letrec, eval_raise, eval_handle, primitive, handler, chain, continuation, resume, environment, trap, error

## Stale-claim warnings

- The interpreter is the only execution path. Do not claim bytecode or JIT exists.
- Do not claim native multi-binding `LetRec` exists. The interpreter handles Phase 160 tuple-of-lambdas mutual recursion through single-binding `LetRec`.
- Do not claim full row polymorphism. Only duplicate validation exists.
- Effect aliases are not implemented.
- Full contract discharge is not implemented.
- Legacy AST lowering is not implemented.

## Quick facts

- **Location**: `crates/ash-interp/src/cps.rs`
- **Entry point**: `eval_term(term, env, chain)`
- **Test files**: `crates/ash-interp/tests/task_1591_cps_ir.rs` through `task_1596_cps_ir.rs`, plus `task_1616*_cps_ir_*.rs`
- **Semantics**: `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`, `docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md`
- **Plan**: `docs/plan/PLAN-159-CPS-IR-INTERPRETER.md`, `docs/plan/PLAN-160-CPS-IR-RUNTIME-EXPANSION.md`

## When to use this card

Use this card when:
- Implementing or modifying the interpreter
- Adding new primitive operations
- Debugging evaluation behavior
- Understanding handler chain semantics
- Working on effect dispatch or resume continuations
- Writing interpreter tests

## Key invariants

1. `eval_term` is a thin dispatcher — each term variant has its own `eval_*` function.
2. `Env` is immutable: every binding creates a new frame.
3. `HandlerChain` is searched innermost-first.
4. `Cont` captures `captured_env` — jumps use the captured env, not the current env.
5. `Call` only binds the continuation parameter if not already present.
6. `LetRec` uses placeholder/backfill pattern.
7. `Trap` is the final result mechanism.

## Per-term evaluator reference

| Term | Evaluator | Key behavior |
|------|-----------|-------------|
| `LetVal` | `eval_letval` | Evaluate value, bind, continue |
| `LetPrim` | `eval_letprim` | Resolve args, apply prim, bind, continue |
| `LetCont` | `eval_letcont` | Create cont with captured env, bind, continue |
| `Jump` | `eval_jump` | Resolve cont, execute in captured env |
| `Call` | `eval_call` | Resolve func, bind params, non-overwrite cont |
| `If` | `eval_if` | Eval cond, choose branch |
| `LetRec` | `eval_letrec` | Placeholder/backfill |
| `Raise` | `eval_raise` | Find handler, bind params+resume, execute clause |
| `Handle` | `eval_handle` | Push shallow frame, bind resume, execute body |
| `Trap` | — | Return `CpsError::Trap` |

## Common debugging patterns

### Tracing evaluation

Add print statements in `eval_term` or specific `eval_*` functions to trace execution:

```rust
fn eval_term(term: &Term, env: &Env, chain: &HandlerChain) -> CpsResult<Atom> {
    eprintln!("eval: {:?}", term);
    match term { ... }
}
```

### Checking handler chain state

```rust
eprintln!("chain frames: {}", chain.frames.len());
for (i, frame) in chain.frames.iter().enumerate() {
    eprintln!("  [{}]: {:?}", i, frame);
}
```

### Verifying environment bindings

```rust
eprintln!("env bindings: {:?}", env.bindings.keys().collect::<Vec<_>>());
```

## Related cards

- [CPS IR](cps-ir.md) — the intermediate representation types
- [RuntimeKernel](runtime-kernel.md) — runtime kernel
- [Stdlib Act](stdlib-act.md) — effectful operations
- [The Ash Tower](../language/tower.md) — effect tower

## Edit preflight

Before modifying the interpreter:
1. Check `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md` for the canonical semantics
2. Write a failing test first (TDD)
3. Run `cargo test -p ash-interp --test task_159X_cps_ir` for the relevant test file
4. Run `cargo clippy -p ash-interp --all-targets -- -D warnings`
5. Update this card if adding new evaluators or invariants
