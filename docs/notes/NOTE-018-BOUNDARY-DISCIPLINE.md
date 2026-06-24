# NOTE-018: Boundary Discipline for Target Ash

**Date:** 2026-06-24
**Status:** Living document — inventory in progress
**Purpose:** Define the target Ash boundary discipline: where values, authority, effects,
failures, evidence, memory, apps, providers, and host operations cross from one semantic
region to another. Companion to NOTE-015 (language forms), NOTE-016 (runtime organization),
NOTE-017 (memory regions), NOTE-013 (ambient monad and handler composition), and NOTE-014
(contract systems unification).

## 0. Motivation

The target Ash story is now centered on one ambient computation model with effect rows,
handlers/providers, contracts, evidence, and explicit runtime admission. The remaining
ambiguity is concentrated around boundaries.

Boundary questions have a recurring shape:

```text
What crosses?
Who owns it after crossing?
Which row items, contracts, evidence, or authority are required?
Which failures are possible?
Which trace/provenance records are produced?
Which parts are language semantics and which parts are implementation strategy?
```

This note starts with an inventory. Later revisions should turn each boundary into a
resolved/to-resolve contract.

## 1. Boundary Inventory

### 1.1 Surface-to-Core boundary

**Description:** The boundary between user-facing Ash syntax and Core Ash. Target Ash
should not let legacy workflow, capability, or tower syntax define separate semantics.
Surface forms elaborate into Core terms, row facts, contract discharge metadata, and
sidecar evidence declarations.

**Affects:**

- `workflow`, `act`, `do:Act`, `do:Proc`, `do:Workflow`, `ret`, and workflow statements;
- `capability`, role, policy, resource, law, proof, property, and proposition declarations;
- row annotation syntax, inferred row summaries, and diagnostics;
- source spans and rewrite hints during migration.

**Options:**

1. Keep broad compatibility syntax and lower it aggressively to Core.
2. Introduce canonical target syntax first, then deprecate legacy forms once equivalence
   tests exist.
3. Keep some domain-friendly surface forms permanently, but require their lowering to be
   specified as sugar over Core.

**References:**

- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [SPEC-095b](../spec/SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-099](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-100](../spec/SPEC-100-CORE-TYPE-CHECKING.md)

### 1.2 Function and closure boundary

**Description:** The boundary at ordinary callable entry and exit. Function types must say
which row a body may require, which contracts attach at entry/exit, and what closure
captures are legal.

**Affects:**

- row-bearing function and closure types;
- closure capture of capabilities, resources, process-local values, handlers, and
  continuations;
- precondition/postcondition checking and blame;
- public module summaries and effect-safe higher-order programming.

**Options:**

1. Treat every function as row-bearing and infer rows unless annotated.
2. Preserve pure `Fn` and effectful `Fun` compatibility types during migration.
3. Add stricter closure-capture predicates for process-local, region-local, and authority
   carrying values.

**References:**

- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [NOTE-014](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [SPEC-031](../spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md)
- [SPEC-088](../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [SPEC-097b](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)

### 1.3 Row environment and admission boundary

**Description:** The boundary between a computation's requirement row and the ambient
environment that discharges it. A row is not a grant. Admission, providers, handlers,
roles, policies, resources, and evidence determine whether the requirement is satisfied.

**Affects:**

- row inclusion and open-row solving;
- role/capability entailment;
- policy binding and decision domains;
- resource ownership and borrowing;
- diagnostics for missing or invalid discharge.

**Options:**

1. Discharge most row items statically when possible and leave runtime admission as a
   boundary check.
2. Keep admission primarily runtime-owned, with the type checker producing summaries and
   compatibility obligations.
3. Split discharge by kind: static/evidence/dynamic for contracts, admission for authority,
   ownership for resources, handler/provider frames for operations.

**References:**

- [SPEC-096b](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [DESIGN-041](../design/DESIGN-041-RUNTIME-REGIME-AND-OS-SURFACE.md)

### 1.4 Effect declaration and operation boundary

**Description:** The boundary between declaring an operation vocabulary and requiring or
executing an operation. This is where current capabilities likely become effect operations
plus contracts, provider/handler discharge, and optional extern hooks.

**Affects:**

- `effect` versus `capability` as canonical authoring form;
- canonical operation identity and row item spelling;
- operation argument/result contracts;
- provider and handler implementation shape;
- operation namespace export/import and versioning.

**Options:**

1. Make `effect` canonical and lower `capability` to restricted effect declarations.
2. Keep `capability` as a domain-specific authoring form for authority-bearing effects.
3. Support both, but require one canonical Core/CPS operation identity.

**References:**

- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [NOTE-014](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [SPEC-017](../spec/SPEC-017-CAPABILITY-INTEGRATION.md)
- [SPEC-052](../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md)
- [SPEC-096b](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)

### 1.5 Handler and provider boundary

**Description:** The boundary where operation requirements are interpreted. Handlers peel
operation items from a row and may resume, discard, delay, or duplicate continuations.
Providers install runtime-backed interpretations and may own authority or resources.

**Affects:**

- handler stack order and non-commutative interpretation;
- residual row computation;
- continuation multiplicity and memory retention;
- provider lifetime and authority provenance;
- retry, state, nondeterminism, failure, and host adapter semantics.

**Options:**

1. Treat surface handlers as explicit `handle` scopes lowering to Core/CPS `Handle`.
2. Treat providers as trusted runtime handler frames installed by admission.
3. Distinguish pure/library handlers from authority-bearing providers in diagnostics and
   evidence.

**References:**

- [NOTE-013](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [SPEC-098b](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-102](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)

### 1.6 Extern and host boundary

**Description:** The boundary between Ash semantics and host/FFI execution. Raw host calls
must not be ordinary Ash functions. They are unsafe implementation hooks owned by effects,
trusted handlers, or providers.

**Affects:**

- extern syntax and placement;
- ABI contract checking and host error classification;
- authority provenance for OS, filesystem, network, clock, model, and tool access;
- auditability of trusted code;
- distinction between safe Ash calls and raw host adapters.

**Options:**

1. Put canonical extern hooks in effect declarations.
2. Put backend-specific extern hooks inside trusted provider/handler implementations.
3. Allow both placements under one invariant: ordinary Ash code calls typed operations, not
   raw externs.

**References:**

- [NOTE-014](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN](../design/DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN.md)
- [SPEC-017](../spec/SPEC-017-CAPABILITY-INTEGRATION.md)

### 1.7 Failure boundary

**Description:** The boundary where an abnormal condition becomes a recoverable failure,
an unrecoverable trap, a denied admission, a policy decision, a process exit, or a workflow
boundary result. Target Ash needs these categories separated.

**Affects:**

- `fail`, `panic`, `with_error`, dynamic contract violations, and host ABI failures;
- row spelling for recoverable failures;
- process and workflow terminal states;
- supervision restart policy;
- reports, traces, and blame.

**Options:**

1. Keep `Trap` for unrecoverable diagnostic aborts and use explicit `fail` effects for
   recoverable domain failures.
2. Treat contract violation as trap by default, with explicit lowering to recoverable
   failure when requested.
3. Separate admission denial, policy denial, authority absence, host adapter failure, and
   process failure in diagnostics and evidence even if some share runtime carriers.

**References:**

- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [NOTE-014](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [SPEC-050](../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md)
- [SPEC-099](../spec/SPEC-099-CORE-LANGUAGE.md)
- [DESIGN-041](../design/DESIGN-041-RUNTIME-REGIME-AND-OS-SURFACE.md)

### 1.8 Contract and evidence boundary

**Description:** The boundary between asserted obligations and discharged obligations.
Contracts may be proven statically, backed by evidence, checked dynamically, or reported as
unresolved. Laws and properties have different lifecycles.

**Affects:**

- `requires`, `ensures`, invariants, obligations, laws, proofs, properties, propositions;
- caller/callee blame;
- interface-to-impl precondition and postcondition variance;
- law evidence and property-test reports;
- reflection and audit records.

**Options:**

1. Keep properties outside the effect row as falsification metadata only.
2. Keep laws as evidence obligations discharged once per implementation.
3. Represent dynamic Hoare failures as traps by default, with explicit recoverable failure
   lowering where the surface chooses it.

**References:**

- [NOTE-014](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [SPEC-080](../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [SPEC-081](../spec/SPEC-081-LAW-TEST-EVIDENCE-SUBSTRATE.md)
- [SPEC-098b](../spec/SPEC-098b-TARGET-IR.md)

### 1.9 Process and channel boundary

**Description:** The boundary between isolated processes and their communication paths.
Message passing crosses ownership, memory, effect, failure, and scheduling boundaries.

**Affects:**

- `spawn`, `send`, `receive`, `await`, `join`, cancellation, monitor links;
- channel payload sendability and guard contracts;
- process-local values, closures, continuations, and resource handles;
- mailbox size, timeout, ordering, fairness, and backpressure;
- process exit observation and supervisor behavior.

**Options:**

1. Move owned sendable values by default.
2. Copy explicitly copyable values.
3. Share only through explicit shared/resource handles.
4. Reject process-local, region-local, or unsafe captured values.
5. Defer full session/MPST typing while preserving hooks for later protocol checks.

**References:**

- [NOTE-016](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [NOTE-017](NOTE-017-MEMORY-REGIONS-OWNERSHIP-AND-UTILIZATION.md)
- [SPEC-049](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md)
- [SPEC-096b](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [DESIGN-NOTE-PROCESS-EFFECT](../design/DESIGN-NOTE-PROCESS-EFFECT.md)

### 1.10 Memory and region boundary

**Description:** The boundary between process regions, app regions, iteration subregions,
retained state, and shared resource handles. It explains what memory is released when a
process ends and what may survive.

**Affects:**

- process-region allocation and cleanup;
- iteration subregions for long-lived loops;
- state/resource retention;
- graph history and stream buffering;
- continuation capture, delayed resume, and multi-shot reuse;
- future region inference and Perceus-like reuse.

**Options:**

1. Start with logical process regions and Rust-backed allocation.
2. Add iteration subregions for long-lived services.
3. Add real arenas/pools/slabs only when runtime evidence justifies them.
4. Keep user-visible semantics independent of allocator strategy.

**References:**

- [NOTE-017](NOTE-017-MEMORY-REGIONS-OWNERSHIP-AND-UTILIZATION.md)
- [NOTE-016](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [SPEC-049](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md)
- [SPEC-102](../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md)

### 1.11 App and runtime-kernel boundary

**Description:** The boundary between loaded definitions and running app instances. Files,
modules, workflows, providers, and graph blueprints do not run merely because they exist.
The runtime admits app instances explicitly.

**Affects:**

- `RuntimeKernel`, `AppDefinition`, `AppInstance`, root supervisors, child specs;
- app-local provider/resource admission;
- app-local process and graph namespaces;
- inter-app communication and host routing;
- daemon policy, reload, restart, shutdown, and reporting.

**Options:**

1. Add source-level `app` declarations.
2. Use external manifests or package metadata.
3. Export ordinary Ash values under a known app-spec name.
4. Support multiple app sources while requiring one canonical runtime `AppDefinition`.

**References:**

- [NOTE-016](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [SPEC-070](../spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md)
- [DESIGN-041](../design/DESIGN-041-RUNTIME-REGIME-AND-OS-SURFACE.md)
- [WORKFLOW_SPAWNING_AND_SUPERVISION](../design/WORKFLOW_SPAWNING_AND_SUPERVISION.md)

### 1.12 Behaviour and service-runner boundary

**Description:** The boundary between a static interface/impl and the runtime loop that
executes it. Erlang-like behaviours should be library protocols plus runners, not
privileged language forms.

**Affects:**

- `GenServer`, `Supervisor`, `Stage`, `Source`, `Sink`, `Router`, `AgentLoop`;
- callback contracts and inherited row requirements;
- runner-owned process startup and mailbox protocols;
- specialization, dictionary passing, and public service handles;
- termination and supervisor integration.

**Options:**

1. Model behaviours as ordinary interfaces plus explicit runner functions.
2. Allow library runners to generate child specs and service handles.
3. Keep behaviour syntax out of the language core unless evidence shows repeated boilerplate
   cannot be solved by libraries.

**References:**

- [NOTE-016](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [SPEC-080](../spec/SPEC-080-INTERFACE-EVIDENCE-CONSTRAINTS.md)
- [OTP-002](../ideas/otp/OTP-002-ash-otp-design.md)
- [OTP-003](../ideas/otp/OTP-003-genserver-design-patterns.md)

### 1.13 Reactive stream and graph boundary

**Description:** The boundary between pull streams, push events, and declarative FRP
graphs. These are related but should not be collapsed into `workflow`.

**Affects:**

- `Producer`, `Consumer`, `Pipe`, `Machine`, event emit/subscribe, channels, graph
  definitions, graph interpreters;
- buffering, backpressure, dropping, replay, clocks, scheduling, and glitch freedom;
- state cells, graph history, memo caches, and retention contracts;
- bridge adapters between pull, push, and graph modes.

**Options:**

1. Treat pull as codata/machine protocols.
2. Treat push as channel/event effects with explicit buffering policy.
3. Treat graphs as blueprints interpreted by app/supervisor-started graph interpreters.
4. Define bridge adapters explicitly rather than implicit conversions.

**References:**

- [NOTE-016](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [NOTE-017](NOTE-017-MEMORY-REGIONS-OWNERSHIP-AND-UTILIZATION.md)
- [SPEC-013](../spec/SPEC-013-STREAMS.md)
- [DESIGN-NOTE-COMONADIC-COMPUTATION](../design/DESIGN-NOTE-COMONADIC-COMPUTATION.md)
- [effectful-stream-sinks](../design/effectful-stream-sinks.md)

### 1.14 Module, package, and summary boundary

**Description:** The boundary between local declarations and imported/public facts.
Rows, aliases, groups, types, laws, effect identities, and evidence must survive module
export/import without accidentally granting authority or losing invalidation dependencies.

**Affects:**

- module summaries and public row summaries;
- effect alias/group export and versioning;
- law/proof evidence reuse;
- sealed domains and associated type families;
- dependency invalidation and diagnostics across packages.

**Options:**

1. Export normalized canonical identities plus user-facing display names.
2. Keep aliases transparent and groups diagnostic-only.
3. Require evidence references to be versioned and invalidated when dependencies change.
4. Never export authority grants as mere row aliases.

**References:**

- [NOTE-015](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)
- [SPEC-062](../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
- [SPEC-096b](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-100](../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## 2. Cross-Cutting Boundary Contract

Every boundary should eventually answer the same minimum questions:

1. **Carrier:** what representation crosses the boundary?
2. **Ownership:** who owns it before and after crossing?
3. **Authority:** which row items or admission facts are required?
4. **Contracts:** which predicates, laws, or obligations apply?
5. **Failure:** which failure categories can arise?
6. **Evidence:** which discharge, trace, report, or provenance record is produced?
7. **Lifetime:** how long may the value, authority, resource, or evidence survive?
8. **Migration:** which legacy forms lower to this boundary?

## 3. Working Principle

The boundary rule:

```text
Nothing crosses a target Ash boundary by accident. The language, type checker, runtime,
or library must name the carrier, ownership rule, authority/evidence requirement, failure
classification, and lifetime policy.
```

## 4. References

Internal references:

- [NOTE-013: Ambient Monad and Handler Composition Algebra](NOTE-013-AMBIENT-MONAD-AND-HANDLER-COMPOSITION-ALGEBRA.md)
- [NOTE-014: Contract Systems Unification](NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [NOTE-015: Current-to-Target Language Forms](NOTE-015-CURRENT-TO-TARGET-LANGUAGE-FORMS.md)
- [NOTE-016: Runtime Organization, Behaviours, and Reactive Modes](NOTE-016-RUNTIME-ORGANIZATION-BEHAVIOURS-REACTIVE-MODES.md)
- [NOTE-017: Memory Regions, Ownership, and Utilization](NOTE-017-MEMORY-REGIONS-OWNERSHIP-AND-UTILIZATION.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099: Core Ash](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## 5. Changelog

- 2026-06-24: Initial inventory. Lists target Ash boundaries and expands each with a
  description, affected features, design options, and references.
