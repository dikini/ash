# SPEC-069: Alpha Visible Tower Algebra and Generalized Do Lowering

**Status:** Draft
**Date:** 2026-05-19
**Promotes:** [DESIGN-040](../design/DESIGN-040-ALPHA-ALGEBRAIC-TOWER.md)
**Builds on:** [SPEC-047](SPEC-047-ACT-MONAD.md), [SPEC-048](SPEC-048-PROC-LIBRARY.md), [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](SPEC-051-WORKFLOW-SEMANTICS.md), [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-056](SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md), [SPEC-066](SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md), [SPEC-067](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
**Related:** [SPEC-001](SPEC-001-IR.md), [SPEC-003](SPEC-003-TYPE-SYSTEM.md), [SPEC-004](SPEC-004-SEMANTICS.md), [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md), [SPEC-070](SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
**Plan:** [PLAN-118](../plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md)
**Implementation Tasks:** [TASK-919](../plan/tasks/TASK-919-design040041-current-state-and-scope-reconciliation.md) through [TASK-932](../plan/tasks/TASK-932-alpha-closeout-review-remediation.md)

## 1. Summary

SPEC-069 defines the alpha target for Ash's visible computation tower. Alpha must demonstrate that `Act<A>`, `Proc<A>`, `Workflow<A>`, and ordinary user/library computation constructors are sequenced through Ash-visible algebra and type evidence rather than unrelated parser/runtime magic.

The alpha rule is:

```text
Visible algebra selects construction.
Opaque runtime mechanics implement that construction.
Typed lowering preserves the algebraic evidence.
Execution artifacts are traceable back through typed IR to source.
```

Alpha requires:

1. `Act`, `Proc`, `Workflow`, and accepted user/library constructors such as `Result<_, E>` have public, nameable, typeable construction APIs; `P<A>` has a public nameable/typeable handle surface returned by Proc operations, not a user construction API.
2. `Monad<K>` evidence is the canonical sequencing boundary for `do:K` and comprehension lowering.
3. `do:K` supports full `<-` lowering through selected evidence, not only return-only target resolution.
4. Evidence-selected `return` and `bind` calls lower through typed computation expressions that can be specialized before execution.
5. `Act`, `Proc`, and `Workflow` remain opaque operational carriers; hidden environments, process identities, admission state, and reports are not ordinary user data.
6. TCIR/AMIR/bytecode work retains traceability from source `do`/algebra calls to execution artifacts.
7. OODA remains expressible as libraries/templates/lints over the tower, not as a required primitive IR root for alpha.

## 2. Baseline

The live repository already contains these prerequisites:

- [SPEC-054](SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md): explicit `do:K` syntax, statement forms, Act/Proc migration, and typed do elaboration.
- [SPEC-056](SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md): first-class `Workflow<A>` carrier and workflow algebra-call preservation.
- [SPEC-066](SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md): explicit source `_` holes in do-target type arguments such as `Result<_, E>`.
- [SPEC-067](SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md): constructor-kinded binders and explicit `Monad<K>` evidence lookup at the target-resolution/return-only boundary.

Remaining alpha gaps are full evidence-selected `bind` lowering, reusable typed computation-expression carriers, monomorphization/specialization of generic monadic code, execution lowering through traceable TCIR/AMIR/bytecode, authority handoff updates in older specs, and OODA compatibility as library/template/lint surface.

## 3. Scope

In scope:

1. public alpha algebra contract for `Monad<K>` sequencing;
2. generalized `do:K` and comprehension lowering through selected evidence;
3. relationship between `Act`, `Proc`, `Workflow`, `Result<_, E>`, and user/library monads;
4. typed computation-expression artifacts suitable for TCIR/AMIR lowering;
5. static evidence selection and monomorphization/specialization for accepted generic monadic code;
6. no-magic requirements for tower-specific operations;
7. OODA demotion compatibility requirements;
8. acceptance and non-interference evidence for the alpha visible tower.

Out of scope: JIT/native-code generation; full self-hosting; arbitrary algebraic effects/effect rows/resumable continuations/user-defined handlers; law proving; fully free do-target inference; unrestricted type lambdas or higher-rank polymorphism; implicit domain failure from operational `fail`; remote/distributed daemon semantics.

## 4. Public computation tower

```text
Pure < Act < Proc < Workflow
```

| Layer | Public role | Runtime-owned opaque details | Required visible operations |
| --- | --- | --- | --- |
| `Pure` | deterministic value computation | lexical environment and compile-time type evidence | `fn`, `let`, `match`, constructors, pure calls |
| `Act<A>` | sequential effectful computation | hidden `ActEnv`, capability/provider dispatch, effect trace | `act::unit`, `act::bind`, capability invocation APIs, operational failure hooks |
| `Proc<A>` | process-capable computation | process identity, scheduler, split/join environment, cancellation, mailbox/handle state | `proc::unit`, `proc::bind`, `proc::from_act`, `proc::par`, `proc::await`, `proc::join`/`gather` |
| `Workflow<A>` | governed process computation | admission state, roles/capabilities, obligations, reports, workflow boundary reinterpretation | `workflow::unit`, `workflow::bind`, `workflow::from_proc`, `workflow::requires`, `workflow::ensures`, report/failure APIs |

`P<A>` is the public opaque process-handle surface associated with the `Proc` layer. It is nameable and typeable as an observation handle returned by operations such as `proc::par`, but its process identity, observation state, and scheduler ownership remain runtime-managed and non-denotable as ordinary user data.

The public operations may be backed by compiler/runtime intrinsics. They must still be nameable and typeable Ash-visible APIs. Runtime behavior may specialize an operation such as `proc::par`, but it must not introduce authority, scheduling, capability access, or reporting behavior that no public operation requested.

## 5. `Monad<K>` evidence contract

Canonical shape:

```ash
interface Monad<M : * -> *> {
    return<A>(a: A) -> M<A>;
    bind<A, B>(ma: M<A>, f: A -> M<B>) -> M<B>;
}
```

Semantic requirements:

1. `M` has effective kind `* -> *`;
2. `return` and `bind` are selected from evidence for the exact do target constructor;
3. evidence keys preserve partial-constructor identities such as `Result<_, E>`;
4. overlapping or ambiguous evidence is rejected before typed lowering;
5. selected operation bodies or intrinsic shims are represented explicitly enough for typed lowering and specialization.

A block:

```ash
do:K {
    x <- mx;
    let y = f(x);
    z <- mz(y);
    return g(x, z)
}
```

lowers semantically to nested evidence-selected calls:

```ash
Monad<K>.bind(mx, fn(x) {
    let y = f(x);
    Monad<K>.bind(mz(y), fn(z) {
        Monad<K>.return(g(x, z))
    })
})
```

The parser preserves surface `DoBlock` syntax and must not perform target-specific lowering.

## 6. Do-target rules

1. Explicit `do:K` remains accepted and preferred.
2. Expected-type-directed `do { ... }` may be accepted only when the expected type fixes a unique `K<A>` and `Monad<K>` evidence. Fully free inference is deferred.
3. Partial-constructor targets require exactly one value hole in the SPEC-066 style, e.g. `Result<_, ParseError>`.
4. No implicit lifts are inserted across `Act`, `Proc`, and `Workflow`. Use explicit operations such as `proc::from_act` and `workflow::from_proc`.
5. `fail` inside `do:K` remains tower-scoped operational bottom, not `None`, `Err`, empty list, or any other domain failure value.
6. Ignored monadic actions use explicit `_ <- action;` until a later spec adds bare expression statements.
7. Binders remain simple identifiers unless a later pattern-binding spec extends both `let` and `<-` semantics.

## 7. Typed computation-expression artifact

SPEC-069 requires a reusable typed carrier for evidence-selected computation expressions before AMIR/bytecode lowering. The exact Rust type is chosen by TASK-925 using TASK-920's audit/callsite bindings, but it must represent:

- source span/provenance for each `do` statement and lowered call;
- target constructor identity and evidence identity;
- selected `return`/`bind` operation identities;
- target tower level;
- ordinary pure subexpressions and closures/lambdas;
- explicit tower lifts and tower-specific operations;
- operational-failure boundaries;
- mapping back to the surface `DoBlock`, comprehension, or explicit algebra call that created it.

The carrier must not collapse user constructors into Act/Proc/Workflow runtime terms merely to make execution easier.

## 8. TCIR, AMIR, and bytecode pressure

Alpha should create a minimal traceable execution spine:

```text
Surface AST -> typed computation expression / TCIR -> AMIR -> bytecode logical schema -> VM execution subset
```

Required properties:

1. pure functions and builtin calls have an executable path;
2. `Act` capability boundaries execute through runtime helpers/providers;
3. `Proc` execution can represent process handles, start/admission failure, and awaited observation failure;
4. minimal `Workflow` execution can represent admission, obligations, report construction, and workflow failure reinterpretation;
5. every AMIR/bytecode instruction range has a debug/provenance path back to TCIR and source;
6. bytecode verification does not require reparsing source.

JIT compatibility should influence block/register bytecode shape and stable logical sections, but JIT implementation is not an alpha requirement.

## 9. OODA compatibility and demotion

OODA concepts remain valuable but are not primitive alpha IR roots by default. Older OODA-shaped workflow forms migrate toward library/template calls, workflow contract/reporting patterns, and lints/teaching templates. SPEC-069 does not require deleting historical OODA references in one pass; it requires that new alpha execution IR and bytecode are not designed around OODA as a privileged primitive.

TASK-930 adds the alpha compatibility surface as ordinary `std::ooda`
library/template helpers plus lint/spec guidance. These helpers are source-level
markers only; TCIR/AMIR/bytecode artifacts continue to lower through selected
Monad evidence, visible tower operations, and traceable execution artifacts
rather than OODA-specific opcodes or runtime roots.

## 10. Relationship to runtime regime

[SPEC-070](SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md) owns OS-facing runtime hosting. SPEC-069 owns the language-semantic and execution-artifact side consumed by that runtime kernel. `ash run` and `ashd` must execute the same accepted semantics for a given compiled artifact.

## 11. Acceptance matrix

| ID | Case | Expected result |
| --- | --- | --- |
| A69-1 | `do:Act` with `<-` and final `return` | lowers through selected `Monad<Act>` evidence |
| A69-2 | `do:Proc` binding an `Act<A>` directly | rejected; `proc::from_act` is required |
| A69-3 | `do:Workflow` with workflow algebra operations | preserves workflow construction artifacts and obligations |
| A69-4 | `do:Result<_, E>` | resolves partial-constructor target and lowers through `Monad<Result<_, E>>` evidence |
| A69-5 | user-defined `Monad<Option>` with bind | full `<-` lowering uses selected evidence, not a hidden unrelated dictionary |
| A69-6 | generic `M : * -> * where M: Monad` | accepted instantiations are statically resolved/specialized before execution |
| A69-7 | ambiguous/overlapping `Monad<K>` evidence | rejected before typed computation-expression lowering |
| A69-8 | `fail` inside `do:Result<_, E>` | remains operational bottom; does not implicitly construct `Err` |
| A69-9 | explicit OODA library call | remains ordinary library/template call in TCIR/AMIR |
| A69-10 | bytecode verifier/debug artifact | validates logical sections and explains provenance without source reparsing |
| A69-11 | old Act block compatibility | rewrites through same path or is rejected by documented gate |
| A69-12 | `ash run` vs `ashd` host mode | host mode does not change typed lowering or tower semantics |

## 12. Non-interference requirements

- Do not regress SPEC-066/SPEC-067 target-resolution behavior for holes and HKT evidence.
- Generalized user-monad lowering must not broaden associated-family inversion or proof search beyond SPEC-060 through SPEC-064.
- Act/Proc/Workflow public algebra must not expose hidden runtime environment representations as ordinary user data.
- Bytecode/AMIR work must not bypass capability/admission semantics by directly calling providers without visible Act/Workflow authority.
- OODA demotion must not delete examples or historical docs without a separate compatibility/migration task.

## 13. Implementation tasks

See [PLAN-118](../plan/PLAN-118-DESIGN-040-041-ALPHA-IMPLEMENTATION-PACKET.md).

## 14. Changelog

### 2026-05-19

- Initial draft promoted from DESIGN-040 and paired with SPEC-070/PLAN-118.
