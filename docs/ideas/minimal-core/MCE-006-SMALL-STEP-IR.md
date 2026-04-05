---
status: drafting
created: 2026-03-30
last-revised: 2026-04-05
related-plan-tasks: [TASK-396, TASK-401, TASK-402, TASK-403, TASK-404]
tags: [small-step, ir, execution, alignment, interpreter]
---

# MCE-006: Align Small-Step Semantics with IR Execution

## Status Summary

MCE-006 is no longer blocked on an undefined small-step target.

Phase 61 fixed the upstream semantic backbone in [MCE-005](MCE-005-SMALL-STEP.md):

- workflow-first configurations are canonical;
- ambient context is `(C, P)`;
- dynamic state is expressed in `Γ`, `Ω`, `π`, cumulative trace, cumulative effect summary, and residual workflow terms;
- the main relation is `A ⊢ κ —μ→ κ'`;
- observables are split across configuration state and step labels;
- pure expressions/patterns remain atomic in v1;
- blocked/suspended configurations are distinct from stuckness;
- `Par` uses interleaving plus helper-backed terminal aggregation.

What remains open here is the runtime/interpreter realization of that accepted backbone.

Phase 63 / [TASK-401](../../plan/tasks/TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md) now adds the first frozen runtime-correspondence baseline: one canonical semantic-carrier → runtime mapping table grounded in current interpreter evidence.

## Problem Statement

Small-step semantics must correspond to executable IR evaluation without collapsing the semantic/runtime boundary.

The problem is now concrete:

- MCE-005 defines the semantic contract;
- MCE-006 must explain how the interpreter/runtime realizes that contract;
- any mismatch must be described as an implementation mapping issue, not as lingering ambiguity in the semantic backbone.

## Scope

In scope:

- mapping the accepted MCE-005 configurations onto interpreter/runtime structures;
- identifying where the current runtime already matches the backbone and where it diverges;
- defining the abstract-machine or equivalent execution view implied by the runtime;
- validating that runtime observables preserve the small-step and `SPEC-004` contracts.

Out of scope:

- revising the semantic backbone fixed by MCE-005;
- inventing new surface syntax or new canonical workflow forms;
- backend/JIT/distributed execution design.

Related but separate:

- [MCE-005](MCE-005-SMALL-STEP.md): accepted semantic backbone
- [MCE-007](MCE-007-FULL-ALIGNMENT.md): full five-layer closeout
- [MCE-008](MCE-008-RUNTIME-CLEANUP.md): adjacent runtime inventory/cleanup work

## Accepted Upstream Contract Consumed from MCE-005

The following are now fixed inputs for this exploration.

### 1. Configuration vocabulary

MCE-006 must map some runtime representation to the semantic carriers:

- ambient authority/policy context `A = (C, P)`;
- environment `Γ`;
- obligation state `Ω`;
- provenance state `π`;
- cumulative trace `T`;
- cumulative effect summary `ε̂`;
- residual canonical workflow or terminal result state.

A runtime may represent these indirectly, compactly, or across multiple data structures, but the mapping must preserve their observable meaning.

### 2. Step granularity

The runtime target is workflow-first, not expression-first.

So MCE-006 should assume:

- small-step transitions are anchored at workflow boundaries;
- pure expressions and patterns stay atomic in v1;
- helper-backed actions such as receive selection, policy checks, provenance joins, and parallel aggregation are semantic boundaries, not accidental implementation details.

### 3. Observable contract

MCE-006 must preserve the accepted split:

- configuration state is authoritative for cumulative obligations, provenance, traces, effects, and terminal outcome class;
- labels carry step-local trace/effect deltas.

An implementation may not silently discard that distinction if doing so would break trace/effect/provenance reconstruction or make blocking behavior unobservable.

### 4. Blocking contract

A runtime realization must distinguish:

- terminal completion;
- progress transitions;
- blocked/suspended waiting states;
- invalid/stuck states that should instead be owned by a declared failure boundary.

This matters especially for `Receive` and for runtime-owned control/completion interactions.

### 5. Concurrency contract

`Par` remains semantically interleaving-based with helper-backed terminal aggregation.

MCE-006 therefore owns the operational question:

- how does the runtime choose the next branch to step,
- and how does it encode/helper-call the combined terminal outcome,

without reinterpreting the semantic contract as simple left-to-right sequential execution.

## Remaining Runtime/Interpreter Alignment Questions

The open questions are now implementation-facing.

### 1. Residual workflow representation

How does the runtime represent the residual workflow component of `κ`?

Possible realizations include:

- direct residual AST values;
- explicit continuation frames;
- a hybrid control stack plus current node pointer.

This question belongs here, not in MCE-005.

### 2. Branch-local state for `Par`

The semantic model allows interleaved branch progress plus helper-backed aggregation. The runtime must decide:

- whether branch state is stored as separate workflow instances, frames, tasks, or scheduler entries;
- where branch-local traces/effects/obligations/provenance accumulate before aggregation;
- how helper-owned concurrent combination is surfaced.

### 3. Blocked-state carrier

The runtime needs an explicit representation for blocked/suspended states, especially for:

- blocking `Receive`;
- mailbox/control waits;
- runtime-owned completion observation boundaries.

This may be a scheduler state, queue registration, parked task record, or equivalent.

### 4. Effect and trace accumulation

How are the cumulative `T` and `ε̂` carriers realized operationally?

Key alignment questions:

- append-only trace log vs incremental event sink;
- per-branch effect summaries vs shared accumulator with rollback-free updates;
- projection of runtime records back into `SPEC-004` terminal outcome fields.

### 5. Provenance and obligation mutation boundaries

The semantic model speaks in terms of state transitions over `Ω` and `π`. MCE-006 must explain:

- where those state carriers live at runtime;
- when updates are committed;
- how helper-owned joins/discharges map to concrete operations.

### 6. ControlLink and completion realization

`SPEC-004` already fixes spawned-child completion semantics. MCE-006 must map that contract to runtime structures such as:

- control handles;
- completion tombstones or sealed payload records;
- supervisor-owned observation channels.

This remains runtime/helper work, not a new user-level syntax design.

## Runtime Execution Model Classification

Current interpreter evidence supports classifying the runtime as a hybrid control representation.

Why this is the best fit:

- the main execution path is direct recursive evaluation over residual `Workflow` ASTs via `execute_workflow_with_behaviour_in_state(...)` and `execute_workflow_inner(...)` in [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs);
- there is no first-class explicit continuation/frame machine for ordinary sequencing, branching, or binding;
- but blocked and externally resumed control paths are not carried only by the current AST node: `Receive` waiting lives in mailbox/polling loops, while yield/proxy suspension infrastructure stores resumable continuations in `YieldState.continuation`; adjacent routing structures such as `PendingYield.continuation` in [`../../../crates/ash-interp/src/yield_routing.rs`](../../../crates/ash-interp/src/yield_routing.rs) should be treated as nearby runtime infrastructure rather than as demonstrated main-path carriers until later tasks pin that integration down;
- shared runtime-owned registries (`ControlLinkRegistry`, `ProxyRegistry`, `SuspendedYields`) clearly hold part of the effective machine state outside the residual workflow term itself; `YieldRouter` exists as adjacent routing infrastructure but is not yet evidenced as a uniformly consulted carrier on every main execution path.

So the current interpreter is not a pure continuation machine, but it is also not a pure direct-residual-AST realization once blocking/suspension/control supervision are included.

## Canonical Semantic-Carrier → Runtime Mapping Table

This section is the Phase 63 canonical baseline artifact for later MCE-006 tasks.

| Semantic carrier | Semantic meaning | Current runtime holder(s) | Mapping quality | Update boundary | Observability role | Concrete evidence sources |
|---|---|---|---|---|---|---|
| `A = (C, P)` | Ambient capability/policy context inherited by steps | Primary in-execution carriers are `CapabilityContext`, `PolicyEvaluator`, per-operation `CapabilityPolicyEvaluator`, and role-scoped authority in `Context.role_context`; `RuntimeState.providers` is a provisioning/derivation source that can materialize capability context rather than a uniformly in-flight execution carrier | distributed | Providers may be installed through `RuntimeState::with_provider` and materialized by `RuntimeState::create_capability_context`; policy checks happen at `Workflow::Decide`, `execute_set`, `execute_send`, and receive-policy enforcement | Both: cumulative ambient authority/policy state, plus local decision sites | [`../../../crates/ash-interp/src/runtime_state.rs`](../../../crates/ash-interp/src/runtime_state.rs) (`RuntimeState`, `create_capability_context`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`execute_workflow_with_behaviour_in_state`, `Workflow::Decide`, `active_actor`); [`../../../crates/ash-interp/src/execute_stream.rs`](../../../crates/ash-interp/src/execute_stream.rs) (`execute_core_receive`, `enforce_receive_policy_for_sources`); [`../../../crates/ash-interp/src/context.rs`](../../../crates/ash-interp/src/context.rs) (`role_context`) |
| `Γ` | Runtime environment of current bindings | `Context.bindings` plus nested `parent` chain | direct | Mutated at workflow boundaries that bind values: `Context::set`, `set_many`, and `extend`; threaded recursively through `execute_workflow_inner` | Cumulative current environment for the active residual workflow | [`../../../crates/ash-interp/src/context.rs`](../../../crates/ash-interp/src/context.rs) (`Context`, `extend`, `set_many`, `get`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Workflow::Let`, `Workflow::Observe`, `Workflow::ForEach`, `Workflow::Split`) |
| `Ω` | Current obligation state | Distributed across `Context.obligations` and role-specific `RoleContext` (`active_role.obligations` + discharged set) | distributed | Generic obligation discharge happens through `Context::add_obligation` / `discharge_obligation`; role obligation scope is installed at `Workflow::Oblig` and checked via `Workflow::Check`; role discharge bookkeeping lives in `RoleContext::discharge` | Cumulative state, but only partially surfaced and split by obligation mechanism | [`../../../crates/ash-interp/src/context.rs`](../../../crates/ash-interp/src/context.rs) (`obligations`, `add_obligation`, `discharge_obligation`); [`../../../crates/ash-interp/src/role_context.rs`](../../../crates/ash-interp/src/role_context.rs) (`RoleContext`, `discharge`, `pending_obligations`, `all_discharged`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Workflow::Oblig`, `Workflow::Check`); [`../../../crates/ash-interp/src/eval.rs`](../../../crates/ash-interp/src/eval.rs) (`Expr::CheckObligation`) |
| `π` | Current provenance state | Only partially represented by `ash_core::Provenance` payloads on IR/value-side structures; not threaded as a mutable execution carrier | missing | `Provenance::new` / `fork` allocate lineage metadata, but the main interpreter entry points do not thread a live provenance accumulator through execution | Mostly latent metadata today; not an authoritative runtime cumulative carrier | [`../../../crates/ash-core/src/provenance.rs`](../../../crates/ash-core/src/provenance.rs) (`Provenance`, `fork`, `TraceEvent`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Workflow::Act { provenance: _ }` ignores runtime mutation); [`../../../crates/ash-interp/src/capability_provenance.rs`](../../../crates/ash-interp/src/capability_provenance.rs) (`CapabilityProvenanceTracker` exists but is not threaded through `execute_workflow_inner`) |
| `T` | Cumulative execution trace prefix | No authoritative trace carrier is threaded through workflow execution; closest adjacent structure is `CapabilityProvenanceTracker.events` plus core `TraceEvent` definitions | missing | Trace-like events can be recorded by utility code, but no `execute_workflow_*` entry point owns an append-only cumulative trace structure | Intended cumulative state is currently reconstructible only partially and out-of-band | [`../../../crates/ash-core/src/provenance.rs`](../../../crates/ash-core/src/provenance.rs) (`TraceEvent`); [`../../../crates/ash-interp/src/capability_provenance.rs`](../../../crates/ash-interp/src/capability_provenance.rs) (`CapabilityProvenanceTracker`, `record`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (returns `ExecResult<Value>` only, with no trace accumulator) |
| `ε̂` | Cumulative effect-summary accumulator | No shared runtime effect-summary accumulator; only local effect metadata on capabilities/providers and operation-specific checks | missing | Effect classification appears at provider/capability boundaries (`CapabilityProvider::effect`, capability policy checks), but there is no cumulative interpreter-owned summary updated per step | Local delta source only; no authoritative cumulative carrier | [`../../../crates/ash-interp/src/runtime_state.rs`](../../../crates/ash-interp/src/runtime_state.rs) (`providers` registry feeds capability execution); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Workflow::Observe`, `Act`, `Set`, `Send`, `Decide`); [`../../../crates/ash-interp/src/capability.rs`](../../../crates/ash-interp/src/capability.rs) (`CapabilityProvider::effect`) |
| Residual workflow / control state | The still-to-run workflow together with any runtime-owned blocked/supervision state | Direct `&Workflow` recursion in `execute_workflow_inner`, boxed `continuation` fields in AST nodes, shared `Mailbox`, `ControlLinkRegistry`, `ProxyRegistry`, and `SuspendedYields`; adjacent routing infrastructure such as `YieldRouter` / `PendingYield` exists in-repo but should be treated as nearby evidence rather than as fully demonstrated main-path carriers until later tasks pin that integration down | distributed | Normal progress updates occur by recursive descent into child `Workflow` nodes; blocked receive updates live in mailbox and polling loops; explicit yield suspension stores continuations in `YieldState` / suspended-yield registries; spawn/control updates register lifecycle state in `ControlLinkRegistry` | Both: current executable residual term plus runtime-owned blocked/control side state | [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`execute_workflow_inner`, `Workflow::Seq`, `Par`, `Receive`, `Spawn`, `Pause`, `Resume`, `Kill`); [`../../../crates/ash-interp/src/execute_stream.rs`](../../../crates/ash-interp/src/execute_stream.rs) (`execute_core_receive`, `wait_for_core_message`); [`../../../crates/ash-interp/src/runtime_state.rs`](../../../crates/ash-interp/src/runtime_state.rs) (`RuntimeState` fields); [`../../../crates/ash-interp/src/mailbox.rs`](../../../crates/ash-interp/src/mailbox.rs) (`Mailbox`); [`../../../crates/ash-interp/src/yield_state.rs`](../../../crates/ash-interp/src/yield_state.rs) (`YieldState`, `SuspendedYields`); [`../../../crates/ash-interp/src/yield_routing.rs`](../../../crates/ash-interp/src/yield_routing.rs) (`PendingYield`, `YieldRouter`) |
| `Returned(...)` | Terminal successful completion class | `Ok(Value)` from interpreter execution | direct | Produced at terminal returns (`Workflow::Done`, `Workflow::Ret`) and by successful recursive completion of enclosing workflow forms | Direct terminal class observation only; companion semantic carriers must be reconstructed elsewhere | [`../../../crates/ash-interp/src/error.rs`](../../../crates/ash-interp/src/error.rs) (`ExecResult` re-export); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Workflow::Done`, `Workflow::Ret`, `execute_workflow_inner`) |
| `Rejected(...)` | Terminal rejected/failure completion class | Mixed non-success boundary through `Err(ExecError)` from interpreter execution | reconstructed / approximated | Produced by guard/pattern/policy/capability/runtime failure paths and propagated with `?` through `execute_workflow_inner`, but also reused for non-terminal suspension-style boundaries such as `YieldSuspended` | Partial terminal-class observation only; cumulative semantic payloads are not bundled into the error, and terminal rejection is not carried by a standalone dedicated runtime enum | [`../../../crates/ash-interp/src/error.rs`](../../../crates/ash-interp/src/error.rs) (`ExecError`, `YieldSuspended`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`PolicyDenied`, guard failure, pattern failure, control-link failures) |
| Blocked / suspended class | Non-terminal waiting state distinct from rejection/stuckness | Mixed realization: implicit async waiting in `execute_core_receive`, plus explicit yield suspension in `ExecError::YieldSuspended` and `SuspendedYields`; `YieldRouter.pending` is adjacent routing infrastructure rather than yet-demonstrated uniform main-path state | implicit | Blocking receive parks inside `wait_for_core_message(...)`; proxy/yield suspension records continuation state and returns a suspension-shaped error/result boundary | Partly observable, but not as one uniform runtime result class | [`../../../crates/ash-interp/src/execute_stream.rs`](../../../crates/ash-interp/src/execute_stream.rs) (`execute_core_receive`, `wait_for_core_message`, timeout/nonblocking branches); [`../../../crates/ash-interp/src/error.rs`](../../../crates/ash-interp/src/error.rs) (`ExecError::YieldSuspended`); [`../../../crates/ash-interp/src/yield_state.rs`](../../../crates/ash-interp/src/yield_state.rs) (`SuspendedYields`); [`../../../crates/ash-interp/src/yield_routing.rs`](../../../crates/ash-interp/src/yield_routing.rs) (`YieldRouter`, `PendingYield`) |

## First-Pass Gap Classification

The mapping table already separates ordinary representational indirection from true correspondence risks.

### 1. Semantically safe indirection

These are runtime design choices that do not, by themselves, contradict the MCE-005 contract.

- `A = (C, P)` is distributed rather than packed into one struct. This is acceptable so long as capability authority and policy checks remain reconstructible from `CapabilityContext`, policy evaluators, and role context.
- `Γ` is a nested parent-linked environment instead of one flat immutable map. That is semantically fine because lookup and shadowing remain explicit and stable.
- Residual execution is mostly direct AST recursion, while receive/yield/control side state lives in runtime registries. That supports the chosen hybrid-control classification without forcing a semantic rewrite.
- `ControlLinkRegistry` externalizes supervision state (`Running`, `Paused`, `Terminated`) instead of storing it inside workflow syntax. This is acceptable if later tasks make its observable boundary explicit.

### 2. Documentation-only gaps

These are primarily corpus gaps rather than immediate semantic contradictions.

- Before TASK-401, the repo lacked one canonical table freezing where each accepted carrier currently lives.
- The runtime already had clear holder types for mailbox, control-link lifecycle, proxy lookup, and suspended yields, but MCE-006 did not previously connect them to the accepted carrier vocabulary.
- Phase 63 reporting surfaces needed explicit mention that MCE-006 now has a canonical runtime-correspondence baseline and that later tasks should build on it rather than re-inventory carriers.

### 3. Correspondence risks

These are the main places where the current runtime does not yet cleanly realize the accepted MCE-005 carriers.

- `π`, `T`, and `ε̂` are not threaded as authoritative cumulative execution carriers through `execute_workflow_inner`. Provenance and trace utilities exist nearby, but the interpreter does not currently own them as configuration state.
- Blocked/suspended behavior is not exposed through one first-class runtime result enum. Blocking `Receive` is largely implicit async waiting, while proxy yield suspension is surfaced through a specialized `ExecError::YieldSuspended` plus registries.
- `Par` currently uses `join_all(...)` over child workflow futures and returns `Value::List(...)`, which is operationally useful but not yet the same thing as a documented interleaving machine with helper-backed branch-local cumulative-state aggregation.
- Terminal runtime results are `Ok(Value)` / `Err(ExecError)`, so completion class is visible, but the semantic terminal payloads for obligations/provenance/trace/effect summary are not carried together at that boundary.
- Spawn/control lifecycle is partially realized via `ControlLinkRegistry`, but the completion-payload side of `SPEC-004` is not yet represented by an equally explicit runtime carrier in the current execution path.

## Operational Correspondence: Residual Control, Blocked/Suspended State, and Completion/Control Realization

This section is the Phase 63 / [TASK-402](../../plan/tasks/TASK-402-residual-control-blocked-state-and-completion-realization.md) freeze point for the current runtime story.

It does not redesign the runtime or reopen MCE-005. It records one conservative operational correspondence model that later tasks can reuse.

### State-classification model used

For current interpreter evidence, the runtime-facing execution-state classification is:

1. active residual execution;
2. blocked/suspended waiting;
3. terminal outcome;
4. invalid/runtime-failure boundary.

This is a runtime correspondence classification, not a new semantic syntax. The current runtime does not materialize all four classes as fully separate concrete result carriers: terminal outcomes and runtime-failure boundaries are still partially multiplexed through `Ok(Value)` / `Err(ExecError)` plus surrounding execution context.

### 1. Active residual execution: direct residual AST execution with distributed control side state

Ordinary progress is realized directly by recursive execution over residual `Workflow` ASTs.

- `execute_workflow_with_behaviour_in_state(...)` and `execute_workflow_inner(...)` in [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) recurse on the current workflow form rather than stepping an explicit frame machine.
- Sequencing/binding/control forms continue by calling `execute_workflow_inner(...)` on boxed AST continuations already stored in core workflow nodes such as `continuation` fields.
- `Workflow::Spawn` evaluates the spawn expression, binds the resulting instance, and immediately continues recursive execution; if the spawned instance exposes control authority, the runtime also registers that authority in `ControlLinkRegistry` before continuing.

So the active residual term is realized directly for ordinary execution.

But the full residual-control story is not purely AST-local:

- [`../../../crates/ash-interp/src/runtime_state.rs`](../../../crates/ash-interp/src/runtime_state.rs) stores shared runtime-owned registries for `ControlLinkRegistry`, `ProxyRegistry`, `SuspendedYields`, and `YieldRouter`.
- [`../../../crates/ash-interp/src/yield_state.rs`](../../../crates/ash-interp/src/yield_state.rs) stores resumable `YieldState.continuation` values outside the active call stack.
- [`../../../crates/ash-interp/src/control_link.rs`](../../../crates/ash-interp/src/control_link.rs) stores reusable control-link lifecycle state outside the residual workflow term itself.

Conclusion: active residual execution is direct for the ordinary in-flight workflow term, but the overall residual control representation is hybrid/distributed once runtime-owned supervision and suspension state are included.

Mapping quality:

- direct realization: current in-flight residual workflow term;
- distributed/partial realization: full machine control state once supervision/suspension registries are included.

### 2. Blocked/suspended realization: mixed implicit waiting plus explicit yield suspension

The runtime does realize blocked/suspended behavior, but not through one uniform first-class blocked enum.

#### 2.1 Blocking `Receive` and receive-path control traffic waits

Blocking receive is currently realized as implicit async waiting in [`../../../crates/ash-interp/src/execute_stream.rs`](../../../crates/ash-interp/src/execute_stream.rs):

- `execute_core_receive(...)` dispatches blocking receive modes to `wait_for_core_message(...)`.
- `wait_for_core_message(...)` loops, pumps available control or stream messages into the shared mailbox, sleeps briefly, and returns once a relevant mailbox entry exists.
- When `control` is true, `pump_available_core_message(...)` checks `stream_ctx.try_recv_control()` and pushes control messages into the mailbox under the control capability/channel path.
- Timeout receive is also handled here via `tokio::time::timeout(...)`, with wildcard-arm fallback or `Value::Null` return when the timeout expires.

This is a real blocked-state realization, but it is distributed across:

- the async task parked in the wait loop,
- `Mailbox` contents,
- `StreamContext`/provider polling,
- timeout wrappers in the receive execution path.

There is no inspected first-class runtime value like `BlockedReceive(...)` or `WaitingOnMailbox(...)`.

Mapping quality:

- distributed/partial realization for blocking receive and control waits.

#### 2.2 Yield/proxy suspension

Proxy-oriented suspension is much more explicit.

Evidence:

- `Workflow::Yield` in [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) evaluates the request, allocates a `CorrelationId`, packages a `YieldState` containing the expected response type, resume variable, and residual `continuation`, and stores it in `SuspendedYields`.
- After storing the suspended continuation, the interpreter returns `ExecError::YieldSuspended` from [`../../../crates/ash-interp/src/error.rs`](../../../crates/ash-interp/src/error.rs), carrying the routing metadata needed by the runtime/proxy side.
- `Workflow::ProxyResume` removes the `YieldState` from `SuspendedYields`, binds the response into a fresh context, and resumes by calling `execute_workflow_inner(...)` on the stored continuation.

This is the clearest current realization of a suspension boundary.

Mapping quality:

- direct realization for proxy/yield suspension as stored continuation state;
- partial realization for the global blocked/suspended class because the explicit suspension carrier is not the same carrier used by blocking receive.

#### 2.3 Completion-observation waits

Current runtime evidence for completion-observation waits is weak/missing.

- The inspected `RuntimeState` carries control, proxy, suspended-yield, and yield-routing registries, but no dedicated completion registry or completion-wait queue.
- The inspected `execute.rs` / `execute_stream.rs` paths do not show a first-class wait boundary for “observe child completion and later resume with a sealed completion payload.”

So TASK-402 can freeze only the conservative statement that completion-observation waiting is not yet evidenced as a distinct runtime carrier in the main execution path.

Mapping quality:

- missing/weak realization for completion-observation waits.

### 3. Distinguishing active, blocked/suspended, terminal, and invalid/runtime-failure state

With the above evidence, the current runtime-facing classification is:

| State class | Current runtime realization | Quality | Evidence |
|---|---|---|---|
| Active residual execution | Recursive execution over current `Workflow` term plus AST continuations | direct | [`execute_workflow_inner(...)` in `execute.rs`](../../../crates/ash-interp/src/execute.rs) |
| Blocked/suspended | Mixed carrier: implicit wait loops for receive/control; explicit `YieldState` + `ExecError::YieldSuspended` for proxy suspension | distributed/partial | [`execute_stream.rs`](../../../crates/ash-interp/src/execute_stream.rs), [`yield_state.rs`](../../../crates/ash-interp/src/yield_state.rs), [`error.rs`](../../../crates/ash-interp/src/error.rs) |
| Terminal success/failure | `Ok(Value)` / `Err(ExecError)` result boundary from interpreter execution | direct | [`execute.rs`](../../../crates/ash-interp/src/execute.rs), [`error.rs`](../../../crates/ash-interp/src/error.rs) |
| Invalid/runtime-failure | Mostly absorbed into declared runtime failure variants such as `ExecutionFailed`, `PolicyDenied`, pattern/guard failures, and missing-registry failures rather than exposed as a separate stuck/invalid object | partial/weak | [`error.rs`](../../../crates/ash-interp/src/error.rs), failure branches in [`execute.rs`](../../../crates/ash-interp/src/execute.rs) |

Important boundary notes:

- blocked/suspended is not collapsed into ordinary success because blocking receive parks inside wait loops and yield stores resumable continuation state;
- blocked/suspended is also not uniformly separated from failure at the type level, because explicit proxy suspension currently uses `ExecError::YieldSuspended` rather than a separate runtime result enum;
- invalid/stuck-like situations are generally owned by explicit error boundaries in the current interpreter, so the runtime has a practical failure boundary even though it does not expose one dedicated `invalid configuration` carrier.

### 4. Completion and control-authority realization

#### 4.1 Control authority lifecycle

Control authority is the strongest part of the current completion/control story.

Evidence:

- [`../../../crates/ash-interp/src/control_link.rs`](../../../crates/ash-interp/src/control_link.rs) defines `LinkState::{Running, Paused, Terminated}` and a `ControlLinkRegistry` that registers links, checks health, pauses, resumes, and kills them.
- The registry explicitly documents that pause/resume/check-health are reusable while the instance remains live, and that kill is terminal and invalidates future control operations.
- `Workflow::Spawn` in [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) registers control authority for spawned instances when a control handle is present.
- `Workflow::Pause`, `Resume`, `Kill`, and `CheckHealth` resolve a control link from the current context and delegate to the shared registry, turning registry failures into explicit execution failures.

This gives a direct realization of reusable-versus-terminal control authority boundaries.

Mapping quality:

- direct realization for control authority lifecycle and invalid-after-termination behavior.

#### 4.2 Completion sealing and retained completion state

The current runtime evidence is much weaker for `SPEC-004`-style completion sealing/payload retention.

What is clearly present:

- `ControlLinkRegistry` retains lifecycle state across later checks, including `Terminated`, so terminal invalidation is stable rather than transient.
- The ordinary interpreter result boundary retains only the immediate `Ok(Value)` / `Err(ExecError)` returned from execution.

What is not evidenced on the inspected main path:

- no dedicated runtime completion tombstone or payload-bearing completion record in `RuntimeState`;
- no explicit retained `CompletionPayload`-style carrier holding the child's terminal obligations/provenance/trace/effect summary for later observation;
- no dedicated completion-observation wait channel comparable to `SuspendedYields` for proxy suspension.

So the conservative conclusion is:

- control-link lifecycle and terminal control invalidation are explicit and operationally clear;
- retained completion payload realization is only partial/indirect today and should not be overclaimed as a full `SPEC-004` completion-payload implementation.

### 5. TASK-402 correspondence conclusion

The current runtime can therefore be described conservatively as follows:

- residual execution is primarily direct residual-AST execution, but effective control state is hybrid because registries carry supervision and suspension state outside the active AST node;
- blocked/suspended execution is realized by two different mechanisms: implicit async waiting for receive/control traffic and explicit stored continuations for proxy yield suspension;
- terminal outcomes are observable through `Ok(Value)` / `Err(ExecError)`, but those concrete result carriers do not fully separate declared rejection from every runtime-failure boundary on their own;
- invalid/runtime-failure states are owned by explicit error boundaries rather than a separate stuck-state object, so the distinction from terminal outcome is currently partly semantic/interpretive rather than carried by a dedicated standalone runtime enum;
- control authority is directly realized through `ControlLinkRegistry` lifecycle states;
- completion sealing/retained completion observation is still weak/partial and remains a real follow-on gap for later MCE-006 closeout work.

## Operational Correspondence: `Par` Interleaving, Branch-Local State, and Terminal Aggregation

This section is the Phase 63 / [TASK-403](../../plan/tasks/TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md) freeze point for the current `Par` story.

As with TASK-401 and TASK-402, it is descriptive rather than redesign-oriented. It records what the current interpreter actually does, which parts correspond directly to the accepted MCE-005 / `SPEC-004` concurrency contract, and which parts remain only partial or missing realizations.

### 1. Current `Workflow::Par` operational execution model

The inspected interpreter path for `Workflow::Par` lives in [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs).

Evidence:

- `Workflow::Par { workflows }` special-cases the empty branch list and returns `Ok(Value::Null)`.
- For a non-empty branch list, the interpreter maps each child workflow to one `execute_workflow_inner(...)` future.
- Each child future is built immediately from the current branch list iteration and then all child futures are awaited together with `futures::future::join_all(futures).await`.
- After the await, the runtime folds the joined child results with `collect::<Result<Vec<_>, _>>()?` and, if every child succeeded, returns `Ok(Value::List(Box::new(values)))`.

So the current operational model is not left-to-right sequential recursion over `Par` children. It is a bulk-concurrent construct-and-await shape: construct one child future per branch, jointly await them, then aggregate terminal child results.

Mapping quality:

- direct realization: concurrent child-future construction plus one terminal collation boundary;
- partial realization: runtime execution of multiple branches without collapsing `Par` to obvious sequential evaluation;
- missing realization: no explicit runtime scheduler object or first-class “step one branch, then reassemble residual `Par`” machine state.

### 2. Where branch-local state currently lives, and what is shared

The current branch-local/shared split is visible from the actual `execute_workflow_inner(...)` call arguments in the `Workflow::Par` arm plus the carrier definitions in [`../../../crates/ash-interp/src/context.rs`](../../../crates/ash-interp/src/context.rs), [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs), [`../../../crates/ash-interp/src/runtime_state.rs`](../../../crates/ash-interp/src/runtime_state.rs), and [`../../../crates/ash-interp/src/mailbox.rs`](../../../crates/ash-interp/src/mailbox.rs).

#### 2.1 Branch-local carriers

Each child branch receives its own `ctx.clone()`.

That means the most direct branch-local runtime carrier today is the cloned `Context`, whose fields include:

- `bindings: HashMap<Name, Value>`;
- `parent: Option<Box<Context>>`;
- `obligations: RefCell<HashSet<Name>>`;
- `role_context: Option<RoleContext>`.

Operationally, this means:

- each `Par` branch starts from the same incoming environment snapshot;
- subsequent branch-local bindings/lookup shadowing happen inside that branch's own cloned `Context` value;
- branch-local generic obligations and role context also live inside that clone rather than in one shared mutable `Context`.

This is the clearest current realization of branch-local `Γ`, and it is also the closest current runtime holder for branch-local pieces of `Ω`.

Mapping quality:

- direct realization: branch-local `Γ` snapshot and branch-local residual workflow recursion;
- distributed/partial realization: branch-local `Ω`, because obligations live in cloned `Context`/`RoleContext` state but there is no evidenced helper-backed concurrent merge back into one authoritative terminal `Ω` carrier;
- missing realization: branch-local `π`, `T`, and `ε̂`, because the interpreter still does not thread authoritative provenance/trace/effect accumulators through ordinary execution.

#### 2.2 Shared carriers and coordination surfaces

Each child branch also receives shared or effectively shared runtime surfaces:

- the same borrowed `cap_ctx`, `policy_eval`, and `behaviour_ctx` references;
- the same borrowed `stream_ctx` reference when present;
- `mailbox.clone()`, where `SharedMailbox = Arc<Mutex<Mailbox>>`;
- `control_registry.clone()`, where the registry comes from `RuntimeState.control_registry`;
- `proxy_registry.clone()`, where the registry comes from `RuntimeState.proxy_registry`;
- `suspended_yields.clone()`, where the registry comes from `RuntimeState.suspended_yields`.

So branch-local state is not total isolation. Branch execution is locally separated at the `Context` level, but runtime-owned coordination/supervision infrastructure is shared through `Arc<Mutex<...>>` holders.

This matters because mailbox traffic, control-link lifecycle, proxy lookup, and suspended-yield storage are not accumulated in one per-branch helper-owned concurrent state object before aggregation. They remain shared runtime surfaces consulted or updated by all child executions.

Mapping quality:

- direct realization: shared mailbox/control/proxy/suspension infrastructure across child branches;
- distributed realization: the full concurrent machine state lives partly in branch-local `Context` clones and partly in shared runtime registries;
- missing realization: no inspected per-branch helper-owned cumulative carrier for `Ω` / `π` / `T` / `ε̂` waiting to be joined at `Par` completion.

### 3. Semantic interleaving promised by MCE-005 versus current operational concurrency strategy

The accepted upstream contract from [MCE-005](MCE-005-SMALL-STEP.md) is stronger and more specific than “start several futures and later collect them.”

Semantically, `Par` remains:

- interleaving-based;
- branch-progress oriented;
- paired with helper-backed aggregation at terminal convergence.

Current runtime evidence supports a more modest claim.

The interpreter does not currently expose a first-class runtime object corresponding to:

- a residual `Par` configuration with individually stepable child residual workflows;
- an explicit next-branch selection function or scheduler relation;
- a fairness claim over branch stepping order.

Instead, it constructs all child futures and delegates actual polling/interleaving order to the async runtime plus `join_all(...)`.

That means the current runtime does preserve one important semantic point: `Par` is not reduced to obvious left-to-right sequential execution. But it does not yet realize the semantic contract in the same form as the accepted small-step interleaving machine.

The conservative classification is therefore:

- direct: runtime intent to execute child branches concurrently rather than sequentially;
- partial/distributed: operational interleaving, because actual branch progress is mediated by async future polling rather than an explicit small-step branch scheduler owned by the interpreter model;
- missing: any evidenced runtime fairness or explicit per-step branch-selection correspondence theorem.

So TASK-403 should treat the current `Par` operational model as a usable approximation of the accepted interleaving contract, not as a full direct realization of that contract.

### 4. Current terminal aggregation and its correspondence limits

The current terminal aggregation boundary is concrete and inspectable.

#### 4.1 What the runtime does today

For `Workflow::Par` in `execute.rs`:

- empty `Par` returns `Value::Null` directly;
- non-empty `Par` awaits all child futures with `join_all(...)`;
- the resulting `Vec<ExecResult<Value>>` is folded with `collect::<Result<Vec<_>, _>>()?`;
- if every child completed successfully, the runtime returns `Value::List(Box::new(values))` in the same branch iteration order used to build the futures vector;
- if any child failed, the collection step returns an `Err(ExecError)` rather than a successful aggregate value.

This is a real terminal collation mechanism, and it is more than an undocumented implementation accident.

Mapping quality:

- direct realization: terminal child-value collation into `Value::List` on all-success paths;
- direct realization: after the joint await, any child failure is surfaced at the `ExecResult` boundary during result folding;
- partial realization: one operational all-branches-complete aggregation point.

#### 4.2 Why this is not yet the accepted helper-backed semantic aggregation contract

The accepted semantic contract from MCE-005 / `SPEC-004` is not merely “collect branch values into a list.” It also speaks in terms of helper-backed aggregation over cumulative semantic carriers such as obligations, provenance, traces, and effect summaries.

Current runtime evidence does not support claiming that the present `join_all(...)` + `Value::List(...)` path fully realizes that stronger contract.

Why the claim would be too strong:

- `Ω` is only partially represented, and the current `Par` path shows cloned branch-local `Context` obligation state but no explicit helper-backed concurrent join back into one authoritative terminal obligation state.
- `π`, `T`, and `ε̂` are still missing as authoritative cumulative runtime carriers on the main execution path, so there is nothing concrete here that could be joined in the accepted semantic sense.
- the current aggregate result surface is only `Ok(Value)` / `Err(ExecError)`, not a retained helper result bundling authoritative concurrent semantic payloads.
- `Value::List` fixes one positional output ordering, while the accepted semantic story is about interleaving plus helper-backed branch aggregation rather than merely preserving input-list position as the whole meaning of parallel composition.

The conservative conclusion is therefore:

- direct: current runtime aggregates successful terminal child values into one list result;
- partial/distributed: current runtime has one practical operational `Par` completion boundary;
- missing/correspondence-risk: helper-backed branch-local cumulative-state aggregation for `Ω` / `π` / `T` / `ε̂` is not evidenced and should not be claimed.

### 5. TASK-403 correspondence conclusion

The current `Par` realization can now be frozen as follows:

- operational execution uses bulk async child-future construction and `join_all(...)`, not left-to-right sequential recursion;
- branch-local execution state lives most directly in cloned `Context` values and branch-local residual recursive execution;
- mailbox/control/proxy/suspension infrastructure remains shared across branches through runtime-owned `Arc<Mutex<...>>` carriers;
- semantic interleaving is only partially realized operationally, because the runtime does not expose an explicit branch-step scheduler or helper-owned residual-`Par` machine state;
- terminal child-value aggregation into `Value::List` is direct for successful runtime values, but it is not sufficient evidence for full helper-backed semantic aggregation over cumulative carriers.

This freezes the recommended Phase 63 classification for `Par` as:

- operational model: distributed/partial realization of the accepted interleaving contract;
- aggregation correspondence: partial for terminal value collation, missing for full helper-backed cumulative-state aggregation.

## Phase 63 Closeout: Observable Preservation, Divergence Taxonomy, and MCE-007 Handoff

Phase 63 can now be treated as the frozen MCE-006 runtime-correspondence evidence bundle.

Closeout verdict:

- the current interpreter already realizes part of the accepted [MCE-005](MCE-005-SMALL-STEP.md) backbone for observable purposes;
- it does not yet realize that backbone completely at the level of authoritative cumulative observables;
- the correct overall MCE-006 verdict is therefore: partial observable realization, with contract-preserving follow-up still required for missing cumulative carriers and retained completion-style payloads.

Concretely, the current runtime already provides:

- a real workflow-first execution substrate through recursive execution in [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs);
- a direct successful terminal result boundary through `Ok(Value)` and a concrete non-success boundary through `Err(ExecError)` in [`../../../crates/ash-interp/src/error.rs`](../../../crates/ash-interp/src/error.rs);
- a real blocked/suspended story, but only through mixed carriers (`execute_stream` wait loops plus explicit yield suspension);
- partial obligation carriage in `Context` and `RoleContext`;
- no authoritative main-path cumulative carriers for provenance `π`, trace `T`, or effect summary `ε̂`.

### Observable-preservation checklist

| Observable | Current runtime carrier / boundary | Preservation classification | Conservative closeout statement | Concrete evidence |
|---|---|---|---|---|
| Successful return outcome class | `Ok(Value)` terminal boundary from interpreter execution | direct | Successful terminal return is directly observable at the runtime result boundary, even though companion semantic payloads are not bundled there. | [`../../../crates/ash-interp/src/error.rs`](../../../crates/ash-interp/src/error.rs) (`ExecResult`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Workflow::Done`, `Workflow::Ret`, `execute_workflow_inner`) |
| Rejected / non-success terminal outcome class | `Err(ExecError)` terminal boundary, with variants such as `PolicyDenied`, `ExecutionFailed`, pattern/guard/control failures, and `YieldSuspended` | reconstructed / approximated | Non-success is directly visible, but the runtime multiplexes declared rejection, invalid/runtime-failure ownership, and one explicit suspension-shaped boundary through the same broad error channel, so semantic rejection subclasses are only partially reconstructed from `ExecError` variant plus execution context. | [`../../../crates/ash-interp/src/error.rs`](../../../crates/ash-interp/src/error.rs) (`ExecError`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (policy, pattern, guard, control-link, and yield paths) |
| Blocked vs terminal vs invalid/runtime-failure status | Mixed realization: async wait loops in `execute_stream`, explicit `ExecError::YieldSuspended` plus `SuspendedYields`, and ordinary `Ok(Value)` / `Err(ExecError)` terminal boundaries | correspondence risk | The distinction is partly observable, but not through one authoritative runtime state/result carrier. Blocking receive is implicit/distributed, explicit yield suspension uses a specialized error variant, and invalid/runtime-failure is largely owned by error variants rather than a dedicated invalid-state object. | [`../../../crates/ash-interp/src/execute_stream.rs`](../../../crates/ash-interp/src/execute_stream.rs) (`execute_core_receive`, `wait_for_core_message`); [`../../../crates/ash-interp/src/error.rs`](../../../crates/ash-interp/src/error.rs) (`YieldSuspended`); [`../../../crates/ash-interp/src/yield_state.rs`](../../../crates/ash-interp/src/yield_state.rs) (`YieldState`, `SuspendedYields`) |
| `Ω` obligations | Distributed across `Context.obligations` and role-scoped `RoleContext` obligation/discharge state | reconstructed / approximated | Obligation state is genuinely carried and mutated at runtime, but it is split across generic and role-specific mechanisms and is not packaged as one authoritative terminal observable at the main result boundary. | [`../../../crates/ash-interp/src/context.rs`](../../../crates/ash-interp/src/context.rs) (`obligations`, `add_obligation`, `discharge_obligation`); [`../../../crates/ash-interp/src/role_context.rs`](../../../crates/ash-interp/src/role_context.rs) (`RoleContext`, `discharge`, `pending_obligations`, `all_discharged`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Workflow::Oblig`, `Workflow::Check`) |
| `π` provenance | Only partial metadata carriers on values/IR-side structures; no authoritative mutable execution accumulator | weak / missing | The repo contains provenance-related types and helper utilities, but the main interpreter path does not thread a live provenance carrier through execution, and `Workflow::Act` does not use its provenance payload to update one cumulative runtime state. | [`../../../crates/ash-core/src/provenance.rs`](../../../crates/ash-core/src/provenance.rs) (`Provenance`, `fork`, `TraceEvent`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Workflow::Act { provenance: _ }`); [`../../../crates/ash-interp/src/capability_provenance.rs`](../../../crates/ash-interp/src/capability_provenance.rs) (`CapabilityProvenanceTracker`) |
| `T` trace | No authoritative cumulative trace accumulator on the main execution path | weak / missing | Trace-like utilities exist nearby, but there is no interpreter-owned append-only cumulative trace structure returned from or threaded through `execute_workflow_*`. | [`../../../crates/ash-core/src/provenance.rs`](../../../crates/ash-core/src/provenance.rs) (`TraceEvent`); [`../../../crates/ash-interp/src/capability_provenance.rs`](../../../crates/ash-interp/src/capability_provenance.rs) (`events`, `record`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`ExecResult<Value>` boundary only) |
| `ε̂` effect summary | No shared interpreter-owned cumulative effect accumulator; only local capability/provider effect metadata and operation-specific checks | weak / missing | Effect-related metadata exists at capability/provider boundaries, but the current interpreter does not own one cumulative effect-summary carrier that MCE-007 could treat as the authoritative runtime realization of `ε̂`. | [`../../../crates/ash-interp/src/capability.rs`](../../../crates/ash-interp/src/capability.rs) (`CapabilityProvider::effect`); [`../../../crates/ash-interp/src/runtime_state.rs`](../../../crates/ash-interp/src/runtime_state.rs) (`providers`); [`../../../crates/ash-interp/src/execute.rs`](../../../crates/ash-interp/src/execute.rs) (`Observe`, `Act`, `Set`, `Send`, `Decide`) |

Additional closeout note:

- completion-payload-style observables remain weak/missing on the inspected main path as already frozen by TASK-402: `ControlLinkRegistry` retains lifecycle state, but no equally explicit retained completion payload carrier was evidenced in `RuntimeState` or the main execution path.

### Frozen divergence taxonomy

| Divergence category | Meaning in MCE-006 closeout | Phase 63 examples |
|---|---|---|
| Semantically safe indirection | Runtime structure differs from the semantic presentation, but observable meaning is still preserved or reconstructible without changing the accepted contract. | Distributed `A = (C, P)` holders; parent-linked `Γ`; hybrid residual control with runtime registries; shared runtime registries around direct residual AST execution. |
| Documentation gap | The runtime already had a usable story, but the corpus had not yet frozen or packaged it clearly. | Pre-TASK-404 absence of one observable-preservation checklist and one concise MCE-007 handoff packet, despite TASK-401 through TASK-403 already freezing the underlying evidence. |
| Correspondence risk | The current runtime exposes only a partial or mixed realization of a required observable boundary, so MCE-007 must not mark the row fully closed. | Mixed blocked/suspended versus terminal/invalid classification; `Err(ExecError)` multiplexing several non-success shapes; `Par` value collation without authoritative cumulative-state aggregation. |
| Contract-preserving follow-up required | Missing or weak carriers can be improved later, but only as implementation/correspondence follow-up; they do not justify reopening MCE-005 semantics. | Authoritative cumulative carriers for `π`, `T`, and `ε̂`; stronger terminal packaging for `Ω`; retained completion payload/state observation carriers; helper-backed concurrent cumulative-state aggregation. |

### Concise MCE-007 handoff packet

TASK-398 / MCE-007 should ingest Phase 63 as follows:

- overall verdict: small-step → interpreter correspondence is partially realized, not fully closed;
- rows that already have usable runtime evidence: workflow-first execution substrate, distributed `A = (C, P)` mapping, direct `Γ` carriage, direct successful terminal return boundary, explicit control-link lifecycle, real but mixed blocked/suspended realization, and direct `Par` child-value collation into `Value::List` on all-success paths;
- rows that must stay partial / follow-up-required in the MCE-007 matrix: rejected-vs-runtime-failure subtype separation, blocked vs terminal vs invalid as one authoritative runtime class, authoritative terminal `Ω` packaging, authoritative `π`, `T`, and `ε̂` carriers, retained completion payloads, and helper-backed concurrent cumulative-state aggregation;
- evidence packet to cite: this document plus [TASK-401](../../plan/tasks/TASK-401-runtime-carrier-inventory-and-semantic-mapping-table.md), [TASK-402](../../plan/tasks/TASK-402-residual-control-blocked-state-and-completion-realization.md), [TASK-403](../../plan/tasks/TASK-403-par-interleaving-branch-state-and-aggregation-correspondence.md), and [TASK-404](../../plan/tasks/TASK-404-observable-preservation-gap-classification-and-mce-007-handoff.md);
- non-overclaim rule for MCE-007: do not infer hidden cumulative runtime carriers from nearby provenance/effect utility types or from `Ok(Value)` / `Err(ExecError)` alone.

## Alignment Strategy

The conservative strategy for MCE-006 remains:

1. start from the accepted MCE-005 backbone;
2. inventory the current interpreter/runtime structures that correspond to each semantic carrier;
3. document gaps explicitly as representation or scheduling mismatches;
4. decide whether the current interpreter already realizes the backbone, needs a thin abstract-machine description, or needs contract-preserving refactoring.

TASK-401 completed step 2 and established the frozen baseline for the remaining Phase 63 tasks. TASK-402 added the concrete control/blocking/completion correspondence layer on top of that baseline, and TASK-403 now freezes the current `Par` execution / branch-state / aggregation story.

## Expected Outputs

MCE-006 should eventually produce:

1. a semantic-carrier-to-runtime mapping table;
2. a branch/interleaving realization story for `Par`;
3. a blocked-state realization story for `Receive`, control-owned waits, and explicit yield suspension;
4. a terminal observable preservation checklist for traces, effects, obligations, provenance, and rejection/return outcomes;
5. a statement of whether the current interpreter already realizes the accepted small-step backbone or only approximates it.

Status after TASK-404:

- item 1 now exists in canonical form in this document;
- item 2 now exists in conservative runtime correspondence form for current `Par` execution, branch-local state, and terminal aggregation;
- item 3 now exists in conservative runtime correspondence form for receive/control waits, proxy suspension, and completion/control boundaries;
- item 4 now exists in explicit closeout form as an observable-preservation checklist with conservative row classifications and concrete evidence anchors;
- item 5 now exists in final closeout form: the current interpreter partially realizes the accepted backbone for observable purposes, but does not yet fully realize authoritative cumulative-carrier preservation.

## Relationship to MCE-007

With TASK-404 complete, [MCE-007](MCE-007-FULL-ALIGNMENT.md) can treat MCE-006 as a frozen runtime-evidence packet rather than as an open exploratory note.

The immediate MCE-007-facing takeaway after TASK-404 is:

- the interpreter already has a real execution/control substrate and enough runtime evidence to justify partial small-step → interpreter alignment claims;
- Phase 63 now packages that evidence in one place for carrier placement, control/blocking realization, `Par` execution/aggregation, observable preservation, and gap classification;
- MCE-007 should still keep cumulative semantic carriers such as `Ω` terminal packaging, `π`, `T`, `ε̂`, retained completion payloads, and full helper-backed concurrent aggregation marked partial/follow-up rather than closed.

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Need to close theory-practice gap |
| 2026-04-05 | Reframed around the accepted MCE-005 backbone | Phase 61 fixed the semantic target; remaining work is runtime/interpreter alignment rather than upstream semantic ambiguity |
| 2026-04-05 | Classified the current runtime as a hybrid control representation and froze a canonical semantic-carrier → runtime mapping table | Direct residual AST execution is primary, but receive/yield/control behavior lives partly in runtime-owned registries and stored continuations |
| 2026-04-05 | Added an explicit operational correspondence section for residual control, blocked/suspended state, and completion/control realization | TASK-402 freezes one conservative runtime story: active execution is primarily direct AST recursion, receive waiting is implicit/distributed, proxy suspension is explicit, control authority is directly realized, and completion-payload retention remains only partial/weak |
| 2026-04-05 | Added an explicit `Par` correspondence section covering interleaving strategy, branch-local state, and terminal aggregation | TASK-403 freezes the current `Par` story as bulk async concurrency with cloned branch-local `Context` state and shared runtime registries, plus direct list-value collation but only partial/missing correspondence to helper-backed cumulative-state aggregation |
| 2026-04-05 | Added the observable-preservation closeout, divergence taxonomy, and concise MCE-007 handoff packet | TASK-404 closes Phase 63 conservatively: direct runtime result boundaries and real control substrate are preserved, but authoritative cumulative carriers and retained completion-style payloads remain only partial or missing |