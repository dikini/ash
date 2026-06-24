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

**Current decision pass:** See §4.

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

**Current decision pass:** See §3.

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

**Current decision pass:** See §3.

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

**Current decision pass:** See §2.

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

**Current decision pass:** See §5.

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

**Current decision pass:** See §5.

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

**Current decision pass:** See §6.

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

## 2. Decision Pass A: Failure Boundary

### 2.1 Decision

Target Ash should not have one undifferentiated "failure" concept. It should classify
abnormal outcomes by boundary first, then choose the representation appropriate to that
boundary.

Resolved direction for the first target slice:

1. **Recoverable domain failure** is the ordinary `fail` effect path. It is row-accounted and
   may be handled by user code.
2. **Unrecoverable trap/bottom** is not row-accounted. It aborts the current computation with
   diagnostic state.
3. **Dynamic contract violation** defaults to structured trap/bottom, carrying contract
   discharge and blame metadata. A surface form may explicitly lower a contract violation to a
   recoverable `fail` path when recovery is part of the declared protocol.
4. **Authority absence/admission denial** is a discharge/admission failure, not a raised
   capability operation. It means the required authority was never installed.
5. **Policy denial** is a named policy decision. It must remain distinct from missing
   authority and from host/provider failure.
6. **Host ABI/provider failure** belongs to the trusted adapter/provider boundary. It is not
   proof that the Ash caller lacked authority or violated a contract.
7. **Process failure/cancellation** belongs to process supervision and observation. It may be
   reported through handles, monitors, joins, or supervisor events.
8. **Workflow/app boundary failure** is a boundary reinterpretation/reporting layer over lower
   failures, admission denials, policy denials, and process exits.

The key split:

```text
row-accounted fail     -- recoverable, part of the program protocol
trap/bottom            -- unrecoverable diagnostic abort
admission rejection    -- cannot start/run because requirements are not discharged
boundary report        -- workflow/app/runtime classification of lower causes
```

### 2.2 Failure taxonomy

| Category | Boundary | Row/IR posture | Handler posture | Evidence/reporting |
|---|---|---|---|---|
| recoverable domain failure | function/effect protocol | `FailureEffect` / `fail` row item | user handler may recover | failure payload and handler trace |
| unrecoverable trap | Core/runtime diagnostic | `Trap`, no row item | not resumable | trap reason, span, continuation/runtime context |
| dynamic contract violation | contract boundary | default `TrapReason::ContractViolation`; optional explicit `fail` lowering | not resumable by default | contract id, discharge mode/history, values, blame |
| authority absence | row admission boundary | no operation is admitted | no handler installed for missing authority | missing row item, admission context |
| policy denial | policy boundary | policy decision boundary, not generic capability failure | may branch only through explicit policy protocol | policy name, decision domain, decision evidence |
| host ABI failure | extern/provider boundary | provider/adapter failure; may map to `fail` only if operation declares it | provider-owned recovery only unless surfaced | adapter id, host error, decode/encode context |
| process failure | process/supervision boundary | process terminal state | observed by monitor/join/supervisor | exit reason, child id, restart decision |
| cancellation | process/runtime boundary | cancellation terminal state | cooperative cleanup, not ordinary recovery | initiator, scope, cleanup result |
| workflow failure | workflow boundary | boundary report over lower cause | workflow-level handler/report policy | workflow id, lower cause, obligations/reports |
| app rejection/failure | app/runtime boundary | app admission or lifecycle result | host/operator policy | app id, root supervisor/app policy, lower cause |

### 2.3 Mapping current forms

| Current form/source | Target classification |
|---|---|
| `fail` | recoverable failure effect when the surrounding profile admits it |
| `with_error` | handler/library surface over recoverable failure effects |
| `panic` | trap/debug abort, not domain failure |
| failed `requires`/`ensures` | contract violation; trap by default unless explicitly recoverable |
| missing capability/provider binding | authority/admission failure |
| denied policy decision | policy denial with decision evidence |
| host call throws/returns invalid data | host ABI/provider failure |
| child process exits abnormally | process failure observed by supervisor/monitor/join |
| cancelled process/app | cancellation boundary result |
| workflow rejects before start | admission/requirement/policy rejection |
| workflow aborts while running | workflow failure report over lower cause |

### 2.4 Consequences

This decision avoids four common conflations:

1. **Authority vs correctness:** missing authority is not a contract violation.
2. **Policy vs authority:** a policy can deny an otherwise available authority.
3. **Host failure vs caller failure:** a trusted adapter can fail even when admission and
   contracts were satisfied.
4. **Recoverable failure vs trap:** user recovery is explicit and row-accounted; traps are
   diagnostic bottom.

Diagnostics should therefore avoid a generic `UnhandledEffect` or generic `RuntimeFailure`
when a more precise boundary class is known.

### 2.5 Still to resolve

1. Concrete source spelling for typed recoverable failures.
2. Whether `fail E` uses a single `FailureEffect { failure_type: E }` item or namespaced
   failure operations.
3. Whether `with_error` remains compatibility syntax or becomes a library handler.
4. Exact workflow/app report schema for lower causes.
5. How much of policy denial is represented as a row item versus boundary result.
6. Whether provider/host failures default to trap, recoverable failure, or declared
   operation-specific result types.
7. Blame labels and value redaction rules for contract violation diagnostics.

## 3. Decision Pass B: Effect Declaration and Extern/Host Boundaries

### 3.1 Decision

Target Ash should separate three things that current Ash often discusses together:

```text
operation vocabulary     -- what can be requested
authority/admission      -- who may request it in this context
host/provider adapter    -- how the request is implemented
```

Resolved direction for the first target slice:

1. **Canonical operation identity lives below surface spelling.** Whether the source says
   `effect`, `capability`, or a compatibility form, Core/CPS sees one canonical operation
   identity and one row item.
2. **`effect` is the target vocabulary for operation declarations.** It names typed
   operation signatures, row contribution, contracts, and optional implementation hooks.
3. **`capability` remains a restricted/domain-friendly compatibility surface.** It lowers to
   authority-bearing effect operations plus admission/provider metadata. It is not a
   separate semantic island.
4. **User-defined effects are a target-language direction, but an alpha staging choice.**
   The design vocabulary uses `effect` for ordinary algebraic-operation declarations such
   as failure, choice, and host capabilities. Current alpha specs may restrict which
   namespaces lower to Core/CPS until fully general user-defined resumable effects are
   specified across the effect, type, Core, and IR specs.
5. **Externs are implementation hooks, not ordinary Ash functions.** Ordinary Ash code calls
   typed operations. Trusted effects, handlers, or providers own raw host/FFI adapters.
6. **Extern placement is split by intent.** Canonical host ABI hooks may appear at the
   effect declaration boundary; backend-specific host adapters belong in trusted
   provider/handler implementations.
7. **Provider installation is admission, not definition.** Declaring an effect/capability
   does not install authority. Runtime admission installs a provider/handler frame that may
   discharge the row item.

### 3.2 Canonical lowering shape

The target lowering story should be:

```text
surface declaration
  -> canonical operation identity
  -> row item required by callers
  -> operation contracts
  -> provider/handler implementation requirement
  -> optional trusted extern adapter
```

For a capability-like declaration:

```ash
capability FsRead : read(path: String) returns String
    requires path != "";
```

Target meaning:

```text
operation identity: cap fs.read
row item:           {cap fs.read}
contracts:          requires path != ""
implementation:     admitted provider/handler for fs.read
extern:             optional trusted host hook owned by provider/effect
```

For an authority-bearing effect-like declaration:

```ash
effect Fs {
    read(path: String) -> String
        requires path != "";
}
```

Target meaning is the same canonical operation identity if the declaration is authority
bearing:

```text
EffectOp(cap fs.read) : (String) -> String
row {cap fs.read}
```

For a pure library algebraic effect such as choice, the canonical identity would live in a
non-capability namespace:

```text
EffectOp(effect choice.choose) : (List<A>) -> A
row {effect choice.choose}
```

The surface may be friendlier than the Core identity, but it must not create a second
operation namespace with different semantics.

### 3.3 Extern placement

Externs should be admitted only at trusted implementation boundaries:

| Placement | Use case | Visibility to ordinary Ash code |
|---|---|---|
| effect-level extern | canonical host ABI for a standard operation | hidden behind typed operation |
| provider-level extern | backend-specific adapter or deployment-specific host binding | hidden behind admitted provider |
| handler-level extern | trusted interpreter for an effect operation | hidden behind handler installation |
| ordinary `extern fn` | compatibility or bootstrap only, not target user model | should not be directly callable as pure Ash |

The invariant:

```text
ordinary Ash code calls typed operations;
trusted implementation code calls raw externs.
```

### 3.4 Boundary failure split

Effect declaration and host boundaries reuse the failure taxonomy from §2:

| Cause | Boundary classification |
|---|---|
| operation not admitted | authority/admission failure |
| provider missing | admission/runtime configuration failure |
| operation precondition fails | contract violation |
| policy denies operation | policy denial |
| host call fails | host ABI/provider failure |
| host result cannot decode | host ABI/provider failure, possibly contract violation if the provider broke its declared result contract |
| user wants recoverable operation error | operation declares result/failure protocol explicitly |

This preserves the key distinction:

```text
row item says the operation may be required;
admission says whether it may run here;
provider/handler says how it runs;
extern says how trusted implementation reaches the host.
```

### 3.5 Still to resolve

1. Exact target syntax for `effect` declarations.
2. Whether `capability` is permanently supported as a domain form or only migration syntax.
3. Whether effect-level externs are allowed in user-authored source, trusted packages only,
   or compiler/runtime-owned modules only.
4. How provider implementations are declared and typed.
5. How operation contracts are checked at provider boundaries versus caller boundaries.
6. How effect aliases/groups export operation identities without becoming authority bundles.
7. How canonical operation identities are versioned across packages.

## 4. Decision Pass C: Row Environment and Admission Boundary

### 4.1 Decision

Target Ash should treat rows and admission as separate layers:

```text
requirement row      -- what the computation may need
ambient environment  -- what this scope/context can discharge
admission event      -- what a runtime boundary installs for an instance
```

Resolved direction for the first target slice:

1. **Rows are canonical requirement facts, not authority grants.** A row item can be
   inferred, normalized, exported, imported, and compared without installing the authority or
   resource it names.
2. **The ambient environment carries discharge facts.** It may contain admitted roles,
   capability/provider bindings, owned channel endpoints, resource ownership facts, policy
   handlers, contract evidence, failure handlers, and evidence sinks.
3. **Discharge is kind-specific.** Capability, role, policy, contract, resource, channel,
   process, failure, and evidence row items do not share one generic "handled effect" rule.
4. **Admission is explicit at runtime boundaries.** Starting a workflow/app/process or
   installing a provider/handler can add discharge facts to the environment. Merely loading
   a declaration cannot.
5. **Role entailment is discharge, not row normalization.** A role may discharge a capability
   requirement when admitted, but the required row item remains `cap ...` for audit and
   diagnostics.
6. **Aliases and groups are not authority bundles.** They expand row spelling or improve
   diagnostics, but they do not discharge requirements.
7. **Open-row solving must not invent privilege.** A row variable can be constrained by use,
   expected type, or public summary shape; it must not absorb ambient authority just because
   authority happens to be available.

### 4.2 Discharge matrix

| Row item kind | Requirement means | Discharged by | Not discharged by |
|---|---|---|---|
| capability | operation authority may be needed | admitted capability binding, provider frame, or admitted role entailment | row alias, declaration existence, global provider presence |
| resource | owned/borrowed resource access may be needed | ownership, borrow, split/join, provenance fact, admitted resource handle | capability authority alone |
| role | role-specific authority/context may be needed | role admission at workflow/app/process boundary | role declaration existence |
| policy | named policy decision may be needed | compatible named policy binding/evaluator/handler | anonymous boolean expression unless explicitly lowered |
| contract | predicate/law/obligation must be discharged | static proof, evidence, dynamic check strategy, law proof, runtime contract handler | property test alone, silent erasure |
| channel | endpoint operation may be needed | owned endpoint with compatible direction/message/guard facts | channel type name alone |
| process | runtime process operation may be needed | process-capable profile/runtime context | `Act` profile or plain function context |
| failure | recoverable failure route may be needed | failure-capable profile and handler/reporting policy | trap support alone |
| evidence | proof/audit/report sink may be needed | available evidence sink or boundary recorder | ordinary logging capability unless admitted as evidence sink |

### 4.3 Checking phases

Row admission should be visible across phases:

```text
surface/Core checking
  infer and normalize requirement rows
  reject impossible/local violations
  emit public summaries

admission planning
  compare root/app/workflow requirements with available grants/config
  construct ambient discharge environment

runtime execution
  run under admitted providers/handlers/resources
  record evidence/provenance/failure classifications
```

This prevents two bad extremes:

1. compile-time rows pretending to grant authority;
2. runtime admission hiding all requirements from type checking and module summaries.

### 4.4 Diagnostics

Diagnostics should name the failed discharge rule, not only the missing row item.

Examples:

```text
missing capability admission: cap fs.read
role operator is admitted, but does not entail cap fs.write
policy production_rate has no evaluator in this app instance
channel orders.in exists, but this process owns no receive endpoint
contract requires non_empty_path was neither proven nor assigned a dynamic check
effect group IO expands to cap fs.read, but groups do not grant authority
```

### 4.5 Still to resolve

1. Exact `AmbientEffectEnvironment` carrier in Core/typechecker/runtime APIs.
2. Which role/capability entailments are checked statically, dynamically, or both.
3. How policy decision domains are typed and matched against evaluators.
4. How resource split/join/borrow facts are represented.
5. How public module summaries expose open rows without exposing private aliases/groups.
6. How admission plans are serialized for runtime/app startup.
7. Whether profile lifting remains explicit forever or becomes inferred under narrow rules.

## 5. Decision Pass D: Process/Channel and Memory/Region Boundaries

### 5.1 Decision

Target Ash should treat process communication as an ownership boundary, not a shared-memory
shortcut.

Resolved direction for the first target slice:

1. **Processes are isolated by default.** A process owns its region, mailbox-visible state,
   handlers/providers installed for it, and process-local resources.
2. **Channel send crosses both a process boundary and a region boundary.** The sent payload
   must be moved, copied, shared through an explicit handle, serialized, or rejected.
3. **Owned sendable values move by default.** After a move send, the sender loses access and
   the receiver owns the value in its region.
4. **Copy is explicit and type/evidence-backed.** Copying requires a copyable value type or
   an explicit serialization/copy protocol.
5. **Sharing is not implicit.** Shared access must use an explicit immutable shared value,
   resource handle, provider handle, or app/runtime-managed object with a declared lifetime.
6. **Process-local and region-local values do not escape.** Raw local handles, borrowed
   views, non-serializable host objects, local graph interpreter state, and unsafe captured
   continuations are rejected at channel/app boundaries.
7. **Process termination releases the process region.** Retained diagnostics, reports,
   traces, shared handles, and supervisor state must copy or summarize what they need under a
   separate lifetime policy.
8. **Long-lived loops need subregions or equivalent discipline.** Per-message/request data
   should not be retained unless placed in explicit state/resource/report structures.

### 5.2 Boundary crossing modes

| Mode | Meaning | Required evidence | Sender after send | Receiver after receive |
|---|---|---|---|---|
| move | transfer ownership | `Send`/move-safe payload | loses access | owns value |
| copy | duplicate value | `Copy` or serialization/copy evidence | keeps original | owns duplicate |
| share | share controlled reference/handle | `Share` or resource/provider handle contract | keeps handle/reference under rules | receives handle/reference under rules |
| serialize | encode/decode through boundary format | `Serialize`/decode contract | keeps or moves source by protocol | owns decoded value |
| reject | payload cannot cross | `ProcessLocal`/`RegionLocal`/unsafe capture fact | no send occurs | no receive occurs |

The type names are provisional. They may become interfaces, marker traits, contracts, or
compiler-known predicates. The semantic distinction is required either way.

### 5.3 First-slice channel payload policy

First slice should prefer a conservative rule:

```text
if payload is owned and sendable:
  move
else if payload is explicitly copyable:
  copy
else if payload is an admitted shared/resource handle:
  share handle
else if boundary requires serialization and evidence exists:
  serialize
else:
  reject
```

Specific defaults:

| Payload shape | Default |
|---|---|
| plain ADT/record/tuple/list of sendable owned values | move |
| small scalar/copyable value | copy or move, implementation choice if semantics identical |
| closure | reject unless capture set is sendable and callable boundary is specified |
| continuation | reject across process boundary by default |
| multi-shot pure continuation | still reject across process boundary until region/capture rules are specified |
| process handle/channel endpoint | allow only if endpoint direction/lifetime permits it |
| provider/resource handle | share only through explicit resource/provider contract |
| borrowed view into process region | reject |
| host object/raw handle | reject unless wrapped in admitted resource handle |
| graph interpreter internal state | reject |

### 5.4 Region lifetime rules

The user-visible lifetime model should be:

```text
process region       -- released when process terminates
iteration subregion  -- released after loop/message/request step
state/resource       -- retained intentionally across steps
trace/report sink    -- retained by explicit bounded policy
shared provider/app  -- retained by admission/runtime policy
```

Important consequences:

- supervisor restart creates a fresh child process region;
- a failed child region must not stay alive through accidental closures/traces;
- app boundaries are also memory/authority/reporting boundaries;
- allocator strategy is implementation detail as long as these semantic lifetimes hold.

### 5.5 Interaction with handlers and continuations

Continuations are memory-bearing values. The process/channel boundary therefore depends on
continuation multiplicity and capture safety:

| Continuation case | Boundary decision |
|---|---|
| affine local resume | process-local by default; do not send |
| discarded resume | captured region may be released when unreachable |
| delayed resume in same process | allowed only if it does not outlive captured region facts |
| multi-shot pure resume | reusable in one process when row/capture rules permit; not automatically sendable |
| continuation capturing provider/resource/process-local state | reject across process/app boundary |

This keeps continuation multiplicity tied to memory retention, not only control flow.

### 5.6 Still to resolve

1. Final names and kind of `Send`, `Copy`, `Share`, `Serialize`, `ProcessLocal`,
   `RegionLocal`, and `ResourceHandle`.
2. Whether closure sendability is ever allowed and how capture rows/lifetimes are checked.
3. Whether any continuation can cross a process boundary under a future serialized/protocol
   representation.
4. Exact channel endpoint ownership, delegation, close, and direction rules.
5. Guard failure behavior: consume message, leave message, fail receive, or policy-selected.
6. Mailbox and buffer memory diagnostics.
7. Runtime allocator strategy and whether logical regions precede real arenas.
8. App-to-app sendability rules, especially for shared provider/resource handles.

## 6. Decision Pass E: App and Runtime-Kernel Boundary

### 6.1 Decision

Target Ash should make runtime admission explicit and keep definitions separate from running
instances.

Resolved direction for the first target slice:

1. **Definitions do not run.** Loading a module, workflow, service, provider, graph, or app
   declaration creates available artifacts, not running instances.
2. **The RuntimeKernel is the host container and control plane.** It owns loaded artifacts,
   provider/resource registries, schedulers, process tables, app/workflow instance tables,
   report sinks, and daemon control surfaces.
3. **An AppDefinition is a runtime blueprint.** It names roots, provider/resource
   requirements, child specs, graph specs, service endpoints, report policies, and admission
   profiles.
4. **An AppInstance is an admitted running app.** It has identity, app-local namespaces,
   app-local provider/resource admission, a root supervisor, process/graph/service
   instances, and trace/report sinks.
5. **A RuntimeKernel may host many AppInstances concurrently.** App instances are isolated by
   default even when they share the same kernel process or host provider pool.
6. **Provider/resource lifetime is not authority.** A runtime-global provider may exist for
   daemon efficiency, but an app/process/workflow may use it only through its admission
   context.
7. **Inter-app communication is explicit.** Typed channels, service handles, router grants,
   event topics, graph adapters, or host routing capabilities must be admitted deliberately.
8. **Host starts and Ash spawns are different boundaries.** `ash run`/daemon start admits a
   root app/workflow/process; Ash `spawn` creates a child under an already admitted runtime
   context.

### 6.2 Runtime layering

The target runtime layering should be:

```text
RuntimeKernel
  loaded artifacts and registries
  provider/resource pools
  control plane

AppInstance
  app identity and admission context
  root supervisor
  app-local process namespace
  app-local graph/service namespace
  app-local provider/resource grants
  report/trace policy

Process/Workflow/Service/Graph instance
  scheduled execution
  local region/state
  installed handlers/providers
  lifecycle events
```

This preserves the multi-app story from NOTE-016 while keeping one-shot CLI execution and
daemon execution semantically aligned.

### 6.3 Admission flow

Starting an app or root entry should follow this shape:

```text
load artifacts/config
resolve AppDefinition or one-shot entry
check row/profile/resource/policy requirements
construct AdmissionContext
install admitted provider/handler/resource facts
allocate AppInstanceId or root instance id
start root supervisor or root computation
record lifecycle/report metadata
```

Failure before `AdmissionContext` is established is rejection/configuration failure, not
ordinary program failure.

### 6.4 App isolation and sharing

Default app rule:

```text
same RuntimeKernel does not imply shared authority, memory, namespace, or reports.
```

Allowed sharing must name:

1. the shared carrier, such as channel, service handle, event topic, graph adapter, resource
   handle, provider pool, or report sink;
2. the admission authority that permits both sides to use it;
3. the payload/sendability and memory-retention rules;
4. the failure/reporting behavior if the shared carrier fails.

### 6.5 App definition surface

The canonical runtime representation should be `AppDefinition` regardless of authoring
surface.

Candidate authoring surfaces:

| Surface | Strength | Risk |
|---|---|---|
| source-level `app` declaration | language-native, type-checkable, easy to reference | adds new declaration surface |
| external manifest | good for deployment/operator config | splits language/runtime facts |
| exported Ash value | library-friendly and compositional | may blur declaration versus execution if not constrained |
| generated package metadata | convenient for tooling | can hide important admission facts |

Recommendation for the next pass: support one canonical `AppDefinition` data model first,
then decide whether source `app` syntax is required or whether manifests/exported values can
author it.

### 6.6 Still to resolve

1. Final authoring surface for `AppDefinition`.
2. Whether one-shot execution admits an ephemeral `AppInstance` or a smaller root-entry
   instance.
3. App namespace identity: globally unique, root-qualified, package-qualified, or
   daemon-assigned.
4. Shared provider pool metering, quotas, and failure propagation across app instances.
5. Inter-app service dependency and startup ordering.
6. Hot reload and migration of app instances, graph instances, and retained state.
7. Report/trace retention policy across app restarts and daemon lifetime.
8. OS/control-plane caller identity beyond first-slice same-user assumptions.

## 7. Cross-Cutting Boundary Contract

Every boundary should eventually answer the same minimum questions:

1. **Carrier:** what representation crosses the boundary?
2. **Ownership:** who owns it before and after crossing?
3. **Authority:** which row items or admission facts are required?
4. **Contracts:** which predicates, laws, or obligations apply?
5. **Failure:** which failure categories can arise?
6. **Evidence:** which discharge, trace, report, or provenance record is produced?
7. **Lifetime:** how long may the value, authority, resource, or evidence survive?
8. **Migration:** which legacy forms lower to this boundary?

## 8. Working Principle

The boundary rule:

```text
Nothing crosses a target Ash boundary by accident. The language, type checker, runtime,
or library must name the carrier, ownership rule, authority/evidence requirement, failure
classification, and lifetime policy.
```

## 9. References

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

## 10. Changelog

- 2026-06-24: Initial inventory. Lists target Ash boundaries and expands each with a
  description, affected features, design options, and references.
- 2026-06-24: Added first decision pass for the failure boundary, separating recoverable
  `fail`, traps, contract violations, authority/admission failures, policy denials, host
  adapter failures, process failure/cancellation, and workflow/app boundary reports.
- 2026-06-24: Added second decision pass for effect declaration and extern/host boundaries:
  `effect` is the target operation vocabulary, `capability` lowers to restricted
  authority-bearing effect operations, canonical operation identity lives below surface
  spelling, and raw externs remain trusted implementation hooks.
- 2026-06-24: Added third decision pass for row environment and admission boundaries:
  rows are requirement facts, ambient environments carry kind-specific discharge facts,
  admission is explicit at runtime boundaries, role entailment is discharge rather than row
  normalization, and aliases/groups never grant authority.
- 2026-06-24: Added fourth decision pass for process/channel and memory/region boundaries:
  channel sends cross ownership and region boundaries, owned sendable values move by default,
  copy/share/serialization require explicit evidence, process-local and region-local values
  are rejected, process termination releases its region, and long-lived loops need
  iteration-local retention discipline.
- 2026-06-24: Added fifth decision pass for app and runtime-kernel boundaries: definitions
  do not run, `AppDefinition` is a runtime blueprint, `AppInstance` is an admitted running
  app, one `RuntimeKernel` may host many isolated app instances, provider lifetime is not
  authority, inter-app communication requires explicit grants, and host starts are distinct
  from Ash process spawns.
