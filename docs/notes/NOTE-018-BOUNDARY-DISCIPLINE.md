# NOTE-018: Boundary Discipline for Target Ash

**Date:** 2026-06-24
**Status:** Living document — inventory in progress
**Purpose:** Define the target Ash boundary discipline: where values, authority, effects,
failures, evidence, memory, apps, providers, and host operations cross from one semantic
region to another. Companion to NOTE-015 (language forms), NOTE-016 (runtime organization),
NOTE-017 (memory regions), NOTE-013 (ambient monad and handler composition), and NOTE-014
(contract systems unification).

## 0. Motivation

The target Ash story is now centered on one ambient computation model with computation rows,
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
should not let current workflow, capability, or tower syntax define separate semantics.
Surface forms elaborate into Core terms, row entries, contract discharge metadata, and
sidecar evidence declarations.

**Affects:**

- `workflow`, `act`, `do:Act`, `do:Proc`, `do:Workflow`, `ret`, and workflow statements;
- `capability`, role, policy, resource, law, proof, property, and proposition declarations;
- row annotation syntax, inferred row summaries, and diagnostics;
- source spans and rewrite hints during stdlib/docs/test corpus migration.

**Options:**

1. Introduce canonical target syntax and migrate stdlib/docs/tests to it.
2. Treat old project-owned uses as corpus migration work, not a language compatibility
   contract.
3. Keep any domain-friendly surface forms permanently only if they earn target status, and
   require their lowering to be
   specified as sugar over Core.

**Current decision pass:** See §11.

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
2. Collapse current pure `Fn` and effectful `Fun` distinctions into the target
   row-bearing callable model while migrating project-owned corpus uses.
3. Add stricter closure-capture predicates for process-local, region-local, and authority
   carrying values.

**Current decision pass:** See §11.

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
   boundary obligations.
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

- `effect` as canonical authoring form and current capability syntax as migration input;
- canonical operation identity and row item spelling;
- operation argument/result contracts;
- trusted stdlib handler bodies that call `builtin(...)` for runtime-provided operations;
- provider and handler implementation shape;
- operation namespace export/import and versioning.

**Options:**

1. Make `effect` canonical and lower current `capability` syntax to restricted effect
   declarations during corpus migration.
2. Require one canonical Core/CPS operation identity regardless of legacy source spelling.

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

**Current decision pass:** See §8.

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

1. Keep properties outside the computation row as falsification metadata only.
2. Keep laws as evidence obligations discharged once per implementation.
3. Represent dynamic Hoare failures as traps by default, with explicit recoverable failure
   lowering where the surface chooses it.

**Current decision pass:** See §9.

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

**Current decision pass:** See §7.

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

**Current decision pass:** See §10.

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

**Current decision pass:** See §11.

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
3. Whether `with_error` remains a target library handler form or is removed during corpus
   migration.
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

1. **Canonical operation identity lives below surface spelling.** Current `capability`
   declarations migrate to `effect` declarations; Core/CPS sees one canonical operation
   identity and one row item.
2. **`effect` is the target vocabulary for operation declarations.** It names typed
   operation function signatures and their row contribution.
3. **`capability` is subsumed by `effect`.** The target language does not need separate
   capability declaration syntax. Current capability declarations are corpus migration
   work: rewrite them as effect declarations plus provider/admission metadata where needed.
4. **User-defined effects are a target-language direction, but an alpha staging choice.**
   The design vocabulary uses `effect` for ordinary algebraic-operation declarations such
   as failure, choice, and host operations. Current alpha specs may restrict which
   namespaces lower to Core/CPS until fully general user-defined resumable effects are
   specified across the effect, type, Core, and IR specs.
5. **Runtime builtins are calls from trusted handler bodies, not declarations.** Effect
   declarations define the operation interface. Trusted stdlib handlers implement those
   operations with ordinary `fn` methods whose bodies call `builtin(...)` using a typed
   runtime primitive symbol/key. `extern fn` remains out of scope for the current target
   language.
6. **Externs are out of scope for the current target language.** Ordinary Ash code calls typed
   operations. A later host/FFI design may add trusted implementation hooks, but raw extern
   declarations are not part of the current target surface.
7. **Provider installation is admission, not definition.** Declaring an effect does not
   install authority. Runtime admission installs a provider frame that may discharge the row
   item.

### 3.2 Canonical lowering shape

The target lowering story should be:

```text
surface declaration
  -> canonical operation identity
  -> row item required by callers
  -> provider implementation requirement
```

The plain target effect declaration shape is:

```ash
effect Fs {
    fn read(path: Path) -> String;
    fn write(path: Path, contents: String) -> Unit;
}
```

Effect operations use `fn` because they are callable function-shaped operations. They have a
special lowering role, but that does not make them non-functions. The relationship should
resemble interfaces and implementations: an effect declaration names callable signatures,
while providers supply interpretations.

Calling a resolved effect operation contributes the same resolved operation identity to the
row:

```ash
fn load(path: Path) -> {Fs.read} String {
    Fs.read(path)
}
```

Rows and call sites use the same name-resolution rules. The effect syntax does not specify
how names are imported, aliased, or canonicalized; that belongs to module loading,
resolution, symbol naming, and aliasing. Any resolvable operation name may be used:

```ash
fn load(path: Path) -> {fs.read} String {
    fs.read(path)
}
```

The two spellings above are equivalent only if ordinary name resolution maps `Fs.read` and
`fs.read` to the same operation identity in their respective scopes. After name resolution,
Core/CPS sees one canonical operation identity and one row item.

For an authority-bearing effect-like declaration:

```ash
effect Fs {
    fn read(path: Path) -> String;
}
```

Target meaning is the same canonical operation identity if the declaration is authority
bearing:

```text
EffectOp(fs.read) : (String) -> String
row {fs.read}
```

For a pure library algebraic effect such as choice, the canonical identity is still a
direct operation identity:

```text
EffectOp(choice.choose) : (List<A>) -> A
row {choice.choose}
```

The surface may be friendlier than the Core identity, but it must not create a second
operation namespace with different semantics. Authority-bearing operations are effect
operations interpreted by providers and discharged by admission; they do not need a
separate capability declaration form or a `cap` prefix on ordinary operation row items. The
syntax for introducing authority/admission facts, contracts, and extern hooks is
intentionally outside this plain effect-declaration slice.

### 3.3 Extern placement

Externs should be admitted only at trusted implementation boundaries:

| Placement | Use case | Visibility to ordinary Ash code |
|---|---|---|
| effect-level extern | canonical host ABI for a standard operation | hidden behind typed operation |
| provider-level extern | backend-specific adapter or deployment-specific host binding | hidden behind admitted provider |
| provider-owned extern | trusted interpreter for an effect operation | hidden behind provider installation |
| ordinary `extern fn` | bootstrap/trusted implementation only, not target user model | should not be directly callable as pure Ash |

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
provider says how it runs;
extern says how trusted implementation reaches the host.
```

### 3.5 Still to resolve

1. Exact target syntax for `effect` declarations.
2. The corpus migration path from current capability declarations to target effect
   declarations and provider/admission metadata.
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
   provider bindings, owned channel endpoints, resource ownership facts, policy providers,
   contract evidence, failure providers, and evidence sinks.
3. **Discharge is kind-specific.** Authority, role, policy, contract, resource, channel,
   process, failure, and evidence row items do not share one generic "handled effect" rule.
4. **Admission is explicit at runtime boundaries.** Starting a workflow/app/process or
   installing a provider can add discharge facts to the environment. Merely loading a
   declaration cannot.
5. **Role entailment is discharge, not row normalization.** A role may discharge authority
   needed for an operation when admitted, but the operation row item remains the direct
   operation identity for audit and diagnostics.
6. **Aliases and groups are not authority bundles.** They expand row spelling or improve
   diagnostics, but they do not discharge requirements.
7. **Open-row solving must not invent privilege.** A row variable can be constrained by use,
   expected type, or public summary shape; it must not absorb ambient authority just because
   authority happens to be available.

### 4.2 Discharge matrix

| Row item kind | Requirement means | Discharged by | Not discharged by |
|---|---|---|---|
| authority/admission | operation authority may be needed | provider row requirements discharged by admission, explicit authority fact, or admitted role entailment | row alias, declaration existence, global provider presence |
| resource | owned/borrowed resource access may be needed | ownership, borrow, split/join, provenance fact, admitted resource handle | authority for an operation alone |
| role | role-specific authority/context may be needed | role admission at workflow/app/process boundary | role declaration existence |
| policy | named policy decision may be needed | compatible named policy binding/evaluator/provider | anonymous boolean expression unless explicitly lowered |
| contract | predicate/law/obligation must be discharged | static proof, evidence, dynamic check strategy, law proof, runtime contract provider | property test alone, silent erasure |
| channel | endpoint operation may be needed | owned endpoint with compatible direction/message/guard facts | channel type name alone |
| process | runtime process operation may be needed | process-capable profile/runtime context | `Act` profile or plain function context |
| failure | recoverable failure route may be needed | failure-capable profile and provider/reporting policy | trap support alone |
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
  run under admitted providers/resources
  record evidence/provenance/failure classifications
```

This prevents two bad extremes:

1. compile-time rows pretending to grant authority;
2. runtime admission hiding all requirements from type checking and module summaries.

### 4.4 Diagnostics

Diagnostics should name the failed discharge rule, not only the missing row item.

Examples:

```text
missing authority admission for operation fs.read
role operator is admitted, but does not entail authority for fs.write
policy production_rate has no evaluator in this app instance
channel orders.in exists, but this process owns no receive endpoint
contract requires non_empty_path was neither proven nor assigned a dynamic check
effect group IO expands to fs.read, but groups do not grant authority
```

### 4.5 Still to resolve

1. Exact `AmbientEffectEnvironment` carrier in Core/typechecker/runtime APIs.
2. Which role/authority entailments are checked statically, dynamically, or both.
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

## 7. Decision Pass F: Behaviour and Service-Runner Boundary

### 7.1 Decision

Target Ash should treat behaviours as library protocols over process/channel/effect
operations, not as new runtime primitives.

Resolved direction for the first target slice:

1. **A behaviour interface defines callback shape.** It declares state, input, output,
   callback rows, contracts, and required evidence.
2. **A runner defines runtime meaning.** It starts a process, installs mailbox/endpoint
   handling, calls callbacks, manages state transitions, emits lifecycle events, and returns
   service handles.
3. **A supervisor defines lifecycle policy.** Start, restart, shutdown, escalation, and
   child ordering belong to supervisor specs and runner integration, not to the behaviour
   interface itself.
4. **Behaviour implementations are ordinary impl/evidence values.** Runners may receive
   explicit dictionaries/evidence and may be specialized, but no new object hierarchy is
   required.
5. **Callback contracts compose through the runner.** The runner is responsible for checking
   callback preconditions/postconditions at the correct message/state boundary.
6. **Service handles are explicit boundary carriers.** A started behaviour returns typed
   call/cast/stop/monitor handles whose sendability and authority are governed by the
   process/channel boundary.
7. **No target-core `behaviour` keyword is needed yet.** A later surface may add sugar, but
   it must lower to interface + impl + runner + child spec.

### 7.2 Behaviour layering

The target layering should be:

```text
interface Behaviour<...>
  callback signatures
  callback rows
  callback contracts
  evidence constraints

impl Behaviour for Impl
  callback bodies
  law/contract evidence

runner::start<Impl>(args, options)
  allocates process
  installs mailbox/protocol
  invokes callbacks
  owns loop semantics
  returns service handle

supervisor child spec
  chooses start arguments
  chooses restart/shutdown policy
  observes child lifecycle
```

### 7.3 Example boundary split

For a GenServer-like pattern:

| Piece | Owns |
|---|---|
| `interface GenServer<S, Req, Reply>` | callback type/row/contract shape |
| `impl GenServer for Counter` | user-specific state transition code |
| `gen_server::start<Counter>` | process creation, mailbox loop, request/reply protocol |
| `ServerHandle<Req, Reply>` | typed public communication boundary |
| supervisor child spec | restart/shutdown policy |
| app definition | provider/resource admission and startup placement |

This keeps "agent-like workflow" as composition:

```text
AgentLoop behaviour
  + model/tool effects
  + memory/resource handles
  + policy/contracts
  + runner
  + supervisor child spec
```

### 7.4 Still to resolve

1. Standard behaviour interfaces for `GenServer`, `Supervisor`, `Stage`, `Source`, `Sink`,
   `Router`, and `AgentLoop`.
2. Runner API shape: explicit dictionaries, type applications, module paths, or generated
   wrappers.
3. How callback row requirements are summarized into child specs and app admission.
4. How callback contract blame is assigned between caller, runner, and implementation.
5. Whether service handles are affine, shareable, serializable, or process-local by default.
6. Whether behaviour runners can be hot-reloaded and how state migration is typed.
7. Which lifecycle events are mandatory for traces/reports.

## 8. Decision Pass G: Handler and Provider Boundary

### 8.1 Decision

Target Ash should use `handler` as the primary surface term for operation interpreters.
This matches algebraic-effect terminology in other languages and keeps the syntax familiar.
`provider` is a synonym for `handler`, not a separate semantic category. Documentation and
examples should prefer `handler`; existing runtime- or admission-oriented text may still say
provider where that is the established name for a handler registry, pool, or adapter.

The important distinction is authority: some handlers are pure/library interpreters, while
others have authority requirements in their own rows and are installed by admission.

Resolved direction for the first target slice:

1. **Handlers interpret operation requirements.** A handler matches canonical operation
   identity, peels the matching row item from the handled body, and contributes its own row
   requirements to the residual computation.
2. **Handler order is operationally significant.** Effect rows are unordered requirement
   sets; handler nesting determines interpretation order. Commutation is a law/evidence fact,
   not a default assumption.
3. **Authority-bearing handlers expose authority in their own rows.** A handler that
   implements `fs.read` against the host can eliminate `{fs.read}` from the handled body,
   but its implementation contributes its own authority/admission/host/provenance row
   requirements.
4. **Pure/library handlers and authority-bearing handlers must be distinguished.** A pure
   nondeterminism handler and a filesystem handler both interpret operations, but only the
   authority-bearing handler may require admission and touch the host.
5. **Resume strategy is part of handler semantics.** Resume, discard, delay, or multi-shot
   reuse determines the meaning of sequencing for the interpreted operation.
6. **Continuation multiplicity constrains handler legality.** Affine resumes may be used at
   most once; multi-shot resumes require pure continuation rows and safe captures.
7. **Authority introduction is explicit.** Authority may be introduced by an admission
   environment or by explicit authority/resource values, then discharged through the same
   row-environment mechanisms as other requirements. A handler declaration by itself does
   not introduce authority.

### 8.2 Row transformation

The handler rule should keep the row-peeling intuition explicit:

```text
body row:
  {op, rest... | a}

handler matches:
  {op | r}

match:
  r := {rest... | a}

after handling:
  residual row = handler.row union r
```

Consequences:

- a handler removes only the operation it interprets;
- unmatched requirements remain in the residual row;
- handler-local requirements are added to the residual row;
- the row still does not encode handler order.

### 8.3 Handler classes

| Class | Example | Authority posture | Typical resume behavior |
|---|---|---|---|
| pure library handler | choice/all-results, option, validation | no host authority | discard, resume, or multi-shot under row rules |
| state/resource handler | state cell, transaction log | owns scoped state/resource | affine/deep resume |
| failure handler | `with_error`, branch-local failure drop | owns failure interpretation | discard or resume with default |
| scheduling handler | delayed resume/future/timer | owns scheduler interaction | delayed affine resume |
| host-backed handler | fs/http/model/tool handler | authority in handler row, may own host adapter | operation-specific |
| contract handler | dynamic Hoare check/reporting | owns dynamic check/report policy | usually trap/fail, not arbitrary resume |

### 8.4 Handler/provider authority and lifetime

Handler lifetime and handler authority are separate. The same rule applies when an existing
runtime text uses the synonym "provider":

```text
handler/provider loaded        != authority granted
handler/provider pool exists   != app may use it
handler/provider installed     == admitted discharge fact for a scope
```

Handler/provider scopes may be runtime-global, app-local, workflow-instance-local,
process-local, or operation-local. The scope controls lifetime and sharing; the admission
context controls who may use the handled operations.

### 8.5 Authority tracking through handlers

Authority should be tracked in the handler's own computation row, using the regular row
requirement and discharge machinery. The handled body names operations directly; the
handler implementation contributes whatever authority/admission/host/provenance
requirements are needed to interpret those operations.

Example shape:

```text
body row:      {fs.read}
handler row:   {authority fs.read, host.fs.read, records provenance}
result row:    {authority fs.read, host.fs.read, records provenance}
```

The handler eliminates the operation requirement from the body, but it does not erase its
own requirements. A pure handler can eliminate the same operation without authority:

```text
body row:      {fs.read}
handler row:   {}
result row:    {}
```

This lets a mock filesystem, replay log, sandbox, or proof-oriented interpreter handle
`fs.read` without host authority, while a host filesystem handler exposes its own
authority and host-boundary requirements.

Authority introduction remains deliberately general in this slice:

- an admission environment may provide authority facts for an app/process/workflow scope;
- explicit authority or resource values may be passed to a provider;
- roles or policies may discharge authority requirements when admitted;
- provider installation may make discharge facts available, but only under an admission
  context that entails the provider's own authority row.

This note does not yet specify whether authority facts are linear, affine, shared,
persistent, budgeted, or scoped. Those multiplicity/lifetime rules are a separate design
topic. Until then, authority should be treated as ordinary computation-row requirements
with no special syntax beyond the unresolved authority/admission fact spelling.

### 8.6 Handler surface styles

Target Ash should allow handler interpretation to be expressed without inventing a separate
semantic category for every style. Two surface families are useful:

1. **Explicit scoped installation.** A Koka-like surface makes the provider frame
   visible around a body. The exact spelling remains open, but the meaning is direct:
   install an interpreter for a scoped computation and lower to Core/CPS `Handle`.
2. **Function-local operation elimination.** A Frank-like surface keeps ordinary callable
   declarations and uses an `on` form inside the function body to eliminate an effectful
   computation produced by an ordinary thunk function.

Illustrative Frank-like shape:

```ash
effect Choice {
    fn choose<A>(xs: List<A>) -> A;
}

fn all_choices<A, r>(body: Unit -> {choice.choose | r} A) -> {r} List<A> {
    on body() {
        done(value) =>
            [value]

        choice.choose =>
            with |xs, resume| ->
                xs.flat_map(fn x -> all_choices(fn () -> resume(x)))
    }
}
```

The `done` clause name is illustrative. The important point is that normal completion of an
`on` scrutinee is not written with `return`: `return` is reserved for `do` syntax as the
ambient monad's unit. Direct-style function and provider bodies produce ordinary expression
results.

Use site with the Frank-like style:

```ash
fn pairs() -> {choice.choose} (Int, Int) {
    do {
        x <- choose([1, 2]);
        y <- choose([10, 20]);
        return (x, y)
    }
}

fn all_pairs() -> {} List<(Int, Int)> {
    all_choices(pairs)
}
```

The provider is installed by ordinary function application. The provider receives an
ordinary function value; it calls that thunk under `on` to wrap the produced computation. No
separate provider registry, ambient default, or app-level provider lookup is needed for this
case:

```text
pairs                  : Unit -> {choice.choose} (Int, Int)
all_choices(pairs)     : {} List<(Int, Int)>
```

Anonymous computations can be passed with ordinary anonymous functions:

```ash
fn all_pairs_inline() -> {} List<(Int, Int)> {
    all_choices(fn () -> do {
        x <- choose([1, 2]);
        y <- choose([10, 20]);
        return (x, y)
    })
}
```

A future or library-provided computation-thunking form such as `delay(do { ... })` may
elaborate to the same kind of thunk:

```ash
all_choices(delay(do {
    x <- choose([1, 2]);
    y <- choose([10, 20]);
    return (x, y)
}))
```

That elaboration belongs to evaluation-mode/computation-thunking syntax, not to
effect/handler syntax. Handler functions should not require special contextual delayed
argument rules.

The explicit scoped style should express the same installation more directly when a named
handler value or handler expression is easier to read. The exact surface spelling remains
open, but a Koka-like use site would have this shape:

```ash
handler AllChoices<A> {
    on body() {
        done(value) =>
            [value]

        choice.choose =>
            with |xs, resume| ->
                xs.flat_map(fn x -> AllChoices(fn () -> resume(x)))
    }
}

fn all_pairs_scoped() -> {} List<(Int, Int)> {
    with AllChoices {
        pairs
    }
}
```

The scoped form is also handler installation at the use site:

```text
pairs                         : Unit -> {choice.choose} (Int, Int)
with AllChoices { pairs }     : {} List<(Int, Int)>
```

Both examples are illustrative syntax. The semantic requirement is that the visible
handler installation determines which handler handles `choice.choose`; ordinary lexical
nesting/function application determines precedence when handlers are nested.

Both styles use the same row transformation. For a handler that handles `choice.choose`
and has handler-local row `p`:

```text
body row:    {choice.choose | r}
handler row: p
result row:  {r} union p
```

Frank-like application:

```ash
all_choices(
    fn () -> do {
        log("start");
        x <- choose([1, 2]);
        return x
    }
)
```

and explicit scoped installation:

```ash
with AllChoices {
    do {
        log("start");
        x <- choose([1, 2]);
        return x
    }
}
```

both have the same row behavior:

```text
body row:    {choice.choose, log.write}
handled op:  choice.choose
handler row for AllChoices: {}
result row:  {log.write}
```

If the handler has authority or host requirements, those requirements are added in both
styles:

```text
body row:    {fs.read}
handled op:  fs.read
handler row for HostFs: {authority fs.read, host.fs.read, records provenance}
result row:  {authority fs.read, host.fs.read, records provenance}
```

Thus the difference between the two styles is surface placement only:

```text
Frank-like style:
  handler is an ordinary callable receiving a thunk that produces an effectful computation.

Explicit scoped style:
  handler is visibly installed around a lexical computation body.

Both:
  lower to the same Core/CPS Handle;
  eliminate handled operation rows at the installation site;
  preserve unmatched residual rows;
  add handler-local row requirements.
```

Handler composition is nesting in both styles:

```ash
handler1(handler2(handler3(body)))
```

and:

```ash
with Handler1 {
    with Handler2 {
        with Handler3 {
            body
        }
    }
}
```

have the same composition shape after lowering. The default assumption is
non-commutativity:

```text
handler1(handler2(body)) != handler2(handler1(body))
```

unless the program carries explicit law/evidence that the relevant handler algebras
commute. Such evidence may later allow the compiler to reorder or optimize handlers, but
in the absence of evidence, handler order is semantically visible and preserved.

The important distinction from `match` is type-directed:

```text
match scrutinee { ... }
  scrutinee is a value;
  clauses are value/data-constructor patterns.

on computation { ... }
  scrutinee is an effectful computation/port;
  clauses are completion and operation cases;
  operation cases name an operation and define an operation-clause function.
```

`on` is deliberately match-like in shape, but it is not pattern matching over ordinary
values. It is an eliminator for computations with rows. The completion clause handles normal
completion; operation clauses handle canonical operation identities. The operation-clause
function receives the operation arguments plus a continuation for the rest of the
computation.

The operation arm shape is:

```text
operation_name =>
    with |operation_args..., resume| -> body
```

For example:

```ash
choice.choose =>
    with |xs, resume| ->
        xs.flat_map(fn x -> all_choices(fn () -> resume(x)))
```

This separates the operation identity being handled from the function that implements the
operation clause. The `with |...| -> ...` form is not an ordinary provider installation and
does not call the operation. It defines the operation-clause function that the provider
runtime applies to the raised operation arguments and resume continuation.

The two `with` contexts are distinct:

```text
expression position:
  with Provider { body }
  -- install a provider around a computation

on-arm right-hand side:
  operation.name => with |args..., resume| -> body
  -- define the operation-clause function for one operation
```

Operation clause parameters are binders, not arbitrary patterns or expressions. Dispatch is
by resolved operation identity. If a provider needs to inspect an operation argument, it
binds the argument and uses ordinary `match` or `if` inside the clause body.

The declaration remains an ordinary `fn`. An optional `operator` spelling may be admitted
as an intent-signaling synonym for functions that primarily bind or control effectful
ports, but it must not introduce a new semantic category:

```ash
operator all_choices<A, r>(body: Unit -> {choice.choose | r} A) -> {r} List<A> {
    on body() { ... }
}
```

elaborates like:

```text
fn all_choices<A, r>(body: Unit -> {choice.choose | r} A) -> {r} List<A> { ... }
```

Core lowering is shared across styles:

```text
explicit scoped handler
Frank-like fn/operator with on
  -> Core/CPS Handle with completion clause, operation clauses, resume metadata, and residual row
```

This keeps the useful Frank property that the callable type, handled computation parameter,
completion case, operation cases, resume types, and recursive calls are checked together. A
separate packaging function around a separately declared handler is allowed as ordinary
library style, but it is not the only Frank-like authoring path.

### 8.7 Still to resolve

1. Exact surface syntax for explicit scoped handlers and handler installation.
2. Whether handlers lower to ordinary Core/CPS handler frames, a separate handler-chain
   carrier, or a unified handler chain with trusted metadata.
3. How handler chains are represented in Core summaries versus CPS/runtime state.
4. Which operations support delayed resume and how scheduler interaction is typed.
5. How handler laws are expressed as evidence, especially commutation and state/failure
   interaction laws. The default is non-commutative handler composition for both
   Frank-like nested calls and explicit scoped installation.
6. How handler authority provenance is recorded and exposed in reports.
7. How deep versus shallow handlers are spelled, if both are admitted.
8. Exact grammar for `on`, including completion clause spelling and the final spelling of
   operation arms. Current direction is `operation.name => with |args..., resume| -> body`.
   The baseline provider argument is an ordinary thunk function such as `Unit -> {r} A`;
   convenience forms such as `delay(do { ... })` belong to evaluation-mode/computation-
   thunking syntax.
9. Whether `operator` is accepted as a permanent synonym for `fn`, a lint/documentation
   convention, or only a future readability affordance.
10. Authority fact spelling and discharge APIs, separate from later multiplicity/lifetime
    questions such as linear, affine, shared, scoped, persistent, or budgeted authority.

## 9. Decision Pass H: Contract and Evidence Boundary

### 9.1 Decision

Target Ash should unify contracts through shared discharge/evidence machinery without
collapsing their logical lifecycles.

Resolved direction for the first target slice:

1. **Hoare contracts are site-specific.** `requires`, `ensures`, invariants, and guards
   attach to computation or communication boundaries and compose through sequencing.
2. **Laws are universal obligations.** A law attaches to an interface/module/impl boundary
   and is discharged once per implementation or exported evidence unit, not per invocation.
3. **Properties are falsification metadata, not discharge.** A property can generate tests,
   reports, confidence, and counterexamples, but it does not prove a law or discharge a hard
   row obligation by itself.
4. **No contract disappears silently.** Static proof, evidence proof, demotion to dynamic
   check, or rejection must be recorded as discharge/evidence metadata.
5. **Dynamic Hoare failure defaults to structured trap.** It may lower to recoverable `fail`
   only when the surface protocol explicitly chooses recovery and row-accounts that failure.
6. **Blame is part of the boundary.** Runtime contract failures must preserve enough
   metadata to assign caller/callee/provider/runner responsibility once blame rules are
   specified.
7. **Evidence can justify optimization only under explicit trust rules.** Law/proof evidence
   can enable specialization or contract elision; property results alone cannot.

### 9.2 Lifecycle matrix

| Contract/evidence form | Attachment site | Discharge timing | Runtime posture | Evidence/reporting |
|---|---|---|---|---|
| `requires` | function/callback/operation entry | per call or statically proven | check before body or trap/fail | predicate, arguments, discharge mode |
| `ensures` | function/callback/operation exit | per successful return or statically proven | check after result or trap/fail | result, old values if needed, discharge mode |
| invariant | loop/data/resource/graph boundary | per boundary step or statically proven | check at declared boundary | state snapshot and boundary id |
| guard | channel receive/send/select boundary | per communication event | check before admission/receive behavior | message binder, endpoint, guard result |
| obligation | workflow/app/governance boundary | admission, execution, or closeout | boundary-specific | obligation id and lifecycle event |
| law | interface/module/impl boundary | once per implementation/evidence unit | erased after discharge | proof/test/symbolic evidence ref |
| proof | evidence declaration | compile/load/check time | not ordinary runtime code | proof artifact and trust mode |
| property | test/falsification harness | test time | no runtime row item | generated cases, failures, confidence |

### 9.3 Discharge outcomes

Every hard contract obligation should end in exactly one recorded outcome:

```text
Static      -- proven by checker/SMT/refinement
Evidence    -- discharged by proof/law evidence/test-evidence mode where trusted
Dynamic     -- runtime check inserted with discharge metadata
Rejected    -- counterexample, missing evidence, invalid predicate, or policy says no demotion
Deferred    -- accepted only where project/build policy explicitly allows unresolved evidence
```

Dynamic checks are not silent fallbacks. If a static proof is unknown and the compiler
demotes to dynamic checking, the discharge record should say so.

### 9.4 Boundary interactions

Contracts interact with other boundaries as follows:

- **Function boundary:** `requires` and `ensures` define caller/callee obligations.
- **Provider boundary:** operation contracts define caller input obligations and provider
  output obligations.
- **Process/channel boundary:** guards and payload contracts define message admission and
  potential receive behavior.
- **Behaviour runner boundary:** callback contracts are checked at runner-owned
  message/state boundaries.
- **App/workflow boundary:** obligations, reports, policies, and closeout contracts are
  interpreted as governance requirements.
- **Module summary boundary:** discharged law/proof evidence must be exportable,
  versioned, and invalidated when dependencies change.

### 9.5 Still to resolve

1. Caller/callee/provider/runner blame rules.
2. Interface-to-impl precondition/postcondition variance and subsumption.
3. Monadic Hoare logic through `bind`, handlers, and row-polymorphic sequencing.
4. Temporal/liveness contract vocabulary for processes, workflows, streams, and graphs.
5. Interaction with lazy/memo timing and force-site contract checks.
6. Evidence serialization, trust modes, and cross-module/package cache invalidation.
7. Property-to-law workflow, if any, and whether tested evidence can ever satisfy a law under
   a named trust policy.
8. Redaction policy for values captured in runtime contract diagnostics.

## 10. Decision Pass I: Reactive Stream and Graph Boundary

### 10.1 Decision

Target Ash should keep pull streams, push events, and declarative graphs as distinct
reactive modes with explicit adapters.

Resolved direction for the first target slice:

1. **Pull is codata/machine-oriented.** A pull producer is demanded by a consumer through a
   `next`-like protocol and naturally supports backpressure.
2. **Push is operational.** Push events use channels, emit/subscribe effects, mailboxes, or
   callbacks; buffering, dropping, replay, ordering, and fairness are runtime policies.
3. **Graphs are declarations/data.** A graph blueprint does not run until an app/supervisor
   starts a graph interpreter instance.
4. **Bridge adapters are explicit.** `Producer -> channel publisher`, `channel subscriber ->
   Producer`, `pull source -> graph input`, and `graph output -> event sink` must name their
   buffering/backpressure/retention semantics.
5. **Retention is declared, not inferred as unbounded history.** Stream steps, graph ticks,
   state cells, memo caches, replay logs, and traces must have explicit lifetime/bounding
   policy.
6. **Reactive modes are not workflow primitives.** Workflows may govern or start them, but
   stream and graph semantics live in libraries/runners/interpreters over process/effect
   boundaries.

### 10.2 Mode split

| Mode | Primary shape | Execution owner | Memory posture | Failure posture |
|---|---|---|---|---|
| pull stream | `Producer<A>`, `Pipe<A, B>`, `Machine<I, O>` | consumer demand or process-backed source | step/iteration subregion plus retained producer state | `next` result/failure protocol |
| push event | channel, `Emit<T>`, `Subscribe<T>`, mailbox | producer/runtime scheduler | explicit buffer/drop/backpressure/spill policy | send/receive/failure policy |
| graph | `GraphDefinition` + `GraphInterpreter` | app/supervisor-started interpreter | declared state cells, windows, caches, history | graph instance failure/restart/report policy |

### 10.3 Bridge adapters

Reactive bridge adapters should be ordinary library/runtime components with explicit
contracts:

| Adapter | Must declare |
|---|---|
| pull producer to channel publisher | demand policy, channel capacity, send failure, cancellation |
| channel subscriber to pull producer | receive mode, timeout, buffering, message retention |
| push source to graph input | queue/drop/backpressure policy, clock/tick behavior |
| graph output to sink | delivery guarantee, replay, failure handling |
| pull source to graph input | sampling policy, tick source, stale value behavior |
| graph to pull producer | snapshot/history policy and graph instance lifetime |

No adapter should silently introduce unbounded buffering.

### 10.4 Graph interpreter boundary

A graph interpreter instance should name:

```text
GraphDefinition id
input/output ports
clock/scheduler policy
state cell retention
history/replay policy
provider/handler environment
failure/restart policy
trace/report policy
```

Open graph semantics such as glitch freedom, logical time, incremental recomputation, and
hot reload belong to graph interpreter contracts, not ordinary function evaluation.

### 10.5 Still to resolve

1. Standard library shapes for `Producer`, `Consumer`, `Pipe`, `Machine`, `Signal`, and
   graph definitions.
2. Whether `observe` remains a comonadic library/sugar form, becomes graph/stream-specific,
   or is retired from core workflow vocabulary.
3. Exact push buffer policy vocabulary and defaults.
4. Clock model: host time, logical clocks, discrete ticks, or hybrid.
5. Glitch-freedom and incremental recomputation guarantees for graph interpreters.
6. Hot reload/migration for graph state cells and stream producers.
7. How reactive retention contracts appear in app admission and memory diagnostics.

## 11. Decision Pass J: Compiler-Facing Boundaries

### 11.1 Decision

Target Ash should make the compiler-facing boundaries explicit:

```text
surface syntax      -> elaborated Core
function/closure    -> row-bearing callable plus checked captures
module/package      -> versioned public semantic summaries
```

Resolved direction for the first target slice:

1. **Core owns semantics, surface owns ergonomics.** Current workflow/tower/capability
   syntax should not be preserved as a language compatibility layer. Any old project-owned
   uses should be migrated to target surface forms that elaborate to Core constructs, row
   facts, discharge metadata, and sidecar evidence.
2. **Core type checking verifies elaborated facts.** Core checking does not own parsing,
   desugaring, type-class search, proof search, or arbitrary syntax migration. Those happen
   before Core or in separate verifier/discharge phases.
3. **Every callable is row-bearing at the semantic boundary.** Pure functions are the empty
   row case; current `Fn`/`Fun` distinctions should converge on row-bearing function or
   continuation types.
4. **Closure capture is checked as a boundary.** Captured values must be legal for the
   closure's row/profile and for any later boundary the closure may cross.
5. **Process-local, region-local, handler-chain, provider, and continuation captures are not
   ordinary values.** They require explicit capture/sendability rules and are rejected where
   rules are absent.
6. **Module summaries export facts, not authority.** Public summaries carry canonical type,
   row, effect, law/evidence, and declaration identities needed by importers, but never grant
   runtime authority.
7. **Aliases/groups remain transparent or diagnostic.** They may be exported for readability
   and diagnostics, but importers must resolve to canonical row/effect identities before
   checking and admission.
8. **Versioning and invalidation are part of the boundary.** Public row summaries,
   effect identities, law/proof evidence, and type-computation summaries must invalidate when
   their dependencies change.

### 11.2 Surface-to-Core lowering contract

Surface lowering should produce:

| Surface feature | Lowered/recorded target |
|---|---|
| `fn`, lambdas, calls | Core functions/calls with row-bearing callable types |
| `do`, current `act`, current typed `do:*` | migrate to target ambient sequencing plus row/profile checks |
| current `workflow` and workflow statements | migrate to governed function/app/runtime metadata plus Core body |
| current `capability` declarations / target `effect` declarations | canonical operation identities and row items |
| `handle`/provider scopes | Core/CPS handler/provider metadata |
| contracts | predicate refs plus discharge obligations/metadata |
| laws/proofs/properties | sidecar evidence/test metadata, not ordinary expressions |
| comprehensions/observe/co-comprehensions | library calls or reactive/comonadic sugar |

Core should not preserve surface-only constructs as semantic islands.

### 11.3 Function and closure boundary

Callable semantics should track:

```text
parameters
result
latent row
contracts
capture set
capture legality
public summary identity
```

Closure capture rules should classify captured values at least as:

| Capture kind | Default posture |
|---|---|
| pure data | allowed in pure/effectful closures |
| value produced by effect | allowed only when closure row/profile admits it |
| capability/provider/resource handle | allowed only with explicit authority/lifetime facts |
| process-local/region-local value | rejected across process/app boundaries |
| continuation | affine/process-local by default; stricter rules for multi-shot pure |
| handler/provider chain | trusted/runtime capture, not ordinary user data |

The target rule is not "closures are forbidden in pure code"; it is "captures must not leak
effects, authority, memory, or control beyond their permitted boundary."

### 11.4 Module/package summary boundary

Public summaries should export enough for downstream checking without exposing private
implementation facts or granting authority:

```text
public type and constructor identities
public callable signatures and row summaries
public effect operation identities
public aliases/groups as transparent/diagnostic facts
public law/proof evidence references
public type-level computation summaries
visibility/opacity/version metadata
dependency invalidation keys
```

Imported summaries should be registered before dependent checking so import order does not
change row/type/evidence results.

### 11.5 Still to resolve

1. Exact target replacement for every current workflow statement still used in
   stdlib/docs/tests.
2. Final user-facing syntax for row variables, effect declarations, handlers, and app specs.
3. Corpus migration schedule for `workflow`, `act`, `ret`, `do:Act`, `do:Proc`, and
   `do:Workflow` usages in stdlib/docs/tests.
4. Closure capture representation in Core summaries and diagnostics.
5. Whether callable public summaries expose inferred rows by default or only stable
   annotated rows.
6. Versioning format for effect identities, row aliases/groups, evidence refs, and operation
   contracts.
7. Cross-package evidence cache trust and invalidation policy.
8. Diagnostic or tooling rewrite hints for migrating project-owned corpus forms to target
   forms.

## 12. Cross-Cutting Boundary Contract

Every boundary should eventually answer the same minimum questions:

1. **Carrier:** what representation crosses the boundary?
2. **Ownership:** who owns it before and after crossing?
3. **Authority:** which row items or admission facts are required?
4. **Contracts:** which predicates, laws, or obligations apply?
5. **Failure:** which failure categories can arise?
6. **Evidence:** which discharge, trace, report, or provenance record is produced?
7. **Lifetime:** how long may the value, authority, resource, or evidence survive?
8. **Migration:** which current corpus forms should be rewritten to this boundary?

## 13. Working Principle

The boundary rule:

```text
Nothing crosses a target Ash boundary by accident. The language, type checker, runtime,
or library must name the carrier, ownership rule, authority/evidence requirement, failure
classification, and lifetime policy.
```

## 14. References

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

## 15. Changelog

- 2026-06-25: Added handler surface-style guidance: explicit scoped handlers and
  Frank-like ordinary `fn`/optional `operator` definitions with an `on` computation
  eliminator both lower to the same Core/CPS handler machinery.
- 2026-06-25: Clarified current-capabilities-as-effects direction: providers are the general
  operation interpreters, authority is tracked in the provider's own row and introduced or
  discharged through ordinary admission/environment mechanisms, while authority
  multiplicity/lifetime remains a separate future design topic.
- 2026-06-24: Initial inventory. Lists target Ash boundaries and expands each with a
  description, affected features, design options, and references.
- 2026-06-24: Added first decision pass for the failure boundary, separating recoverable
  `fail`, traps, contract violations, authority/admission failures, policy denials, host
  adapter failures, process failure/cancellation, and workflow/app boundary reports.
- 2026-06-24: Added second decision pass for effect declaration and extern/host boundaries:
  `effect` is the target operation vocabulary, current `capability` syntax is migration
  input to effect operations, canonical operation identity lives below surface spelling, and
  raw externs remain trusted implementation hooks.
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
- 2026-06-24: Added sixth decision pass for behaviour and service-runner boundaries:
  behaviour interfaces define callback shape, runners define runtime loop semantics,
  supervisors define lifecycle policy, service handles are explicit carriers, and no target
  core behaviour primitive is needed yet.
- 2026-06-24: Added seventh decision pass for handler and provider boundaries: handlers
  interpret canonical operations by row peeling, handler order is operationally significant,
  providers are trusted/admitted handler frames for runtime-backed operations, resume strategy
  and continuation multiplicity constrain legality, and provider installation is admission
  rather than declaration.
- 2026-06-24: Added eighth decision pass for contract and evidence boundaries: Hoare
  contracts are site-specific, laws are universal obligations discharged once per evidence
  unit, properties remain falsification metadata, hard contracts record discharge outcomes,
  dynamic Hoare failures trap by default unless explicitly recoverable, and evidence can
  justify optimization only under explicit trust rules.
- 2026-06-24: Added ninth decision pass for reactive stream and graph boundaries: pull is
  codata/machine-oriented, push is operational and requires explicit buffering policy,
  graphs are declarations interpreted by app/supervisor-started runners, bridge adapters are
  explicit, and retention must be declared rather than inferred as unbounded history.
- 2026-06-24: Added tenth decision pass for compiler-facing boundaries: surface syntax
  elaborates to Core rather than defining semantic islands, every callable is row-bearing at
  the semantic boundary, closure captures are checked for effect/authority/memory/control
  leakage, and module summaries export canonical facts without granting authority.
- 2026-06-27: Normalized target-row wording from effect rows to computation rows and row
  entries at the boundary inventory level.
