---
id: ref.stdlib.index
title: Standard Library Tower Index
kind: index
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
slice: reference-slice-3
owner: stdlib
last_verified: 2026-06-11
verified_against:
  git_commit: c1f53d76
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-047-ACT-MONAD.md
    - docs/spec/SPEC-048-PROC-LIBRARY.md
    - docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md
    - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md
    - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
  code:
    - std/src/act.ash
    - std/src/proc.ash
    - std/src/workflow.ash
    - std/src/result.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-cli/tests/example_corpus_check.rs
    - crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs
    - crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs
  examples:
    - examples/07-phase105/01-do-act.ash
    - examples/07-phase105/03-do-proc-from-act.ash
    - examples/09-phase108/01-do-workflow-unit.ash
    - examples/09-phase108/04-workflow-explicit-lifts.reference.ash
    - tests/std/result.ash
related:
  depends_on:
    - ref.index
    - ref.language.act
    - ref.language.proc
    - ref.language.workflow
    - ref.language.generalized_do
  explains:
    - ref.stdlib.act
    - ref.stdlib.proc
    - ref.stdlib.workflow
    - ref.stdlib.result
    - ref.status.feature_matrix
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - std/src/act.ash changes
  - std/src/proc.ash changes
  - std/src/workflow.ash changes
  - std/src/result.ash changes
  - std/src/lib.ash changes
  - docs/spec/SPEC-047-ACT-MONAD.md changes
  - docs/spec/SPEC-048-PROC-LIBRARY.md changes
  - docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md changes
  - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md changes
  - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md changes
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
  - crates/ash-cli/tests/stdlib_corpus_check.rs changes
  - crates/ash-cli/tests/example_corpus_check.rs changes
---

# Standard Library Tower Index

This section documents the current public stdlib/API surfaces for the alpha tower. It is separate from the language concept pages:

- [Act effects](../language/effects-act.md), [Proc processes](../language/processes-proc.md), [Workflow boundaries](../language/workflows.md), and [generalized do](../language/generalized-do.md) explain language concepts.
- These stdlib pages list the public library names, current examples, evidence, and limits for the API surfaces in `std/src`.

## Public Tower Map

The current public reading order is:

```text
Pure < Act < Proc < Workflow
```

Pure expressions compute ordinary values. `Act<A>` is the first effectful carrier. `Proc<A>` is the process-structured carrier above Act. `Workflow<A>` is the governed workflow carrier above Proc. Current alpha evidence preserves explicit crossing between layers; it does not insert implicit lifts.

Use the explicit public operations when crossing the tower:

| Crossing | Current public operation |
| --- | --- |
| Pure value to Act | `act::unit(value)` |
| Pure value to Proc | `proc::unit(value)` |
| Act to Proc | `proc::from_act(action)` |
| Pure value to Workflow | `workflow::unit(value)` or `do:Workflow { return value }` |
| Proc to Workflow | `workflow::from_proc(proc_value)` |
| Act to Workflow | `workflow::from_act(action)` |

`Result<T, E>` is an ordinary domain value type. It is not a tower carrier in `Pure < Act < Proc < Workflow`, even though current generalized do evidence can lower selected `do:Result<_, E>` shapes in typechecker evidence tests. Domain failure values and operational bottom remain distinct.

## Pages

- [Act stdlib](act.md): `Act<A>`, `act::unit`, sequencing helpers, guards, policy checks, and hidden `ActEnv` limits.
- [Proc stdlib](proc.md): `Proc<A>`, public process handles `P<A>`, explicit Act lifts, child admission, await/join/gather operations, and scheduling limits.
- [Workflow stdlib](workflow.md): `Workflow<A>`, value-level algebra, explicit lower-tower lifts, and current contract-operation limits.
- [Result stdlib](result.md): `Result<T, E>`, `Ok`/`Err`, helper functions, and the domain-failure versus operational-bottom boundary.
- [Standard algebra](algebra.md): current `std::algebra` Semigroup/Monoid/Functor/Applicative/Monad surfaces, Phase 134 Comonad/Kleisli additions, and deferred Cokleisli/Coapplicative/category boundaries.

## Evidence Baseline

This page was checked against HEAD `710340f`. The direct stdlib files `std/src/act.ash`, `std/src/proc.ash`, `std/src/workflow.ash`, and `std/src/result.ash` are classified as expected-pass by `crates/ash-cli/tests/stdlib_corpus_check.rs`.

Examples in this section are marked as runnable only when the current example corpus or cited tests support that classification. Otherwise they are reference-only or illustrative.
