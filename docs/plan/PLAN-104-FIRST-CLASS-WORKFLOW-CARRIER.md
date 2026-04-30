# PLAN-104: First-Class Workflow Carrier

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 108 promotes DESIGN-033 into first-class `Workflow<A>` as a contract-indexed process carrier and typed-do/comprehension target. Do not implement parser/typechecker/runtime work without the corresponding task file.

**Goal:** Implement [SPEC-056](../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md) by adding `Workflow<A>` as a first-class computation constructor with a Monad-shaped workflow dictionary, enabling `do:Workflow` blocks and `[...]: Workflow` comprehensions through existing SPEC-054/SPEC-055 infrastructure while translating deprecated legacy workflow declarations to the same implementation path.

**Architecture:** Phase 108 is workflow-form/projection-first and compatibility-preserving. Before public Workflow operations can execute, the phase hardens `WorkflowForm`, source-ordered `WorkflowHeaderEvent`s, projection-event/alignment model, staged `ContractPlan`, non-denotable contract argument classes, obligation vocabulary, and the legacy-body adapter contract. Implementation then proceeds in an execution-friendly order: parser/classifier/header events; public `Workflow<A>` and intrinsic parameter classes; WorkflowForm-preserving typed-do; intrinsic call elaboration; executable lowering/runtime projection through the existing Proc/workflow boundary; deprecated legacy declaration translation; comprehension, module summaries, diagnostics, and closeout. Accepted legacy contract semantics are implemented in the new path rather than deferred.

**Tech Stack:** Rust 2024, `ash-core`, `ash-parser`, `ash-typeck`, `ash-engine`, `ash-interp`, `ash-stdlib`, existing `DoTarget`/`DoDictionary` infrastructure, Proc runtime substrate, workflow admission/report carriers, capability/resource provenance substrate.

---

## Phase 108: First-Class Workflow Carrier

**Status:** 📝 Planned
**Spec:** [SPEC-056](../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md)
**Design:** [DESIGN-033](../design/DESIGN-033-WORKFLOW-CONTRACT-OPERATOR-LIFTING.md)
**Depends on:** Phase 105 / [SPEC-054](../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), Phase 106 / [SPEC-055](../spec/SPEC-055-MONAD-COMPREHENSION-SYNTAX.md), Proc/runtime specs [SPEC-048](../spec/SPEC-048-PROC-LIBRARY.md), [SPEC-049](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](../spec/SPEC-051-WORKFLOW-SEMANTICS.md), and authority/resource specs [SPEC-052](../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)/[SPEC-053](../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md).

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-768](tasks/TASK-768-first-class-workflow-spec-plan-packet.md) | Promote DESIGN-033 into SPEC-056/PLAN-104 and register Phase 108 | Docs/Planning | 4 | ✅ Complete |
| [TASK-769](tasks/TASK-769-workflow-form-projection-semantics.md) | Define workflow form, projection, obligation, and adapter semantics | Docs/Semantics | 7 | 📝 Planned |
| [TASK-770](tasks/TASK-770-workflow-contract-surface-classifier-and-header-events.md) | Add workflow contract surface, classifier, and source-ordered header events | Parser/Substrate | 7 | 📝 Planned |
| [TASK-771](tasks/TASK-771-workflow-type-stdlib-and-intrinsic-parameters.md) | Register `Workflow<A>`, workflow operations, and non-denotable intrinsic parameters | Type/Substrate | 9 | 📝 Planned |
| [TASK-772](tasks/TASK-772-workflow-form-preserving-do-target.md) | Add WorkflowForm-preserving `do:Workflow` target | Semantic | 9 | 📝 Planned |
| [TASK-773](tasks/TASK-773-workflow-contract-intrinsic-call-elaboration.md) | Elaborate `workflow::requires` / `workflow::ensures` intrinsic calls | Semantic | 5 | 📝 Planned |
| [TASK-774](tasks/TASK-774-workflow-lowering-runtime-projection.md) | Add executable Workflow lowering and runtime projection tests | Runtime/Semantic | 6 | 📝 Planned |
| [TASK-775](tasks/TASK-775-legacy-workflow-translation-and-deprecation.md) | Translate deprecated legacy workflow declarations and emit warnings | Compatibility | 8 | 📝 Planned |
| [TASK-776](tasks/TASK-776-workflow-comprehension-target.md) | Enable `[...]: Workflow` comprehensions through SPEC-055 path | Semantic | 5 | 📝 Planned |
| [TASK-777](tasks/TASK-777-workflow-contract-summary-import-export.md) | Preserve workflow type/contract summaries across module exports/imports | Substrate | 7 | 📝 Planned |
| [TASK-778](tasks/TASK-778-workflow-diagnostics-and-negative-tests.md) | Add workflow diagnostics and negative tests | Semantic | 6 | 📝 Planned |
| [TASK-779](tasks/TASK-779-first-class-workflow-closeout.md) | Add examples, reconcile docs/status/changelog, and run final verification | Docs/Planning | 4 | 📝 Planned |

Estimated total: 77 hours.
Remaining after TASK-768: 73 hours.

## Tracks

### Track A: Workflow Form and Compatibility Substrate

- TASK-768 creates the normative packet and Phase 108 traceability.
- TASK-769 is a blocking semantic gate. It defines `WorkflowForm`, `WorkflowNodeId`, projection events, staged `ContractPlan`, `WorkflowHeaderEvent`, obligation vocabulary, non-denotable contract arguments, `any_role` OR semantics, WorkflowForm-preserving do artifacts, delayed lower-Proc coverage obligations, the legacy-body adapter contract, and equality/normalization strata before implementation adds Rust carriers.
- TASK-770 adds parser/surface substrate: `requires:` / `ensures:` do statements, source-ordered legacy header events, and the legacy-compatible classifier skeleton.

### Track B: Public Carrier, Workflow Do, and Runtime Projection

- TASK-771 registers `Workflow<A>` as a builtin unary constructor and adds workflow library operations analogous to `proc`, including first-slice contract-injection operations with non-denotable intrinsic parameters.
- TASK-772 adds the Workflow typed-do target and must preserve a `WorkflowForm` artifact rather than lowering directly to CoreExpr-only dictionary calls.
- TASK-773 adds direct intrinsic-call elaboration for `workflow::requires` / `workflow::ensures` after the type, operation, classifier, and form machinery exist.
- TASK-774 derives executable Proc/runtime projections from Workflow artifacts, proves `unit`/`bind`/`then` execute through existing Proc/workflow boundaries, and verifies contract-injection metadata is not dead.

### Track C: Legacy Compatibility, Comprehension, and Modules

- TASK-775 deprecates current workflow declarations with warnings and translates them through `WorkflowHeaderEvent` + `FromProc(legacy_body_as_proc_summary)` into the same WorkflowForm implementation path.
- TASK-776 enables `[...]: Workflow` by reusing comprehension normalization into typed do.
- TASK-777 ensures imported workflows carry enough public type/contract summary for composition.

### Track D: Diagnostics and Closeout

- TASK-778 hardens diagnostics and negative tests after all semantic paths exist.
- TASK-779 adds examples, reconciles documentation, and performs final verification.

## Implementation Constraints

1. Public type is `Workflow<A>` only. Do not expose `Workflow<C, A>` or contract type parameters.
2. `Workflow` is a typed-do target, not a parser-only semantic form.
3. `[...]: Workflow` must normalize through SPEC-055's existing comprehension-to-do path.
4. No implicit lifts: `Act<A>` and `Proc<A>` enter `Workflow<A>` only through `workflow::from_act` and `workflow::from_proc`.
5. Dynamic admission is forbidden in Phase 108.
6. Parallel workflow operators and `WorkflowHandle<A>` are deferred.
7. Contract normalization must preserve structure; only identity rewrites are allowed before coverage/evidence-preserving optimization.
8. `WorkflowForm` and projection-event alignment are the semantic source of truth; do not implement `Workflow<A>` as an unrelated `Proc<A>` plus dead metadata wrapper.
9. `workflow::from_proc` and `workflow::from_act` produce lower-contract coverage obligations. They must not require immediate empty-header coverage at the local expression site when an enclosing/composed workflow contract can cover the obligation.
10. Runtime scheduling, process handles, and workflow boundary terminal states must not be redefined.
11. Imported workflow/proc/act values used under `Workflow` must have sufficient summaries or be rejected.
12. Coverage/evidence is operationally significant: do not implement it as a dead placeholder with no diagnostics or runtime projection path.
13. Deprecated legacy workflow declarations must warn and translate into the same `WorkflowForm` path as first-class workflow expressions. New implementation work must not maintain a separate legacy runtime/typechecking semantic path.
14. Accepted legacy-compatible contract semantics, including role checks, current capability/resource header semantics, and `any_role` OR semantics, must be implemented in the new path rather than deferred.
15. `Requirement` and `OpenPostcondition` are non-denotable intrinsic parameter classes, not ordinary source-level Ash types.

## Verification Strategy

Every implementation task must include:

1. focused parser/typechecker/runtime tests for its changed layer;
2. regression checks proving `do:Act`, `do:Proc`, and Act/Proc comprehensions remain unchanged;
3. positive tests for `Workflow<A>` type registration and workflow operations;
4. positive tests for `requires:` / `ensures:` statement forms, `workflow::requires` / `workflow::ensures` intrinsic calls, `do:Workflow`, and `[...]: Workflow` equivalence;
5. negative tests for implicit Act/Proc lifts, opaque contract argument misuse, contract statement misuse outside Workflow, and missing workflow summaries;
6. runtime/lowering tests proving `unit`/`bind`/`then` Proc projections execute through existing boundaries and contract-injection metadata is preserved;
7. coverage/evidence diagnostics for rejected contracts or opaque imported values;
8. legacy workflow declaration deprecation-warning tests and equivalent `WorkflowForm` translation tests;
9. `cargo fmt --check`;
10. focused `cargo test -p ash-typeck`, `cargo test -p ash-parser`, `cargo test -p ash-interp`, or affected crates as appropriate;
11. affected-crate `cargo clippy --all-targets --all-features -- -D warnings`;
12. independent subagent verification before marking a task complete.

## Deferred Follow-on Phase Candidates

Later phases should own:

- full coverage solving beyond the conservative Phase 108 checker, including proof search, dynamic residualization, and richer admission planning;
- richer workflow contract/admission/reporting combinators beyond first-slice `requires` and `ensures` (`admit`, `report`, policy bundles, public reflection);
- `WorkflowHandle<A>` and handle-latent obligation lifecycle;
- workflow-level `par`, `spawn`, `scatter`, `cancel`, `await`, `join`, and `gather`;
- dynamic admission as an explicit audited capability;
- public contract reflection/tooling APIs;
- target inference for workflow comprehensions;
- formatter/LSP support for workflow examples;
- user-defined Monad dictionaries.

## Coordination / Non-Interference

Phase 108 builds on completed Act/Proc typed-do and comprehension infrastructure. It must not re-open Phase 105/106 syntax decisions except where a Workflow-specific target adds new behavior behind explicit `Workflow` resolution.

Phase 108 also builds on Proc/runtime/capability/resource substrates. It must not redefine:

- `proc::par`, `proc::await`, `proc::join`, or `proc::gather` runtime semantics;
- operational bottom and `with_error` semantics;
- workflow boundary terminal outcomes;
- capability/resource authority provenance;
- module import/export rules beyond adding workflow contract summaries.

## Completion Criteria

Phase 108 is complete when:

- [ ] SPEC-056 is registered in docs/spec/README.md.
- [ ] PLAN-104 and TASK-768 through TASK-779 are registered in PLAN-INDEX.md.
- [ ] `Workflow<A>` is a public builtin unary type constructor.
- [ ] `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures` exist and type-check.
- [ ] `Workflow` resolves as a SPEC-054 do target.
- [ ] `do:Workflow` type-checks and elaborates through a WorkflowForm-preserving artifact.
- [ ] `workflow::unit`, `workflow::bind`, and `workflow::then` have executable Proc/runtime projections through existing Proc/workflow boundaries, with contract metadata preserved.
- [ ] `[...]: Workflow` comprehensions elaborate through the same path.
- [ ] Implicit Act/Proc-to-Workflow lifts are rejected with explicit-lift hints.
- [ ] Workflow contract/coverage evidence exists and is used by diagnostics and lowering/runtime projection where applicable; non-dischargeable or opaque obligations are conservatively rejected rather than silently accepted.
- [ ] Imported workflow summaries are preserved or missing summaries are rejected.
- [ ] Deprecated legacy workflow declarations warn and translate to the same `WorkflowForm` path as equivalent first-class workflow expressions.
- [ ] Examples and docs state deferred parallel/dynamic-admission behavior honestly.
- [ ] Full verification and independent review pass.
