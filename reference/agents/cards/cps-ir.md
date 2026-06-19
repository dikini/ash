---
id: ref.agents.cps-ir
title: CPS IR Agent Card
kind: agent-card
audience: [agent]
authority: derivative
canonical_page: ref.language.cps-ir
canonical_page_path: reference/language/cps-ir.md
status: current
stability: alpha
owner: language
last_verified: 2026-06-19
verified_against:
  git_commit: b7d6137f
refresh_trigger:
  - reference/language/cps-ir.md changes
---

# CPS IR Agent Card

## Retrieval tags

cps, ir, intermediate-representation, continuation-passing-style, eval, interpreter, term, value, atom, lam, cont, jump, call, letval, letprim, letcont, letrec, if, raise, handle, effect, handler, resume, row, serialization, sexp, json

## Stale-claim warnings

- The CPS IR is an isolated prototype. Do not claim it is connected to the legacy AST or Lean differential testing.
- Do not claim bytecode compilation or JIT is implemented. Only the interpreter exists.
- Do not claim mutual recursion is supported. Only single `LetRec` works.
- Do not claim full row polymorphism. Only duplicate validation exists.
- Effect aliases are not implemented.
- Full contract discharge is not implemented.

## Quick facts

- **Location**: `crates/ash-core/src/cps.rs` (types), `crates/ash-core/src/sexp.rs` (serialization)
- **Entry point**: `ash_interp::cps::eval_term(term, env, chain)`
- **Test files**: `crates/ash-interp/tests/task_159*_cps_ir.rs`
- **Spec**: `docs/spec/SPEC-098b-TARGET-IR.md`
- **Semantics**: `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`
- **Plan**: `docs/plan/PLAN-159-CPS-IR-INTERPRETER.md`

## When to use this card

Use this card when:
- Implementing or modifying CPS IR data structures
- Writing CPS IR test fixtures
- Debugging interpreter behavior
- Adding new term forms or primitive operations
- Working on serialization (JSON or S-expressions)
- Understanding handler chain semantics

## Key invariants

1. Values are inert; terms perform computation.
2. Every term eventually reduces to a `Jump`.
3. Continuations capture their definition environment (`captured_env`).
4. Handler chain search is innermost-first.
5. Shallow handlers are removed after one use; providers persist.
6. The `Call` non-overwriting binding rule preserves outer continuation references.
7. `LetRec` uses placeholder/backfill for recursive binding.

## Common patterns

### Creating a simple CPS program

```rust
use ash_core::cps::*;

let term = Term::LetVal {
    name: "x".to_string(),
    value: Value::Atom(Atom::Int(42)),
    body: Box::new(Term::Jump {
        cont: ContRef::Label("exit".to_string()),
        arg: Atom::Var("x".to_string()),
        row: EffectRow::default(),
    }),
};
```

### Evaluating a CPS program

```rust
use ash_core::cps::*;
use ash_interp::cps::{eval_term, CpsError};

let mut env = Env::new();
let mut chain = HandlerChain::new();
match eval_term(&term, &mut env, &mut chain) {
    Err(CpsError::Trap(TrapReason::Custom(reason))) => {
        // Program completed with result
    }
    Err(e) => panic!("Error: {:?}", e),
    Ok(atom) => panic!("Unexpected return: {:?}", atom),
}
```

### Serializing to S-expression

```rust
use ash_core::sexp::{term_to_string, string_to_term};

let sexp = term_to_string(&term).unwrap();
let roundtripped = string_to_term(&sexp).unwrap();
assert_eq!(term, roundtripped);
```

## Related cards

- [CPS Interpreter](cps-interpreter.md) — how the interpreter works
- [Stdlib Act](stdlib-act.md) — effectful operations
- [Stdlib Proc](stdlib-proc.md) — process management
- [The Ash Tower](../language/tower.md) — effect tower

## Edit preflight

Before modifying CPS IR types:
1. Check `docs/spec/SPEC-098b-TARGET-IR.md` for the canonical type definitions
2. Run `cargo test -p ash-core -p ash-interp` to ensure tests pass
3. Update serialization tests if adding new variants
4. Update this card if adding new invariants or patterns
