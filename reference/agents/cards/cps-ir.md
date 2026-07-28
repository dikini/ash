---
id: ref.agents.cps-ir
title: CPS IR Agent Card
kind: agent-card
audience: [agent]
authority: derivative
canonical_page: ref.language.cps-ir
canonical_page_path: ../../language/cps-ir.md
status: current
stability: alpha
owner: language
last_verified: 2026-07-28
verified_against:
  git_commit: b7d6137f
  specs:
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
  tasks:
    - docs/plan/tasks/TASK-1590-cps-ir-core-data-structures.md
    - docs/plan/tasks/TASK-2037-engine-owned-cps-executor-and-runtime-crate-rename.md
    - docs/plan/tasks/TASK-1966-docs-reference-historical-quarantine.md
  code:
    - crates/ash-core/src/cps.rs
    - crates/ash-core/src/sexp.rs
  tests:
    - crates/ash-engine/src/private_cps/tests/
    - crates/ash-engine/tests/task_2037_engine_owned_cps_executor.rs
  examples: []
refresh_trigger:
  - reference/language/cps-ir.md changes
related:
  depends_on:
    - ref.language.cps-ir
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-098b-TARGET-IR.md
---

# CPS IR Agent Card

## Retrieval tags

cps, ir, intermediate-representation, continuation-passing-style, eval, interpreter, term, value, atom, lam, cont, jump, call, letval, letprim, letcont, letrec, if, raise, handle, effect, handler, resume, row, serialization, sexp, json

## Stale-claim warnings

- The CPS IR is an isolated prototype. Do not claim it is connected to the legacy AST or Lean differential testing.
- Do not claim bytecode compilation or JIT is implemented. Only the interpreter exists.
- Do not claim native multi-binding `LetRec` is supported. Phase 160 supports the documented tuple-of-lambdas mutual-recursion pattern inside single-binding `LetRec`.
- Do not claim full row polymorphism. Only duplicate validation exists.
- Effect aliases are not implemented.
- Full contract discharge is not implemented.

## Quick facts

- **Location**: `crates/ash-core/src/cps.rs` (types), `crates/ash-core/src/sexp.rs` (serialization)
- **Execution boundary**: external callers submit an admitted request to `Engine`; checked-CPS
  validation and evaluation are Engine-private.
- **Kernel location**: `crates/ash-engine/src/private_cps/`
- **Test files**: `crates/ash-engine/src/private_cps/tests/` and
  `crates/ash-engine/tests/task_2037_engine_owned_cps_executor.rs`
- **Spec**: `docs/spec/SPEC-098b-TARGET-IR.md`
- **Semantics**: `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`, `docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md`
- **Plan**: `docs/plan/PLAN-159-CPS-IR-INTERPRETER.md`, `docs/plan/PLAN-160-CPS-IR-RUNTIME-EXPANSION.md`

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

### Executing a CPS program

External code cannot call a CPS evaluator. It must use the public Engine admission and admitted-
request APIs, then receive a canonical terminal envelope. The private kernel is an implementation
detail, not an alternate client route. TASK-2040 retains deletion of direct-AST and differential
migration material; TASK-2041 owns end-state API-absence and client-parity evidence.

### Serializing to S-expression

```rust
use ash_core::sexp::{term_to_string, string_to_term};

let sexp = term_to_string(&term).unwrap();
let roundtripped = string_to_term(&sexp).unwrap();
assert_eq!(term, roundtripped);
```

## Related cards

- [CPS Interpreter](cps-interpreter.md) — how the interpreter works

## Edit preflight

Before modifying CPS IR types:
1. Check `docs/spec/SPEC-098b-TARGET-IR.md` for the canonical type definitions
2. Run `cargo test -p ash-core -p ash-engine --lib` to ensure tests pass
3. Update serialization tests if adding new variants
4. Update this card if adding new invariants or patterns
