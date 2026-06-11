---
id: ref.stdlib.proc
title: Proc Standard Library
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
    - docs/spec/SPEC-048-PROC-LIBRARY.md
    - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
  code:
    - std/src/proc.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-cli/tests/example_corpus_check.rs
    - crates/ash-engine/tests/task_718_proc_stdlib.rs
    - crates/ash-engine/tests/task_719_proc_from_act_stdlib.rs
    - crates/ash-typeck/tests/alpha_tower_opaque_carriers.rs
  examples:
    - examples/05-phase98/02-proc-par-await-join.ash
    - examples/05-phase98/03-proc-scatter-gather.ash
    - examples/07-phase105/03-do-proc-from-act.ash
related:
  depends_on:
    - ref.stdlib.index
    - ref.stdlib.act
    - ref.language.proc
    - ref.language.generalized_do
  explains:
    - ref.stdlib.workflow
    - ref.status.known_limitations
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-048-PROC-LIBRARY.md
refresh_trigger:
  - std/src/proc.ash changes
  - std/src/lib.ash changes
  - docs/spec/SPEC-048-PROC-LIBRARY.md changes
  - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md changes
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
  - crates/ash-engine/tests/task_718_proc_stdlib.rs changes
  - crates/ash-engine/tests/task_719_proc_from_act_stdlib.rs changes
---

# Proc Standard Library

`Proc<A>` is the public stdlib carrier for process-structured computations. It sits above `Act` and below `Workflow`. The public handle type `P<A>` is nameable/typeable as the process-handle surface returned by Proc operations, but it is not user-constructible.

For the language concept, read [Proc processes](../language/processes-proc.md). This page is the public stdlib/API surface in `std/src/proc.ash`.

## Public Types and Functions

`std/src/proc.ash` currently exposes:

| Name | Public shape | Notes |
| --- | --- | --- |
| `ParHandles<A, B>` | `(P<A>, P<B>)` | Tuple alias for handles returned by `par`. |
| `proc::unit` | `A -> Proc<A>` | Wraps a value in Proc. |
| `proc::from_act` | `Act<A> -> Proc<A>` | Explicit upward lift from Act to Proc. |
| `proc::bind` | `Proc<A>, (A) -> Proc<B> -> Proc<B>` | Sequences Proc computations. |
| `proc::then` | `Proc<A>, Proc<B> -> Proc<B>` | Runs first Proc, then second. |
| `proc::await` | `P<A> -> Proc<A>` | Observes one process handle. |
| `proc::yield` | `() -> Proc<Unit>` | Cooperative scheduler yield. |
| `proc::par` | `Proc<A>, Proc<B> -> Proc<ParHandles<A, B>>` | All-or-none child admission for two children. |
| `proc::scatter` | `List<A>, (A) -> Proc<B> -> Proc<List<P<B>>>` | All-or-none child admission over a list. |
| `proc::join` | `P<A>, P<B> -> Proc<(A, B)>` | Waits for two handles. |
| `proc::gather` | `List<P<A>> -> Proc<List<A>>` | Waits for a list of handles. |

`std/src/lib.ash` exposes `pub mod proc;`, so qualified `proc::...` names are the intended reference spelling.

## Examples

`examples/07-phase105/03-do-proc-from-act.ash` is classified as expected-pass and demonstrates the required explicit Act-to-Proc boundary:

```ash
pub fn proc_greeting(name: String) -> Proc<String> {
    do:Proc {
        message <- proc::from_act(do:Act {
            value <- act::unit("hello, " + name);
            return value
        });
        return message
    }
}
```

`examples/05-phase98/02-proc-par-await-join.ash` and `examples/05-phase98/03-proc-scatter-gather.ash` are also classified as expected-pass by the example corpus. Their source-level composition is current, while some runtime observations are exercised by engine tests that force returned Proc closures.

## Tower Position

`Proc` is the process layer:

```text
Pure < Act < Proc < Workflow
```

`do:Proc` does not accept a raw `Act<A>` bind. The typechecker tests assert that direct Act binds in `do:Proc` fail and that `proc::from_act(...)` remains accepted.

## Limitations

- `P<A>` is a public handle surface, not a user data constructor.
- Runtime process identity types such as `ProcessId` and `ProcessHandle` are not source-denotable public types.
- `proc::from_act` returns a Proc computation value. It does not eagerly expose a process handle at a workflow boundary.
- Proc does not silently become Workflow. Use [Workflow `from_proc`](workflow.md).
