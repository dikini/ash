---
id: docs.plan.semantic-rule-coverage
title: Semantic Rule Coverage Map
kind: implementation-coverage-map
status: active
authority: planning-and-review
last_verified: 2026-07-27
---

# Semantic Rule Coverage Map

This is the human-review surface for target semantic-rule coverage. Canonical specs own the full
feature domain; `docs/spec/SEMANTIC-TRACEABILITY.json` owns machine-validated links. This map
reports each rule's implementation, evidence, parity, and missing target-spec clauses.

For semantic work, link a task to one or more rows below and update the row before writing a
fixture. A source example is evidence only. Every row reports these independent axes:

- **Implementation:** `implemented`, `partial`, or `not_implemented`.
- **Evidence:** `proved`, `tested`, or `none`.
- **Parity:** `matches_spec` or `below_spec`.

`implemented` requires realization of the rule's complete target-spec domain. A completed task or
layer handoff does not change an incomplete rule from `partial`/`below_spec`. New behavior outside
the target rule requires a specification update before implementation.

## How to read this map

This is a composition map of target-rule realization, not a whole-language progress scorecard.
Layer values are `implemented`, `partial`, `not_implemented`, or `not_applicable`.
`not_applicable` means the layer is outside the rule's realization path; `non-authorizing` means
the layer transports requirements or metadata without installing runtime/admission authority.

Read every omitted or non-authorizing layer as a named handoff, not a demand for cross-layer work.
For example, TASK-2013 produces checked typed-handler facts; TASK-2014 consumes those facts to
construct and authorize an admission artifact and frame instructions; TASK-2008 projects the
resulting terminal envelope. Handoffs, tests, and proofs provide evidence for their stated scope;
they do not by themselves establish target-spec parity.

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
- **Layer status:** Type partial; Core partial; CPS not_applicable; admission/runtime
  not_applicable; verification partial.
- **Missing target-spec clauses:** expression, call, closure, pattern, and import lowering.

### Calls and continuations

- **Canonical owner:** `SEM-CPS-CALL-001`, `SEM-CPS-JUMP-001`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** calls with parameters, closures, recursion, and imports.

### Core control and terminals

- **Canonical owner:** `SEM-CPS-LETVAL-001`, `SEM-CPS-IF-001`, `SEM-CPS-RETURN-001`,
  `SEM-CPS-TRAP-001`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** source control lowering beyond the currently realized forms.

**TASK-2031B evidence handoff:** verification-only reconciliation of two lexical-scope CLI negative
assertions to the existing checked Core-to-CPS bridge-domain rejection. It consumes
TASK-2003/TASK-2004/TASK-2014 admission facts and changes no Type, Core, CPS, admission/runtime,
or terminal layer.

**Handoff:** complete. **Evidence:** tested by the focused lexical-scope target 6/6 with the
canonical shared run-admission rejection; the recorded workspace Rust gate passed. This handoff
does not change production Rust or semantic-layer realization.

### Operations and lookup

- **Canonical owner:** `SEM-EFFECT-LOOKUP-001`, `SEM-EFFECT-RAISE-001`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** arbitrary operations, arguments, imports, and chains.

### Handlers and deep affine resume

- **Canonical owner:** `SEM-EFFECT-HANDLE-001`, `SEM-EFFECT-DEEP-AFFINE-HANDLE-001`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** multi-clause, open-row, imported, and multi-shot behavior.

### Rows and imported summaries

- **Canonical owner:** `SPEC-097b`, `TYPE-TARGET-ROW-001`
- **Layer status:** Type partial; Core partial metadata; CPS not_applicable; admission/runtime
  non-authorizing; verification partial.
- **Missing target-spec clauses:** row polymorphism, expansion, and discharge.

### Production admission and frames

- **Canonical owner:** `TASK-2004`, `TASK-2014`
- **Layer status:** Type partial; Core partial; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** production artifacts and route coverage required by the target
  rules.

### Terminal envelopes and async control

- **Canonical owner:** `TASK-2008`, `TASK-2014`
- **Layer status:** Type partial; Core not_applicable; CPS partial; admission/runtime partial;
  verification partial.
- **Missing target-spec clauses:** terminal outcomes and routes not yet realized.

**TASK-2031C prerequisite handoff:** Linux test-host verification that a programmatic
SIGINT reaches an isolated Tokio listener before TASK-2008's exact admitted `time::sleep` route is
evaluated. Type/Core/CPS/admission, the existing CLI forwarding, and Engine control precedence are
consumed existing layers. TASK-2008 consumes the terminal outcome, while TASK-2032 owns
client-parity integration.

**Handoff:** complete. **Evidence:** tested; the test-only probe explicitly classifies the managed
sandbox as unavailable; capable-host controls retain exit 130 and the exact V1 cancellation
envelope on stdout and `--output`. No production CLI/Engine or semantic-layer realization changed.

**TASK-2031F evidence handoff:** correction of three existing stdlib callable negative
assertions to TASK-2003's current PureAnf bridge-domain rejection. Type/Core/CPS/admission/runtime
layers are consumed existing behavior; run-route impact is none and TASK-2032 retains parity
evidence.

**Handoff:** complete. **Evidence:** tested by three controls retaining parse/check success and the
exact shared current PureAnf bridge-domain diagnostic; `module_resolution` passed 17/17. No
semantic-layer or production behavior changed.

### Engine-only client contracts

**TASK-2035 semantic workflow record:**
[TASK-2035](tasks/TASK-2035-canonical-client-test-contracts.md) defines
`CONF-SYNTH-SOURCE-WRAPPER-001`, `OBS-REPL-ENGINE-CLIENT-001`, and
`CONF-ENGINE-ONLY-CLIENT-001` for the one Engine executor route.

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec

**Missing target-spec clauses:** Realize every selected wrapper, REPL route, and daemon route through Engine; then realize the remaining target SPEC-077 and SPEC-011 domains before claiming parity.

**Layers:** type partial; core partial; cps partial; admission-runtime not_implemented;
verification not_implemented.

**Run-route impact:** prerequisite.

**Consumes:** `AUDIT-204-TEST-EXEC-002`, `AUDIT-204-REPL-001`, `AUDIT-204-REPL-002`, and the
seven named `AUDIT-204-DEFERRED-*` cases; target grammar/type/Core/CPS rules; and the existing
Engine admitted-request seam.

**Produces:** exact source-wrapper and fail-closed results in SPEC-077, the SPEC-011 REPL
Engine-client rule, and the SPEC-026 single-executor comparison rule.

**Downstream owner:** TASK-2038 implements test wrappers; TASK-2039 implements REPL; TASK-2042
implements daemon transport and `ash run` parity; TASK-2041 owns four-client parity.

**Evidence detail:** none. The source and deferred examples in TASK-2035 are contract text, not
test or proof evidence. **Parity evidence:** not applicable; no client route is realized by this
documentation task.

**Non-goals:** Source lowering, Engine APIs, test-runner execution, REPL execution, daemon transport, a general source synthesizer, and Lean implementation.

**Next obligation:** TASK-2038, TASK-2039, and TASK-2042 must implement their named routes with focused tests; TASK-2041 must establish the same-admitted-program four-client terminal comparison.

## TASK-2037 Engine-owned CPS executor boundary

**Task:** [TASK-2037](tasks/TASK-2037-engine-owned-cps-executor-and-runtime-crate-rename.md)
**Canonical rules:** `SEM-TARGET-CORE-CPS-001`, `SEM-EFFECT-ADMISSION-001`,
`OBS-TARGET-PROJECTION-001`, `CONF-ENGINE-ONLY-CLIENT-001`, `SEM-CPS-TRAP-001`,
`SEM-EFFECT-TIMEOUT-001`, and `SEM-EFFECT-CANCEL-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec

**Missing target-spec clauses:** Selected client routes, full target Core/CPS domains, deletion of direct-AST and differential material, and TASK-2041's four-client terminal comparison remain incomplete.

**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime partial;
verification partial.

**Run-route impact:** prerequisite.

**Consumes:** TASK-2035's Engine-only client contract; `AUDIT-204-CPS-001` through
`AUDIT-204-CPS-008`; checked Core/CPS artifacts; and Engine admission provenance.

**Produces:** the Engine-private checked-CPS executor boundary, migrated private CPS regression
coverage, and private Engine test placement for retained AUDIT-204 differential material. That
placement removes public invocation only; TASK-2040 retains the frozen-audit deletion ownership.
It does not activate a client route or rename the residual support crate.

**Downstream owner:** TASK-2038, TASK-2039, TASK-2040, and TASK-2042 consume this boundary;
TASK-2041 owns integration proof and API-absence closeout.

**Evidence detail:**
- **Positive:** `TEST-TASK-2037-ENGINE-OWNED-CPS-POSITIVE`
- **Trap:** `TEST-TASK-2037-ENGINE-OWNED-CPS-TRAP`
- **Timeout:** `TEST-TASK-2037-ENGINE-OWNED-CPS-TIMEOUT`
- **Cancellation:** `TEST-TASK-2037-ENGINE-OWNED-CPS-CANCELLATION`
- **Negative:** `TEST-TASK-2037-ENGINE-OWNED-CPS-NEGATIVE`
- **Mutation:** `TEST-TASK-2037-ENGINE-OWNED-CPS-MUTATION`
- **Parity:** not applicable; no client route or reference-executor comparison is performed by this
  prerequisite boundary task.

**Non-goals:** Test-runner, REPL, daemon, or ash run client-route implementation. Deletion of direct-AST evaluation, the Rust differential stack, or Lean material. Renaming ash-interp while TASK-2040-owned AST material remains. Transferring TASK-2040 deletion ownership when retained audit-listed differential tests move into Engine-private test modules.

**Next obligation:** TASK-2038, TASK-2039, TASK-2042, and TASK-2040 must consume the Engine-private executor boundary; TASK-2041 must prove API absence and four-client normalized-terminal parity.

### Differential parity

- **Canonical owner:** `TASK-2005`, `TASK-439`
- **Layer status:** Type not_applicable; Core partial private targets; CPS partial private targets;
  admission/runtime non-authorizing; verification partial.
- **Missing target-spec clauses:** conformance coverage beyond the trusted corpus controls.

### Contracts, predicates, and proofs

- **Canonical owner:** `SPEC-098b`, `SPEC-100`
- **Layer status:** Type partial; Core partial sidecars; CPS not_applicable; admission/runtime
  not_applicable; verification partial.
- **Missing target-spec clauses:** predicate discharge, proof, and runtime contract semantics.

## Required task record

Each linked task records: canonical rule/spec section; implementation, evidence, and parity status;
missing target-spec clauses; layer status; positive, negative, mutation, and parity evidence where
applicable; non-goals; and the next gap.
Each new or materially revised linked semantic task/record must also contain a **Handoffs** block
with its **Consumes**, **Produces**, intentionally unowned layer and its **downstream owner**, and
**integration/proof responsibility**. Reviewers reject a claim that a passing fixture completes a
target rule without target-spec parity and stated evidence.

## TASK-2001 semantic workflow record

**Task:** [TASK-2001](tasks/TASK-2001-target-grammar-gap-and-spec-conflict-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `GRAM-TARGET-MODULE-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Realize the remaining selected alias, group, handler, newtype, and row forms across their declared layers.
**Layers:** type partial; core partial; cps not_applicable; admission-runtime not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-ENGINE-V8-IMPORTED-HANDLER-ROW-E2E`
- **Negative:** `TEST-PARSER-STALE-DECLARATION-REJECTION`
- **Mutation:** `TEST-CORE-V8-STRUCTURAL-EFFECT-ROW-UNKNOWN-FIELD-REJECTION`
- **Parity:** not applicable; this parser/type-summary slice has no paired execution relation.
**Non-goals:** General grammar, row, and handler realization.
**Next obligation:** Realize the remaining selected alias, group, handler, newtype, and row forms across their declared layers.

## TASK-2002 semantic workflow record

**Task:** [TASK-2002](tasks/TASK-2002-generic-do-and-lowering-sidecar-strategy.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `LOWER-SURFACE-CORE-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Carry every required target sidecar or an explicit unsupported outcome through lowering.
**Layers:** type partial; core partial; cps not_applicable; admission-runtime not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-AMBIENT-DO-SOURCE-ENTRY-BOUNDARY`
- **Negative:** `TEST-ENGINE-NAMED-DO-TARGET-REJECTION`
- **Mutation:** `TEST-ENGINE-INVALID-HELPER-CONTRACT-SIDECAR-GUARD`
- **Parity:** not applicable; the retained sidecars are metadata, not a paired execution relation.
**Non-goals:** General sidecar completeness and runtime contract semantics.
**Next obligation:** Carry every required target sidecar or an explicit unsupported outcome through lowering.

## TASK-2003 semantic workflow record

**Task:** [TASK-2003](tasks/TASK-2003-return-authority-and-cps-kernel-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Extend checked Core/CPS realization only through separately admitted source forms.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence detail:**
- **Positive:** `TEST-ENGINE-SEALED-LOCAL-CALL-CORE-CPS-PRODUCTION`
- **Negative:** `TEST-CORE-CPS-RETURN-AUTHORITY`
- **Mutation:** `TEST-ENGINE-SEALED-LOCAL-CALL-PROVENANCE-GUARD`
- **Parity:** not applicable; this lowering route does not claim a full reference-runtime parity relation.
**Non-goals:** General source control, call, and continuation lowering.
**Next obligation:** Extend checked Core/CPS realization only through separately admitted source forms.

## TASK-2004 semantic workflow record

**Task:** [TASK-2004](tasks/TASK-2004-core-cps-production-boundary-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Admit further source forms only after validated typed lowering and checked Core/CPS evidence.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence detail:**
- **Positive:** `TEST-ENGINE-RUN-HANDLER-FREE-CHECKED-CPS-ADMISSION`
- **Negative:** `TEST-ENGINE-UNARY-NEGATION-PRODUCTION-REJECTION`, `TEST-ENGINE-RUN-FILE-UNARY-NEGATION-PRODUCTION-REJECTION`
- **Mutation:** `TEST-CORE-CPS-ADMISSION-GUARD`
- **Parity:** not applicable; the production boundary does not itself claim a parity relation.
**Non-goals:** A legacy direct-evaluator fallback or general source admission.
**Next obligation:** Admit further source forms only after validated typed lowering and checked Core/CPS evidence.

## TASK-2005 semantic workflow record

**Task:** [TASK-2005](tasks/TASK-2005-direct-runtime-core-cps-semantic-parity.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `OBS-TARGET-PROJECTION-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Expand only case-owned paired observables with explicit divergence dispositions.
**Layers:** type partial; core partial; cps partial; admission-runtime not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-DIRECT-RUNTIME-V3-INT-ADD-PARITY`
- **Negative:** `TEST-DIFFERENTIAL-TRUSTED-DIRECT-ORACLE-GATE`
- **Mutation:** `TEST-DIRECT-RUNTIME-ABSORB-SLEEP-HANDLER-FINGERPRINT-GUARD`
- **Parity:** `TEST-DIRECT-RUNTIME-V3-INT-ADD-PARITY`
**Non-goals:** General direct-runtime to production checked-CPS parity.
**Next obligation:** Expand only case-owned paired observables with explicit divergence dispositions.

## TASK-2008 semantic workflow record

**Task:** [TASK-2008](tasks/TASK-2008-json-variant-observable-projection.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `OBS-TARGET-PROJECTION-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Add canonical envelope cases only with an admitted checked route and focused observable evidence.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence detail:**
- **Positive:** `TEST-CLI-CANONICAL-TERMINAL-PROJECTION`
- **Negative:** `TEST-CLI-UNADMITTED-TRAP-SLEEP-TERMINAL-ENVELOPE`
- **Mutation:** `TEST-CLI-POSTEXECUTION-INVALID-EXIT-PROJECTION`
- **Parity:** not applicable; terminal projection is not a direct-runtime parity claim.
**Non-goals:** A complete terminal matrix for every future execution route.
**Next obligation:** Add canonical envelope cases only with an admitted checked route and focused observable evidence.

## TASK-2013 semantic workflow record

**Task:** [TASK-2013](tasks/TASK-2013-source-handler-and-handle-lowering.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-EFFECT-HANDLE-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Connect validated typed handler lowering to separately authorized production admission.
**Layers:** type partial; core partial; cps partial; admission-runtime not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-TYPECK-CHECKED-HANDLER-SIDECAR`
- **Negative:** `TEST-TYPECK-V7-IMPORTED-HANDLER-ROW-INELIGIBLE`
- **Mutation:** `TEST-TYPECK-HANDLER-CORE-INSPECTION`
- **Parity:** not applicable; the typed handler slice does not claim direct-runtime parity.
**Non-goals:** General handler execution, inference, and residual-row realization.
**Next obligation:** Connect validated typed handler lowering to separately authorized production admission.

## TASK-2014 semantic workflow record

**Task:** [TASK-2014](tasks/TASK-2014-source-handler-runtime-boundary-decision.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-EFFECT-HANDLE-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Admit only further validated handler forms with sealed bindings and terminal-envelope evidence.
**Layers:** type partial; core partial; cps partial; admission-runtime partial; verification partial.
**Evidence detail:**
- **Positive:** `TEST-ENGINE-CLOSED-EMPTY-HANDLER-PRODUCTION-RUN`
- **Negative:** `TEST-ENGINE-HANDLER-SOURCE-RUNTIME-CLOSED`
- **Mutation:** `TEST-ENGINE-FORGED-TRAP-SLEEP-CORE-CLASSIFICATION`
- **Parity:** not applicable; selected production admission is not a full parity relation.
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
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** TASK-2032 must consume this correspondence through the one shared admitted Engine path and prove client parity without a fallback evaluator.
**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime
not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-CONTRACT`
- **Negative:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-INCOMPLETE-REJECTION`
- **Mutation:** `TEST-DOCS-TASK-2031-EFFECT-CORRESPONDENCE-MISMAPPING-REJECTION`
- **Parity:** not applicable; this prerequisite-only mathematical correspondence has no active
  Engine, CLI, or daemon route; TASK-2032 owns integration parity.
**Non-goals:** Parser acceptance, Core lowering, admission/frame installation, Engine execution, and CLI/daemon parity.
**Next obligation:** TASK-2032 must consume this correspondence through the one shared admitted Engine path and prove client parity without a fallback evaluator.

## TASK-2032 shared Engine execution seam record

**Task:** [TASK-2032](tasks/TASK-2032-shared-engine-execution-seam-and-client-parity.md)
**Canonical rules:** `SEM-TARGET-CORE-CPS-001`, `OBS-TARGET-PROJECTION-001`,
`SEM-EFFECT-ADMISSION-001`, `SEM-EFFECT-HANDLERTRAP-001`, `SEM-EFFECT-TIMEOUT-001`,
`SEM-EFFECT-CANCEL-001`, and `SEM-EFFECT-TERMINAL-001`.
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** A separately owned daemon transport/profile/binding task must carry an admitted request and V1 terminal envelope before a selected noncanonical provider or handler route can be daemon-active.
**Layers:** type not_applicable; core not_applicable; cps partial; admission-runtime partial;
verification partial.
**Run-route impact:** active. This task consumes selected checked artifacts and terminal
projection into one Engine execution seam; it does not claim target executor behavior.
**Consumes:** TASK-2004/TASK-2014 checked admissions and authorized frames, TASK-2008 terminal
projection, and TASK-2031 correspondence.
**Produces:** opaque Engine admitted-program request/result integration, in-process client adapters
over the same request, explicit daemon-service activation/rejection evidence, and the
`RUNNABLE-ASH-MATRIX.md` ledger.
**Downstream owner:** Feature-realization tasks own each matrix source/lowering/provider gap;
TASK-2032 retains integration/parity evidence for the selected artifact slices.
**Evidence detail:**
- **Positive:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-POSITIVE` (including the exact
  `deep_affine_clock` checked-CPS `Int(107)` slice)
- **Negative:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-NEGATIVE`
- **Mutation:** `TEST-TASK-2032-SHARED-ENGINE-SEAM-MUTATION`
- **Parity:** `TEST-TASK-2032-CLIENT-ADAPTER-TERMINAL-PARITY` and
  `TEST-TASK-2032-CLIENT-ADAPTER-DEADLINE-REUSE-PARITY` (same in-process request only)
- **Daemon service boundary:** `TEST-TASK-2032-DAEMON-SOURCE-REJECTION`
**Non-goals:** Parser acceptance, Core/CPS lowering, provider implementation, handler semantics, frame authorization, terminal taxonomy, and daemon transport redesign.
**Next obligation:** A separately owned daemon transport/profile/binding task must carry an admitted request and V1 terminal envelope before a selected noncanonical provider or handler route can be daemon-active.

## TASK-439 semantic workflow record

**Task:** [TASK-439](tasks/TASK-439-differential-conformance-harness-rust-first.md)
**Canonical rules:** `CONF-IMPLEMENTATION-001`, `CORE-CPS-SYNTAX-001`, `SEM-TARGET-CORE-CPS-001`
**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Add canonical corpus cases only with a declared target, result relation, and non-passing divergence disposition.
**Layers:** type not_applicable; core partial; cps partial; admission-runtime not_applicable; verification partial.
**Evidence detail:**
- **Positive:** `TEST-CPS-KERNEL-RETURN-CORPUS`
- **Negative:** `TEST-CPS-KERNEL-INPUT-REJECTION`
- **Mutation:** `TEST-CPS-KERNEL-LETPRIM-REJECTION`
- **Parity:** `TEST-DIRECT-RUNTIME-DIFFERENTIAL-HARNESS`
**Non-goals:** A complete canonical corpus executor or reference implementation.
**Next obligation:** Add canonical corpus cases only with a declared target, result relation, and non-passing divergence disposition.
