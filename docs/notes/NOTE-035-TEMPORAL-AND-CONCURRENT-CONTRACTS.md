# NOTE-035: Temporal and Concurrent Contracts

**Date:** 2026-06-29
**Status:** Living document — design direction captured; resolves NOTE-014 GAP 5
**Purpose:** Define temporal/concurrent contracts over Ash's ambient computation model. This note replaces the older idea that `Proc` and `Workflow` need separate contract systems with a single trace-contract substrate. As of Phase 201, `Act`, `Proc`, and `Workflow` are removed historical development forms and may appear only as reference vocabulary in historical docs.

## Pre-Spec Delta

This note should be reconciled into the target specs as follows:

- **SPEC-096 / SPEC-096b Effect System:** add trace/monitor-oriented contract effects and clarify that old `Pure`, `Act`, `Proc`, and `Workflow` wording is legacy reference vocabulary over one ambient computation model, not target surface/Core/IR/runtime machinery.
- **SPEC-097b Type System:** define trace-contract well-formedness separately from value-predicate well-formedness. Trace contracts mention typed event facts, monitor clocks, obligation/evidence facts, and policy facts; they do not perform capability or process operations.
- **SPEC-098b Target IR:** add sidecar shapes for `TraceEvent`, `TraceContract`, `MonitorPlan`, `TemporalContractDiagnostic`, and `WorkflowLedgerFact`.
- **SPEC-099 Core Language:** clarify that Core may record trace events and attach monitor sidecars without adding a separate `Proc` or `Workflow` term family.
- **SPEC-100 Core Type Checking:** add the staged checking rule for trace contracts: classify the event alphabet, type-check the temporal formula, choose static/model-check/evidence/runtime-monitor discharge, and record monitor metadata.
- **SPEC-101 Evaluation Modes:** if later reconciled, state how lazy/memo replay interacts with trace emission and temporal monitor observations.

## 0. Motivation

NOTE-014 identified GAP 5 as the open question for concurrent, distributed, and temporal contracts. The original gap text described `Proc` and `Workflow` as tower levels whose contracts looked fundamentally different from sequential Hoare contracts.

That framing is now too rigid.

Ash has moved from separate `Act`, `Proc`, and `Workflow` monad constructors toward one ambient computation model enriched by rows, handlers, evidence, obligations, provenance, and monitor metadata. Those old names are now historical reference vocabulary only; new development should use process/channel/governance facts over ambient computations.

So NOTE-035 does not introduce a separate contract system for each old tower level. It introduces one monitorable trace-contract substrate:

```text
ambient computation
  -> emits typed trace/evidence facts
  -> monitors check temporal properties over those facts
  -> workflow interpretation lifts some facts into obligations, commitments, evidence, and stage state
```

The difference between operational and normative trace contracts is therefore interpretive:

```text
operational trace: operational facts such as spawn, send, receive, fail, cancel, restart
normative trace:   interpreted facts such as obligation opened/discharged, evidence accepted,
                   approval committed, stage advanced, compensation required
```

The machinery is shared. The alphabet and meaning become richer.

## 1. Core decision

Temporal/concurrent contracts are **trace contracts over the ambient monad**.

Historical `Pure`, `Act`, `Proc`, and `Workflow` wording names older regions in the space of rows, effects, traces, obligations, evidence, and governance facts. These names are not disjoint contract mechanisms and are not target development forms.

Therefore:

1. A trace contract is checked against an event/fact stream, not against a single value boundary.
2. Operational contracts mention process/channel events: spawn, send, receive, complete, fail, cancel, restart, acquire, release, timeout.
3. Normative contracts mention interpreted facts over the same substrate: obligation opened/discharged, commitment made, evidence accepted, actor authorized, stage committed, compensation required.
4. A computation may mix features from several anchors. The contract is classified by the facts it mentions, not by a hard enclosing type constructor.
5. Most trace contracts discharge dynamically through monitors. Static proof or model checking may discharge bounded finite-state protocols, but unknown obligations demote to monitors rather than to value-level dynamic predicates.
6. Monitors consume trace/evidence facts. They do not acquire capability or process authority.

Short form:

```text
Process/runtime facts constrain what the runtime trace does.
Normative facts constrain what selected trace facts mean.
```

## 2. Ambient-axis model

Older Ash documentation sometimes presented:

```text
Pure<A>
Act<A>
Proc<A>
Workflow<A>
```

as separate constructors. The target model is better described as:

```text
Ambient<A, row, evidence, obligations, trace, monitors, ...>
```

Operational/normative regions are approximate areas of this feature space:

| Region | Typical features | Contract interpretation |
|--------|------------------|-------------------------|
| value | values, refinements, equations | predicate over values |
| operation | operation rows, authority, action evidence | Hoare boundary over an effectful action |
| process | process/channel/lifecycle events | temporal property over operational trace |
| normative | obligations, commitments, evidence, policy, stages | temporal/normative property over interpreted trace or ledger |

A computation may carry mixed features:

```text
{PosixFs::read, channel send Msg, process spawn, obligation approve_invoice, evidence audit_log}
```

This is not a violation of the model. It is an ambient computation whose facts sit across several
runtime and interpretive regions.

## 3. Trace contracts versus value contracts

Value contracts check a predicate at a boundary:

```text
requires { x > 0 }
ensures  { result >= old(x) }
```

Trace contracts check a temporal formula over a sequence of typed facts:

```text
always(Receive(request_id) -> eventually(Send(response_for(request_id))))
always(ChildFailed(pid) -> eventually_within(Restart(pid), 5s))
always(StageCommitted(pay_invoice) -> previously(ObligationDischarged(approve_invoice)))
```

These examples are explanatory temporal notation, not committed Ash surface syntax. The important semantic difference is:

```text
value contract:  Predicate(Environment) -> Bool
trace contract:  TemporalFormula(TraceFacts, MonitorState) -> MonitorResult
```

A trace contract may still mention values, but only as fields inside typed events or ledger facts.

## 4. Event and fact model

Trace contracts consume facts. A fact can be a raw operational event or an interpreted workflow fact.

Conceptual IR shape:

```rust
pub enum TraceFactKind {
    Process(ProcessEvent),
    Channel(ChannelEvent),
    Resource(ResourceEvent),
    Operation(OperationEvent),
    Contract(ContractEvent),
    Workflow(WorkflowLedgerFact),
    Evidence(EvidenceEvent),
    Time(TimerEvent),
}
```

Process-oriented facts include:

```rust
pub enum ProcessEvent {
    Spawned { parent: ProcessId, child: ProcessId },
    Started { process: ProcessId },
    Completed { process: ProcessId, result: ValueRef },
    Failed { process: ProcessId, reason: TrapRef },
    Cancelled { process: ProcessId, reason: CancelReason },
    Restarted { supervisor: ProcessId, child: ProcessId },
    Joined { waiter: ProcessId, child: ProcessId },
    TimedOut { process: ProcessId, timer: TimerId },
}
```

Channel-oriented facts include:

```rust
pub enum ChannelEvent {
    Sent { channel: ChannelId, message: ValueRef, sender: ProcessId },
    Received { channel: ChannelId, message: ValueRef, receiver: ProcessId },
    Selected { selector: SelectorId, branch: BranchId },
    Closed { channel: ChannelId, closer: ProcessId },
}
```

Workflow-oriented facts are interpreted facts over the same substrate:

```rust
pub enum WorkflowLedgerFact {
    ObligationOpened { obligation: ObligationId, actor: ActorRef, source: TraceFactRef },
    ObligationDischarged { obligation: ObligationId, evidence: EvidenceRef, source: TraceFactRef },
    CommitmentMade { commitment: CommitmentId, actor: ActorRef, source: TraceFactRef },
    StageCommitted { stage: StageId, evidence: Vec<EvidenceRef>, source: TraceFactRef },
    CompensationRequired { obligation: ObligationId, reason: DiagnosticRef },
    EvidenceAccepted { evidence: EvidenceRef, policy: PolicyRef, source: TraceFactRef },
    EvidenceRejected { evidence: EvidenceRef, policy: PolicyRef, reason: DiagnosticRef },
}
```

`WorkflowLedgerFact` does not require a separate workflow runtime. It is a normative/evidential interpretation attached to trace facts.

## 5. Trace contract shape

A trace contract records its alphabet, temporal formula, monitor plan, and diagnostic shape:

```rust
pub struct TraceContract {
    pub id: TraceContractId,
    pub source_span: Span,
    pub contract_text: String,
    pub boundary: BoundaryId,
    pub alphabet: TraceAlphabet,
    pub formula: TemporalFormula,
    pub interpretation: TraceInterpretation,
    pub discharge: TraceContractDischarge,
    pub diagnostic_shape: TemporalDiagnosticShape,
}

pub enum TraceInterpretation {
    Operational,
    Normative,
    Mixed,
}
```

The `interpretation` is descriptive, not a hard type level. A contract that only mentions process/channel facts is operational. A contract that mentions obligation/evidence/stage facts is normative. A contract that relates both is mixed:

```text
always(Receive(request) -> eventually(ObligationDischarged(handle_request)))
```

## 6. Temporal formula profile

NOTE-035 uses a small temporal vocabulary as design notation:

```rust
pub enum TemporalFormula {
    Fact(TracePattern),
    Not(Box<TemporalFormula>),
    And(Box<TemporalFormula>, Box<TemporalFormula>),
    Or(Box<TemporalFormula>, Box<TemporalFormula>),
    Implies(Box<TemporalFormula>, Box<TemporalFormula>),
    Always(Box<TemporalFormula>),
    Eventually(Box<TemporalFormula>),
    Until(Box<TemporalFormula>, Box<TemporalFormula>),
    Within { formula: Box<TemporalFormula>, bound: DurationExpr },
    Previously(Box<TemporalFormula>),
    Since(Box<TemporalFormula>, Box<TemporalFormula>),
}
```

This is not yet source grammar. It is the Core/IR-side shape that future surface syntax can target.

Bounded operators such as `Within` are monitorable with finite timer state. Unbounded liveness such as `Eventually` may remain pending until the monitored scope closes. A monitor must distinguish:

```text
satisfied       -- obligation observed
violated        -- impossible or deadline exceeded
pending         -- not yet decided
inconclusive    -- trace cut off or monitor scope ended without enough evidence
faulted         -- monitor evaluator failed
```

## 7. Discharge modes

Trace contracts reuse the general discharge idea, but their modes differ from value predicates:

```rust
pub enum TraceContractDischarge {
    StaticModelChecked { evidence: EvidenceRef },
    StaticProved { evidence: EvidenceRef },
    EvidenceSurvivedTesting { evidence: EvidenceRef },
    RuntimeMonitor { plan: MonitorPlanRef },
    Deferred { reason: DeferralReason },
}
```

Most trace contracts should lower to `RuntimeMonitor`. Static proof/model checking is appropriate when the relevant process/channel protocol is finite-state or otherwise summarized by trusted evidence.

Unknown trace proof does not become a value-level `RuntimeCheckPlan`. It becomes a trace-level `MonitorPlan` or an explicit deferral.

## 8. Monitor authority boundary

Monitors consume facts; they do not create authority.

This follows NOTE-034:

```text
ordinary computation performs authority-bearing operations
runtime records operation/process/channel/workflow facts
monitor evaluates temporal formula over recorded facts
```

A monitor must not:

- call a capability provider;
- spawn, cancel, restart, send, or receive on its own behalf;
- install handlers;
- observe wall-clock, randomness, or environment except through admitted timer facts;
- reinterpret redacted evidence as full values.

Timers are facts too. A timeout monitor consumes `TimerEvent` or scheduler-provided deadline facts; it does not independently acquire ambient clock authority.

## 9. Failure and diagnostic semantics

Trace-contract failure is structured bottom by default, parallel to NOTE-029 but not identical to value-predicate failure.

Conceptual trap reasons:

```rust
pub enum TrapReason {
    ContractViolation(ContractDiagnostic),
    ContractPredicateFault(PredicateFaultDiagnostic),
    TemporalContractViolation(TemporalContractDiagnostic),
    TemporalMonitorFault(TemporalMonitorFaultDiagnostic),
    ...
}
```

A temporal violation means the trace made the temporal obligation false:

```text
ChildFailed(pid) occurred, but Restarted(pid) did not occur before the 5s deadline.
```

A monitor fault means the monitor could not evaluate correctly:

```text
monitor state corrupted
unknown event schema
clock fact missing for a bounded formula
redacted evidence required by an unredactable diagnostic shape
```

Recoverable behavior remains explicit. If a temporal violation should trigger compensation rather than terminal bottom, the program must expose a row-accounted `fail`, compensation operation, or workflow obligation path. `TemporalContractViolation` is not silently resumable.

## 10. Workflow lifting

Workflow interpretation is a lifting step:

```text
TraceFact
  -> policy/evidence/provenance interpretation
  -> WorkflowLedgerFact
  -> workflow-level trace contract
```

For example:

```text
ProcessEvent::Completed { process: approve_invoice, result: approved }
ObservationEvidence { actor: manager, policy: invoice_policy, ... }
```

may be lifted to:

```text
WorkflowLedgerFact::ObligationDischarged {
    obligation: approve_invoice,
    evidence: approval_evidence,
    source: completed_event,
}
```

The workflow fact points back to the operational source fact. This preserves auditability and prevents workflow semantics from floating free of runtime behavior.

## 11. Grammar implications

NOTE-035 does not commit public surface syntax. It does, however, reserve the following semantic targets for future grammar:

```ebnf
trace_contract = temporal_contract | monitor_contract | workflow_obligation_contract ;

temporal_contract = "always" "{" temporal_formula "}"
                  | "eventually" "{" temporal_formula "}"
                  | "within" duration "{" temporal_formula "}"
                  ;

monitor_contract = "monitor" identifier "{" temporal_formula "}" ;
workflow_obligation_contract = "obligation" obligation_path [ obligation_clause ] ;
```

This is target notation only. Existing target specs may describe row items and IR carriers before selecting exact surface keywords.

## 12. Type-system implications

Trace-contract well-formedness is separate from value-predicate well-formedness:

```text
Γtrace ⊢ formula ⇓ TraceContract
```

The trace environment contains:

- event schemas;
- event payload types;
- process/channel/resource identities;
- timer/deadline facts;
- workflow ledger fact schemas;
- evidence/provenance policies;
- redaction rules;
- monitor scope boundaries.

The checker rejects formulas that mention facts outside the monitor scope or require authority not represented as an event/fact.

For example, this is rejected:

```text
always(PosixFs::exists(path) -> eventually(Send(response)))
```

because it performs a capability observation inside the monitor formula.

This is accepted if the observation has already been recorded as a fact:

```text
always(FileExistsObserved(path, true) -> eventually(Send(response)))
```

## 13. Operational semantics

A trace monitor is installed at a scope boundary and consumes facts emitted inside that scope:

```text
enter monitored scope
  -> initialize monitor state
  -> ordinary Core/CPS execution emits trace facts
  -> monitor consumes facts incrementally
  -> monitor reports satisfied/pending/violated/faulted at scope close or deadline
```

The monitor is sidecar state. It should not force the core language to grow separate `Proc` and `Workflow` expression families. Core needs to preserve enough metadata for later lowering and runtime monitoring:

```text
RecordTraceFact
RecordWorkflowFact
InstallMonitor
RecordMonitorResult
Trap TemporalContractViolation
```

These names are schematic IR operations/sidecars, not necessarily final term constructors.

## 14. Worked examples

### 14.1 Request-response liveness

Pseudo-temporal notation:

```text
always(Receive(Request(id)) -> eventually(Send(Response(id))))
```

Interpretation:

- `Receive(Request(id))` and `Send(Response(id))` are channel facts.
- The contract is `Proc`-like because it only mentions operational message trace facts.
- If the monitored scope closes while a request remains unanswered, the monitor reports a temporal violation or an inconclusive pending obligation depending on the declared scope policy.

### 14.2 Restart within a deadline

```text
always(ChildFailed(pid) -> eventually_within(Restarted(pid), 5s))
```

Interpretation:

- `ChildFailed` starts a timer obligation.
- `Restarted` discharges that timer obligation.
- Deadline expiry produces `TemporalContractViolation` by default.
- A supervisor that wants recoverable compensation must expose that compensation path explicitly.

### 14.3 Resource release after cancellation

```text
always(ResourceAcquired(r) && Cancelled(process) -> eventually(ResourceReleased(r)))
```

This is operational and safety/liveness mixed. The monitor tracks resource facts and cancellation facts. It does not call the resource manager itself.

### 14.4 Workflow approval before payment

```text
always(StageCommitted(pay_invoice) -> previously(ObligationDischarged(approve_invoice)))
```

Interpretation:

- `StageCommitted` and `ObligationDischarged` are workflow ledger facts.
- The contract is workflow-like because it mentions normative facts.
- The underlying source may be ordinary process events plus policy/evidence metadata.

### 14.5 Mixed operational and normative contract

```text
always(Receive(Invoice(i)) -> eventually(ObligationOpened(approve_invoice(i))))
```

This crosses anchors. It relates an operational message fact to a workflow obligation fact. That is valid in the ambient model: the formula's alphabet is mixed, so the contract interpretation is mixed.

## 15. Relation to earlier contract notes

- NOTE-027 blame applies when the violation has a responsible party or actor. Temporal diagnostics may use actor/process/stage blame rather than caller/callee blame.
- NOTE-029 structured bottom remains the default terminal path. Trace violations use a temporal diagnostic payload rather than overloading value-predicate diagnostics.
- NOTE-030 bind composition remains the sequential value/action story. Trace contracts describe behavior across longer event scopes.
- NOTE-031/NOTE-033 predicate lowering remains the value-predicate story. Temporal formulas lower to trace-contract artifacts, not `LoweredPredicate`.
- NOTE-034 authority separation remains mandatory. Monitors consume facts; they do not acquire authority.

## 16. Design decisions

1. `Act`, `Proc`, and `Workflow` are removed historical development forms and reference vocabulary only.
2. Temporal/concurrent contracts are trace contracts, not separate `Proc` and `Workflow` contract systems.
3. Operational contracts mention process/channel trace facts.
4. Normative contracts mention interpreted obligation/evidence/commitment/policy facts over traces.
5. Mixed contracts are allowed when their alphabet is mixed and every fact is in scope.
6. Runtime monitors are the default discharge mechanism for temporal contracts.
7. Static proof/model checking may discharge bounded finite-state trace contracts.
8. Monitor false/violation and monitor fault are separate diagnostic classes.
9. Monitors consume trace/evidence/timer facts and do not acquire capability, process, or workflow authority.
10. Workflow lifting preserves source trace links so obligations and commitments remain auditable.

## 17. Open questions

1. **Surface syntax.** Should temporal contracts be attached through row items, declaration clauses, monitor blocks, or workflow stage clauses?
2. **Scope close policy.** Should pending liveness at scope close default to violation, inconclusive, or an explicit policy choice?
3. **Fairness assumptions.** Which fairness assumptions, if any, can a trace contract depend on, and how are they recorded as evidence?
4. **Monitor state persistence.** Which monitors survive process restart or workflow resume, and how is monitor state checkpointed?
5. **Mechanized proof boundary.** Which trace-contract fragments are intended for SMT, model checking, proof assistants, or runtime-only monitoring?
6. **Diagnostic blame.** How should caller/callee blame from value contracts compose with actor/process/stage blame from trace contracts?

## 18. References

### Internal references

- **NOTE-014 — Contract Systems Unification.** Source gap register; GAP 5 is resolved by this note.
  `docs/notes/NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md`
- **NOTE-029 — Structured Bottom and Contract Diagnostics.** Defines trap-by-default contract diagnostics.
  `docs/notes/NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md`
- **NOTE-033 — Surface-to-Core Contract Lowering.** Defines value-predicate lowering; NOTE-035 defines the parallel trace-contract lowering target.
  `docs/notes/NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md`
- **NOTE-034 — Contract ↔ Capability Boundary.** Establishes that contract evaluators consume recorded facts rather than acquiring authority.
  `docs/notes/NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md`
- **SPEC-096b — Target Effect System.** Owns row item taxonomy and contract/evidence row syntax.
  `docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md`
- **SPEC-098b — Target IR.** Owns sidecar records and trap diagnostic payloads.
  `docs/spec/SPEC-098b-TARGET-IR.md`
- **SPEC-099 — Core Language.** Owns Core trace-recording and dynamic-check boundaries.
  `docs/spec/SPEC-099-CORE-LANGUAGE.md`
- **SPEC-100 — Core Type Checking.** Owns obligation generation and discharge checking.
  `docs/spec/SPEC-100-CORE-TYPE-CHECKING.md`

### External references

- **Amir Pnueli, "The Temporal Logic of Programs"** (1977). Classic source for using temporal logic to specify program behavior over time.
  https://doi.org/10.1109/SFCS.1977.32
- **Leslie Lamport, TLA+ resources.** TLA+ is a high-level language for modeling concurrent and distributed systems; useful prior art for distinguishing operational traces from temporal properties.
  https://lamport.azurewebsites.net/tla/tla.html
- **Runtime Verification community resources.** Prior art for checking temporal properties over execution traces at runtime.
  https://runtime-verification.github.io/

## 19. Changelog

| Date       | Change |
|------------|--------|
| 2026-06-29 | Initial note. Resolves NOTE-014 GAP 5 by defining trace contracts over the ambient computation model, treating `Pure`/`Act`/`Proc`/`Workflow` as semantic anchors, separating operational trace facts from workflow ledger facts, and specifying monitor discharge, temporal diagnostics, workflow lifting, authority boundaries, and worked examples. |
| 2026-07-06 | Reconciled with Phase 195: `Act`, `Proc`, and `Workflow` are removed historical development forms and reference vocabulary only; active guidance uses operational process/channel facts and normative ledger facts over ambient computations. |
