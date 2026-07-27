---
id: docs.plan.semantic-rule-coverage
title: Semantic Rule Coverage Map
kind: implementation-coverage-map
status: active
authority: planning-and-review
last_verified: 2026-07-27
---

# Semantic Rule Coverage Map

This is the human-review surface for target semantic-rule coverage. Canonical specs own
semantics; `docs/spec/SEMANTIC-TRACEABILITY.json` owns machine-validated links. This map makes a
rule's declared domain and gaps explicit before implementation.

For semantic work, link a task to one or more rows below and update the row before writing a
fixture. A source example is evidence only. A general rule is complete when all applicable declared
layers have the required coverage; `n/a` and `non-authorizing` remain valid terminal ownership
states. Status values are **general**, **bounded**, **planned**, **deferred**, or **not
applicable**.

## How to read this map

This is a composition map of implementation-domain ownership, not a whole-language progress
scorecard. A **bounded** label says that a rule family owns a deliberate finite feature/domain and
the listed layers. It does not mean that the feature is incomplete because another task owns a
downstream layer. A **general** label is complete only for its named owner's declared domain, not
for the entire language. **n/a** means a layer is intentionally outside that family's ownership;
**non-authorizing** means that the layer transports requirements or metadata without installing
runtime/admission authority.

Read every omitted or non-authorizing layer as a named handoff, not a demand for cross-layer work.
For example, TASK-2013 produces checked typed-handler facts; TASK-2014 consumes those facts to
construct and authorize an admission artifact and frame instructions; TASK-2008 projects the
resulting terminal envelope. TASK-2013 therefore does not need to implement runtime merely because
its facts participate in an executable path. End-to-end integration tests and refinement proofs
validate the composed handoffs separately from each task's feature/layer ownership.

## Executable-realization composition

[PLAN-203](PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md) consumes these handoffs to realize one
Surface → Core → CPS → Engine execution path. It is an integration owner, not an additional layer
that changes any family above. New or materially revised PLAN-203 tasks declare whether their
run-route impact is `none`, `prerequisite`, or `active`; `active` routes require a named
CLI/daemon integration case over the same admitted request and normalized terminal result.

The traceability graph carries optional Verus assurance work. A deferred proof obligation is a
visible future item, not a missing runtime layer or a release blocker.

## Rule families
### Surface forms and source-to-Core

- **Canonical owner:** `SPEC-095b`, `SPEC-098c`
- **Layer status:** Type bounded; Core bounded; CPS —; admission/runtime —; evidence bounded.
- **Declared domain and next obligation:** Pure entries, declared operation facts, and selected
  handler facts; general expression, call, closure, pattern, and import lowering is planned.

### Calls and continuations

- **Canonical owner:** `SEM-CPS-CALL-001`, `SEM-CPS-JUMP-001`
- **Layer status:** Type bounded; Core bounded; CPS bounded; admission/runtime bounded; evidence
  bounded.
- **Declared domain and next obligation:** Exact local call and pure forms only; general calls,
  parameters, closures, recursion, and imports are planned.

### Core control and terminals

- **Canonical owner:** `SEM-CPS-LETVAL-001`, `SEM-CPS-IF-001`, `SEM-CPS-RETURN-001`,
  `SEM-CPS-TRAP-001`
- **Layer status:** Type bounded; Core bounded; CPS bounded; admission/runtime bounded; evidence
  bounded.
- **Declared domain and next obligation:** Approved pure ANF and selected control forms only;
  general source control lowering is planned.

**TASK-2031B evidence handoff:** bounded verification-only reconciliation of two lexical-scope
CLI negative assertions to the existing checked Core-to-CPS bridge-domain rejection. It consumes
TASK-2003/TASK-2004/TASK-2014 admission facts and changes no Type, Core, CPS, admission/runtime,
or terminal layer.

### Operations and lookup

- **Canonical owner:** `SEM-EFFECT-LOOKUP-001`, `SEM-EFFECT-RAISE-001`
- **Layer status:** Type bounded; Core bounded; CPS bounded; admission/runtime bounded; evidence
  bounded.
- **Declared domain and next obligation:** Concrete declared/built-in operations only; arbitrary
  operations, arguments, imports, and chains are planned.

### Handlers and deep affine resume

- **Canonical owner:** `SEM-EFFECT-HANDLE-001`, `SEM-EFFECT-DEEP-AFFINE-HANDLE-001`
- **Layer status:** Type bounded; Core bounded; CPS bounded; admission/runtime bounded; evidence
  bounded.
- **Declared domain and next obligation:** Exact closed-row witnesses only; general multi-clause,
  open-row, imported, and multi-shot behavior is planned.

### Rows and imported summaries

- **Canonical owner:** `SPEC-097b`, `TYPE-TARGET-ROW-001`
- **Layer status:** Type bounded; Core bounded metadata; CPS n/a; admission/runtime
  non-authorizing; evidence bounded.
- **Declared domain and next obligation:** V8 structural summaries and selected closed rows;
  general row polymorphism, expansion, and discharge is planned.

### Production admission and frames

- **Canonical owner:** `TASK-2004`, `TASK-2014`
- **Layer status:** Type bounded; Core bounded; CPS bounded; admission/runtime bounded; evidence
  bounded.
- **Declared domain and next obligation:** Path-B selected artifacts only; general artifacts and all
  route coverage are planned.

### Terminal envelopes and async control

- **Canonical owner:** `TASK-2008`, `TASK-2014`
- **Layer status:** Type bounded; Core n/a; CPS bounded; admission/runtime bounded; evidence
  bounded.
- **Declared domain and next obligation:** Selected return/rejection/trap/timeout/cancellation
  routes; full route matrix is planned.

**TASK-2031C prerequisite handoff:** bounded Linux test-host verification that a programmatic
SIGINT reaches an isolated Tokio listener before TASK-2008's exact admitted `time::sleep` route is
evaluated. Type/Core/CPS/admission, the existing CLI forwarding, and Engine control precedence are
consumed existing layers; verification is bounded. TASK-2008 consumes the terminal outcome, while
TASK-2032 owns client-parity integration.

**TASK-2031F evidence handoff:** bounded correction of three existing stdlib callable negative
assertions to TASK-2003's current PureAnf bridge-domain rejection. Type/Core/CPS/admission/runtime
layers are consumed existing behavior; run-route impact is none and TASK-2032 retains parity.

### Differential parity

- **Canonical owner:** `TASK-2005`, `TASK-439`
- **Layer status:** Type n/a; Core bounded private targets; CPS bounded private targets;
  admission/runtime non-production; evidence bounded.
- **Declared domain and next obligation:** Trusted case- and fingerprint-locked corpus controls
  only; general parity is planned.

### Contracts, predicates, and proofs

- **Canonical owner:** `SPEC-098b`, `SPEC-100`
- **Layer status:** Type bounded; Core bounded sidecars; CPS —; admission/runtime —; evidence
  bounded.
- **Declared domain and next obligation:** Predicate provenance only; general discharge, proof, and
  runtime contract semantics are planned.

## Required task record

Each linked task records: canonical rule/spec section; declared domain; layer status changed;
positive, negative, mutation, and parity evidence where applicable; non-goals; and the next gap.
Each new or materially revised linked semantic task/record must also contain a **Handoffs** block
with its **Consumes**, **Produces**, intentionally unowned layer and its **downstream owner**, and
**integration/proof responsibility**. Reviewers reject a claim that a passing fixture implements a
general rule without this row update.

## TASK-2001 semantic workflow record

**Task:** [TASK-2001](tasks/TASK-2001-target-grammar-gap-and-spec-conflict-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `GRAM-TARGET-MODULE-001`
**Domain:** bounded
**Layers:** type bounded; core bounded; cps not-applicable; admission-runtime not-applicable; verification bounded.
**Evidence:**
- **Positive:** `TEST-ENGINE-V8-IMPORTED-HANDLER-ROW-E2E`
- **Negative:** `TEST-PARSER-STALE-DECLARATION-REJECTION`
- **Mutation:** `TEST-CORE-V8-STRUCTURAL-EFFECT-ROW-UNKNOWN-FIELD-REJECTION`
- **Parity:** not applicable; this parser/type-summary slice has no paired execution relation.
**Non-goals:** General grammar, row, and handler realization.
**Next obligation:** Realize the remaining selected alias, group, handler, newtype, and row forms across their declared layers.

## TASK-2002 semantic workflow record

**Task:** [TASK-2002](tasks/TASK-2002-generic-do-and-lowering-sidecar-strategy.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `LOWER-SURFACE-CORE-001`
**Domain:** bounded
**Layers:** type bounded; core bounded; cps not-applicable; admission-runtime not-applicable; verification bounded.
**Evidence:**
- **Positive:** `TEST-AMBIENT-DO-SOURCE-ENTRY-BOUNDARY`
- **Negative:** `TEST-ENGINE-NAMED-DO-TARGET-REJECTION`
- **Mutation:** `TEST-ENGINE-INVALID-HELPER-CONTRACT-SIDECAR-GUARD`
- **Parity:** not applicable; the retained sidecars are metadata, not a paired execution relation.
**Non-goals:** General sidecar completeness and runtime contract semantics.
**Next obligation:** Carry every required target sidecar or an explicit unsupported outcome through lowering.

## TASK-2003 semantic workflow record

**Task:** [TASK-2003](tasks/TASK-2003-return-authority-and-cps-kernel-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Domain:** bounded
**Layers:** type bounded; core bounded; cps bounded; admission-runtime bounded; verification bounded.
**Evidence:**
- **Positive:** `TEST-ENGINE-SEALED-LOCAL-CALL-CORE-CPS-PRODUCTION`
- **Negative:** `TEST-CORE-CPS-RETURN-AUTHORITY`
- **Mutation:** `TEST-ENGINE-SEALED-LOCAL-CALL-PROVENANCE-GUARD`
- **Parity:** not applicable; this lowering route does not claim a general reference-runtime parity relation.
**Non-goals:** General source control, call, and continuation lowering.
**Next obligation:** Extend checked Core/CPS realization only through separately admitted source forms.

## TASK-2004 semantic workflow record

**Task:** [TASK-2004](tasks/TASK-2004-core-cps-production-boundary-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Domain:** bounded
**Layers:** type bounded; core bounded; cps bounded; admission-runtime bounded; verification bounded.
**Evidence:**
- **Positive:** `TEST-ENGINE-RUN-HANDLER-FREE-CHECKED-CPS-ADMISSION`
- **Negative:** `TEST-ENGINE-UNARY-NEGATION-PRODUCTION-REJECTION`, `TEST-ENGINE-RUN-FILE-UNARY-NEGATION-PRODUCTION-REJECTION`
- **Mutation:** `TEST-CORE-CPS-ADMISSION-GUARD`
- **Parity:** not applicable; the production boundary does not itself claim a parity relation.
**Non-goals:** A legacy direct-evaluator fallback or general source admission.
**Next obligation:** Admit further source forms only after validated typed lowering and checked Core/CPS evidence.

## TASK-2005 semantic workflow record

**Task:** [TASK-2005](tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `OBS-TARGET-PROJECTION-001`
**Domain:** bounded
**Layers:** type bounded; core bounded; cps bounded; admission-runtime not-applicable; verification bounded.
**Evidence:**
- **Positive:** `TEST-DIRECT-RUNTIME-V3-INT-ADD-PARITY`
- **Negative:** `TEST-DIFFERENTIAL-TRUSTED-DIRECT-ORACLE-GATE`
- **Mutation:** `TEST-DIRECT-RUNTIME-ABSORB-SLEEP-HANDLER-FINGERPRINT-GUARD`
- **Parity:** `TEST-DIRECT-RUNTIME-V3-INT-ADD-PARITY`
**Non-goals:** General direct-runtime to production checked-CPS parity.
**Next obligation:** Expand only case-owned paired observables with explicit divergence dispositions.

## TASK-2008 semantic workflow record

**Task:** [TASK-2008](tasks/TASK-2008-json-variant-observable-projection.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `OBS-TARGET-PROJECTION-001`
**Domain:** bounded
**Layers:** type bounded; core bounded; cps bounded; admission-runtime bounded; verification bounded.
**Evidence:**
- **Positive:** `TEST-CLI-CANONICAL-TERMINAL-PROJECTION`
- **Negative:** `TEST-CLI-UNADMITTED-TRAP-SLEEP-TERMINAL-ENVELOPE`
- **Mutation:** `TEST-CLI-POSTEXECUTION-INVALID-EXIT-PROJECTION`
- **Parity:** not applicable; terminal projection is not a direct-runtime parity claim.
**Non-goals:** A complete terminal matrix for every future execution route.
**Next obligation:** Add canonical envelope cases only with an admitted checked route and focused observable evidence.

## TASK-2013 semantic workflow record

**Task:** [TASK-2013](tasks/TASK-2013-source-handler-and-handle-lowering.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-EFFECT-HANDLE-001`
**Domain:** bounded
**Layers:** type bounded; core bounded; cps bounded; admission-runtime not-applicable; verification bounded.
**Evidence:**
- **Positive:** `TEST-TYPECK-CHECKED-HANDLER-SIDECAR`
- **Negative:** `TEST-TYPECK-V7-IMPORTED-HANDLER-ROW-INELIGIBLE`
- **Mutation:** `TEST-TYPECK-HANDLER-CORE-INSPECTION`
- **Parity:** not applicable; the typed handler slice does not claim direct-runtime parity.
**Non-goals:** General handler execution, inference, and residual-row realization.
**Next obligation:** Connect validated typed handler lowering to separately authorized production admission.

## TASK-2014 semantic workflow record

**Task:** [TASK-2014](tasks/TASK-2014-source-handler-runtime-boundary-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-EFFECT-HANDLE-001`
**Domain:** bounded
**Layers:** type bounded; core bounded; cps bounded; admission-runtime bounded; verification bounded.
**Evidence:**
- **Positive:** `TEST-ENGINE-CLOSED-EMPTY-HANDLER-PRODUCTION-RUN`
- **Negative:** `TEST-ENGINE-HANDLER-SOURCE-RUNTIME-CLOSED`
- **Mutation:** `TEST-ENGINE-FORGED-TRAP-SLEEP-CORE-CLASSIFICATION`
- **Parity:** not applicable; selected production admission is not a general parity relation.
**Non-goals:** General handler/provider execution or row-derived frame installation.
**Next obligation:** Admit only further validated handler forms with sealed bindings and terminal-envelope evidence.

## TASK-2031 λAsh-Effect correspondence record

**Task:** [TASK-2031](tasks/TASK-2031-lambda-ash-effect-correspondence.md)
**Status:** Complete
**Canonical rules:** `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`,
`OBS-TARGET-PROJECTION-001`, `SEM-EFFECT-LOOKUP-001`, `SEM-EFFECT-RAISE-001`,
`SEM-EFFECT-HANDLE-001`, `SEM-EFFECT-DISCHARGE-001`, `SEM-EFFECT-MISSDISCHARGE-001`,
`SEM-EFFECT-RESUME-001`, `SEM-EFFECT-HANDLERTRAP-001`, `SEM-EFFECT-PROVIDER-001`,
`SEM-EFFECT-ADMISSION-001`, `SEM-EFFECT-TIMEOUT-001`, `SEM-EFFECT-CANCEL-001`, and
`SEM-EFFECT-TERMINAL-001`.
**Domain:** general
**Layers:** type not-applicable; core not-applicable; cps general; admission-runtime
not-applicable; verification bounded.
**Evidence:**
- **Positive:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-CONTRACT`
- **Negative:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-INCOMPLETE-REJECTION`
- **Mutation:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-MISMAPPING-REJECTION`
- **Parity:** not applicable; this prerequisite-only mathematical correspondence has no active
  Engine, CLI, or daemon route; TASK-2032 owns integration parity.
**Non-goals:** Parser acceptance, Core lowering, admission/frame installation, Engine execution, and CLI/daemon parity.
**Next obligation:** TASK-2032 must consume this correspondence through the one shared admitted Engine path and prove client parity without a fallback evaluator.

## TASK-439 semantic workflow record

**Task:** [TASK-439](tasks/TASK-439-differential-conformance-harness-rust-first.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Domain:** bounded
**Layers:** type not-applicable; core bounded; cps bounded; admission-runtime not-applicable; verification bounded.
**Evidence:**
- **Positive:** `TEST-CPS-KERNEL-RETURN-CORPUS`
- **Negative:** `TEST-CPS-KERNEL-INPUT-REJECTION`
- **Mutation:** `TEST-CPS-KERNEL-LETPRIM-REJECTION`
- **Parity:** `TEST-DIRECT-RUNTIME-DIFFERENTIAL-HARNESS`
**Non-goals:** A complete canonical corpus executor or reference implementation.
**Next obligation:** Add canonical corpus cases only with a declared target, result relation, and non-passing divergence disposition.
