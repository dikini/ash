---
id: ref.stdlib.act
title: Act Standard Library
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: stdlib
last_verified: 2026-06-11
verified_against:
    git_commit: 61efd59f
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-047-ACT-MONAD.md
    - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
  code:
    - std/src/act.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-cli/tests/example_corpus_check.rs
    - crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs
  examples:
    - examples/07-phase105/01-do-act.ash
    - examples/07-phase105/02-act-sugar.ash
related:
  depends_on:
    - ref.stdlib.index
    - ref.language.act
    - ref.language.generalized_do
  explains:
    - ref.stdlib.proc
    - ref.status.known_limitations
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-047-ACT-MONAD.md
refresh_trigger:
  - std/src/act.ash changes
  - std/src/lib.ash changes
  - docs/spec/SPEC-047-ACT-MONAD.md changes
  - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md changes
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
  - crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs changes
---

# Act Standard Library

`Act<A>` is the public stdlib carrier for effectful computations. It is opaque. The runtime owns the hidden `ActEnv` state-threading substrate; source code cannot name or construct `ActEnv`.

For the language concept, read [Act effects](../language/effects-act.md). This page is the public stdlib/API surface in `std/src/act.ash`.

## Public Types and Functions

`std/src/act.ash` currently exposes:

| Name | Public shape | Notes |
| --- | --- | --- |
| `Act<A>` | `pub builtin type Act<A>` | Opaque effect carrier. |
| `Policy` | `pub type Policy = String` | Public alias. |
| `act::unit` | `A -> Act<A>` | Wraps a value in Act. |
| `act::bind` | `Act<A>, (A) -> Act<B> -> Act<B>` | Sequences Act computations. |
| `act::then` | `Act<A>, Act<B> -> Act<B>` | Runs the first Act, then the second. |
| `act::guard` | `String, Act<A> -> Act<A>` | Guards an Act computation with a policy string. |
| `act::policy_check` | `String -> Bool` | Builtin policy predicate. |

The file also declares builtin implementation hooks `__unit`, `__bind`, `__then`, `__fail`, and `__guard`. User code should treat the public wrappers above as the API.

`std/src/lib.ash` re-exports `unit`, `bind`, `then`, and `guard` from `act`; because those names are generic and collide with other modules, qualified `act::...` spelling is clearer in reference examples.

## Example

The current example corpus classifies `examples/07-phase105/01-do-act.ash` as passing. Minimal shape:

```ash
pub fn greeting_action(name: String) -> Act<String> {
    do:Act {
        let prefix = "hello, ";
        message <- act::unit(prefix + name);
        return message
    }
}
```

This is a runnable-current example through the example corpus baseline. The `act { ... }` spelling in `examples/07-phase105/02-act-sugar.ash` is also classified as expected-pass, but `do:Act` is the clearer spelling for generalized do documentation.

## Tower Position

`Act` sits above pure code and below `Proc`:

```text
Pure < Act < Proc < Workflow
```

There is no implicit lift from `Act<A>` into `Proc<A>` or `Workflow<A>`. Use [Proc `from_act`](proc.md) or [Workflow `from_act`](workflow.md) when crossing upward.

## Limitations

- `Act` is not `Result`. Use [Result](result.md) for domain success/failure values.
- `Act<Result<A, E>>` is the conventional shape for effectful work that returns a domain-level result; it still remains an Act computation.
- `ActEnv` is runtime-managed and hidden. The typechecker test `alpha_tower_opaque_carriers.rs` asserts it is not source-denotable.
- Arbitrary algebraic effect handlers are not part of this current public stdlib surface.
