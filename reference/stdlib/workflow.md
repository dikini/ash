---
id: ref.stdlib.workflow
title: Workflow Standard Library
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
    - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md
    - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
  code:
    - std/src/workflow.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-cli/tests/example_corpus_check.rs
    - crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs
  examples:
    - examples/09-phase108/01-do-workflow-unit.ash
    - examples/09-phase108/02-do-workflow-contract-statements.ash
    - examples/09-phase108/03-workflow-algebra-intrinsics.reference.ash
    - examples/09-phase108/04-workflow-explicit-lifts.reference.ash
related:
  depends_on:
    - ref.stdlib.index
    - ref.stdlib.act
    - ref.stdlib.proc
    - ref.language.workflow
    - ref.language.generalized_do
  explains:
    - ref.status.known_limitations
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md
refresh_trigger:
  - std/src/workflow.ash changes
  - std/src/lib.ash changes
  - docs/spec/SPEC-051-WORKFLOW-SEMANTICS.md changes
  - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md changes
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
  - examples/09-phase108/README.md changes
---

# Workflow Standard Library

`Workflow<A>` is the governed computation carrier at the top of the current public tower. It is the value-level stdlib carrier associated with workflow admission and runtime boundary behavior, but this page only documents the public stdlib functions in `std/src/workflow.ash`.

For the language concept, read [Workflow boundaries](../language/workflows.md). RuntimeKernel details live under [runtime](../runtime/README.md).

## Public Functions

`std/src/workflow.ash` currently exposes:

| Name | Public shape | Notes |
| --- | --- | --- |
| `workflow::unit` | `A -> Workflow<A>` | Wraps a value in Workflow. |
| `workflow::bind` | `Workflow<A>, (A) -> Workflow<B> -> Workflow<B>` | Sequences Workflow computations. |
| `workflow::then` | `Workflow<A>, Workflow<B> -> Workflow<B>` | Runs first Workflow, then second. |
| `workflow::from_proc` | `Proc<A> -> Workflow<A>` | Explicit upward lift from Proc to Workflow. |
| `workflow::from_act` | `Act<A> -> Workflow<A>` | Explicit upward lift from Act to Workflow. |

Contract operations such as `workflow::requires` and `workflow::ensures` are not listed as public stdlib functions in `std/src/workflow.ash`; that file notes they remain compiler-prelude metadata because their parameter classes are not source-denotable Ash types yet.

## Examples

`examples/09-phase108/01-do-workflow-unit.ash` is classified as expected-pass:

```ash
pub fn approved_value() -> Workflow<Int> {
    do:Workflow {
        return 1
    }
}
```

`examples/09-phase108/04-workflow-explicit-lifts.reference.ash` is reference-only. It documents the intended explicit lift spelling, but it is not claimed as an end-to-end runnable source-file example in the current corpus:

```ash
pub fn workflow_from_proc() -> Workflow<Int> {
    do:Workflow {
        x <- workflow::from_proc(proc_step());
        return x
    }
}
```

## Tower Position

`Workflow` is the top public tower carrier:

```text
Pure < Act < Proc < Workflow
```

Current typechecker tests assert that raw `Proc<A>` and raw `Act<A>` binds in `do:Workflow` are rejected and that explicit `workflow::from_proc(...)` and `workflow::from_act(...)` remain accepted.

## Limitations

- The value-level stdlib page is not a full runtime-admission manual.
- Workflow contract statements and runtime reports are documented through language/runtime pages and tests, not as ordinary public stdlib functions here.
- Reference-only workflow algebra/lift examples are not promoted to runnable examples unless the example corpus classifies them that way.
- There are no implicit Act-to-Workflow or Proc-to-Workflow lifts.
