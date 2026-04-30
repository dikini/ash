# PLAN-104: First-Class Workflow Carrier

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 108 promotes DESIGN-033 into first-class `Workflow<A>` as a contract-indexed process carrier and typed-do/comprehension target. Do not implement parser/typechecker/runtime work without the corresponding task file.

**Goal:** Implement [SPEC-056](../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md) by adding `Workflow<A>` as a first-class computation constructor with a Monad-shaped workflow dictionary, enabling `do:Workflow` blocks and `[...]: Workflow` comprehensions through existing SPEC-054/SPEC-055 infrastructure.

**Architecture:** Phase 108 is workflow-form/projection-first. Before adding Rust carriers or public library operations, it hardens the structure-preserving `WorkflowForm` grammar, projection-event/alignment model, staged `ContractPlan`, obligation vocabulary, and `requires`/`ensures` semantics. Implementation then adds the legacy-compatible contract syntax bridge, derives internal carriers from that preserved form, registers `Workflow` as a builtin unary computation constructor, adds `workflow::unit`/`bind`/`then`/`from_proc`/`from_act`/`requires`/`ensures`, and reuses the existing typed-do and comprehension elaboration path. The phase is sequential-workflow-only: workflow handles, parallel workflow operators, dynamic admission, and richer contract/admission/reporting combinators are deferred.

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
| [TASK-769](tasks/TASK-769-workflow-form-projection-semantics.md) | Define `WorkflowForm`, projections, alignment ids, staged contracts, and obligations | Docs/Semantics | 6 | 📝 Planned |
| [TASK-776](tasks/TASK-776-workflow-contract-syntax-and-legacy-translation.md) | Add workflow contract syntax, opaque intrinsic calls, and legacy declaration translation | Parser/Semantics | 6 | 📝 Planned |
| [TASK-770](tasks/TASK-770-workflow-type-and-stdlib-operations.md) | Register public `Workflow<A>` type, internal carriers, and `workflow` stdlib operations | Substrate | 9 | 📝 Planned |
| [TASK-771](tasks/TASK-771-workflow-do-target-dictionary.md) | Add `Workflow` typed-do dictionary and `do:Workflow` checking/elaboration | Semantic | 8 | 📝 Planned |
| [TASK-772](tasks/TASK-772-workflow-comprehension-target.md) | Enable `[...]: Workflow` comprehensions through SPEC-055 path | Semantic | 5 | 📝 Planned |
| [TASK-773](tasks/TASK-773-workflow-contract-summary-import-export.md) | Preserve workflow type/contract summaries across module exports/imports | Substrate | 7 | 📝 Planned |
| [TASK-774](tasks/TASK-774-workflow-diagnostics-and-negative-tests.md) | Add diagnostics for workflow target, explicit lifts, coverage, and opaque summaries | Semantic | 5 | 📝 Planned |
| [TASK-775](tasks/TASK-775-first-class-workflow-closeout.md) | Add examples, reconcile docs/status/changelog, and run final verification | Docs/Planning | 4 | 📝 Planned |

Estimated total: 54 hours.

## Tracks

### Track A: Workflow Form, Contract Syntax, and Semantic Substrate

- TASK-768 creates the normative packet and Phase 108 traceability.
- TASK-769 is a blocking semantic gate. It defines `WorkflowForm`, `WorkflowNodeId`, projection events, staged `ContractPlan`, obligation vocabulary, `requires`/`ensures` semantics, delayed lower-Proc coverage obligations, and equality/normalization strata before implementation adds Rust carriers.
- TASK-776 adds the required surface bridge: `requires:` / `ensures:` do statements, opaque intrinsic-call elaboration for `workflow::requires` / `workflow::ensures`, conservative legacy-compatible contract-expression name resolution, and deprecated legacy workflow declaration translation to the same `WorkflowForm` path.
- TASK-770 derives internal `WorkflowContract`, `AdmissionEnvelope`, `ContractPlan`, and `CoverageEvidence` carriers from the preserved workflow form while registering the public type/library surface.

### Track B: Public Carrier and Library Surface

- TASK-770 registers `Workflow<A>` as a builtin unary constructor and adds workflow library operations analogous to `proc`, including first-slice contract-injection operations `workflow::requires` and `workflow::ensures`.
- This track must preserve the public type as `Workflow<A>` and keep contract/evidence internals hidden.
- Carrier implementation must be derived from the TASK-769 workflow-form/projection model, not from an unrelated `Proc` plus metadata pair.

### Track C: Typed Do and Comprehension Integration

- TASK-771 adds the `Workflow` `DoDictionary` and validates `do:Workflow`.
- TASK-772 enables `[...]: Workflow` by reusing comprehension normalization into typed do.
- These tasks must not fork SPEC-054/SPEC-055 lowering logic.

### Track D: Modular Summaries, Diagnostics, and Closeout

- TASK-773 ensures imported workflows carry enough public type/contract summary for composition.
- TASK-774 hardens diagnostics and negative tests.
- TASK-775 adds examples, reconciles documentation, and performs final verification.

## Implementation Constraints

1. Public type is `Workflow<A>` only. Do not expose `Workflow<C, A>` or contract type parameters.
2. `Workflow` is a typed-do target, not a new parser-only semantic form.
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

## Verification Strategy

Every implementation task must include:

1. focused parser/typechecker/runtime tests for its changed layer;
2. regression checks proving `do:Act`, `do:Proc`, and Act/Proc comprehensions remain unchanged;
3. positive tests for `Workflow<A>` type registration and workflow operations;
4. positive tests for `requires:` / `ensures:` statement forms, `workflow::requires` / `workflow::ensures` intrinsic calls, `do:Workflow`, and `[...]: Workflow` equivalence;
5. negative tests for implicit Act/Proc lifts, opaque contract argument misuse, and missing workflow summaries;
6. coverage/evidence diagnostics for rejected contracts or opaque imported values;
7. legacy workflow declaration deprecation-warning tests and equivalent `WorkflowForm` translation tests;
8. `cargo fmt --check`;
9. focused `cargo test -p ash-typeck`, `cargo test -p ash-parser`, `cargo test -p ash-interp`, or affected crates as appropriate;
10. affected-crate `cargo clippy --all-targets --all-features -- -D warnings`;
11. independent subagent verification before marking a task complete.

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

Phase 108 builds on completed Act/Proc typed-do and comprehension infrastructure. It must not re-open Phase 105/106 syntax decisions.

Phase 108 also builds on Proc/runtime/capability/resource substrates. It must not redefine:

- `proc::par`, `proc::await`, `proc::join`, or `proc::gather` runtime semantics;
- operational bottom and `with_error` semantics;
- workflow boundary terminal outcomes;
- capability/resource authority provenance;
- module import/export rules beyond adding workflow contract summaries.

## Completion Criteria

Phase 108 is complete when:

- [ ] SPEC-056 is registered in docs/spec/README.md.
- [ ] PLAN-104 and TASK-768 through TASK-776 are registered in PLAN-INDEX.md.
- [ ] `Workflow<A>` is a public builtin unary type constructor.
- [ ] `workflow::unit`, `workflow::bind`, `workflow::then`, `workflow::from_proc`, `workflow::from_act`, `workflow::requires`, and `workflow::ensures` exist and type-check.
- [ ] `Workflow` resolves as a SPEC-054 do target.
- [ ] `do:Workflow` type-checks and elaborates through `workflow::bind`/`workflow::unit`.
- [ ] `[...]: Workflow` comprehensions elaborate through the same path.
- [ ] Implicit Act/Proc-to-Workflow lifts are rejected with explicit-lift hints.
- [ ] Workflow contract/coverage evidence exists and is used by diagnostics and lowering/runtime projection where applicable; non-dischargeable or opaque obligations are conservatively rejected rather than silently accepted.
- [ ] Imported workflow summaries are preserved or missing summaries are rejected.
- [ ] Deprecated legacy workflow declarations warn and translate to the same `WorkflowForm` path as equivalent first-class workflow expressions.
- [ ] Examples and docs state deferred parallel/dynamic-admission behavior honestly.
- [ ] Full verification and independent review pass.
