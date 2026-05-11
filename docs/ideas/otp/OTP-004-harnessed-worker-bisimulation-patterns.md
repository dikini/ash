# OTP-004: Harnessed Worker / Bisimulation-like Control Patterns for Ash

**Status:** Drafting
**Related:** [OTP-003 GenServer-like Design Patterns](OTP-003-genserver-design-patterns.md), [SPEC-048 Proc Library](../../spec/SPEC-048-PROC-LIBRARY.md), [SPEC-049 Process Runtime Semantics](../../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050 Operational Bottom and Scoped Handling](../../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-052 Capability Interfaces and Implementations](../../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md), [SPEC-053 Runtime Resources and Authority Provenance](../../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md), [SPEC-054 Generalized Typed Do-Notation](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md), [SPEC-056 First-Class Workflow Carrier](../../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md), [NOTE-011 Type-Level Protocols](../../notes/NOTE-011-TYPE-LEVEL-PROTOCOLS-CAPABILITY-AUTHORITY-AND-DISTRIBUTED-PARTICIPANTS.md)
**Last revised:** 2026-05-11
**Audience:** Ash language/runtime designers, stdlib authors, example authors, and future conformance-test authors.

---

## 1. Summary

This note explores Ash patterns for systems with **bisimulation-like mechanics**: two communicating processes evolve related states, where one process performs work and the other controls or verifies that work. The motivating target is an **LLM workflow control harness expressed in pure Ash**. The worker does not need real intelligence; it can be deterministic, random, or deliberately faulty. The important substrate is the control relationship: proposal, authorization, execution, evidence, verification, commit, rejection, repair, and trace comparison.

Where [OTP-003](OTP-003-genserver-design-patterns.md) uses GenServer-like examples to explore stateful service decomposition, this note uses harnessed worker examples to explore **verified process decomposition**.

Working thesis:

> A GenServer demonstrates stateful service decomposition. A harnessed worker demonstrates verified process decomposition.

The likely Ash-native core is:

```text
reference product semantics
  -> lockstep Proc protocol
  -> Workflow-governed harness
```

Other variants are adapters: shadow models, evidence-carrying workers, capability membranes, event-log replay, typed protocol descriptors, supervisor-style restart, and N-version differential execution.

All examples below are **design sketches**, not guaranteed accepted Ash syntax. Before promotion to a spec or executable example corpus, examples must be rewritten against the live parser/typechecker surface.

---

## 2. Motivation

The harnessed-worker pattern is useful for several Ash goals.

1. **LLM harness modeling without an LLM.** A dumb worker can propose edits, actions, or results. The Ash controller verifies or rejects them. This isolates harness semantics from model quality.
2. **Communicating process examples.** The pattern naturally uses two processes with distinct responsibilities and communication protocols.
3. **Control and verification first.** It tests whether Ash can represent policy, evidence, obligations, provenance, and failure/retry loops as ordinary workflow/process structure.
4. **Differential and bisimulation tests.** A single scenario can be implemented as a product-state reference machine, a lockstep two-process protocol, a weak trace/replay verifier, and a workflow-governed harness. These should agree on accepted traces.
5. **Memory/process-boundary pedagogy.** The worker owns execution state; the controller owns expected model, policy, and obligations. This teaches state projection rather than one monolithic workflow retaining everything.

---

## 3. Core model: two related state machines

At the abstract level:

```text
Worker:
  WState --worker_step--> WState' + Output + Evidence

Controller:
  CState --control_check(Output, Evidence)--> CState' + Verdict
```

The relation of interest is:

```text
R(CState, WState)
```

The harness wants to preserve:

```text
R(CState, WState)
worker emits step/output/evidence
controller accepts that step/output/evidence
------------------------------------------------
R(CState', WState')
```

For an LLM-like harness:

| Concept | Harnessed-worker interpretation |
|---|---|
| LLM/agent | Worker process |
| Control harness | Controller process / workflow |
| Prompt/task | Challenge or task spec |
| Tool call | Worker request mediated by controller |
| Chain-of-thought / transcript | Evidence, trace, or non-authoritative explanation |
| Verifier/judge | Controller check function |
| Patch/test transcript | Evidence-carrying report |
| Retry/repair | Controller verdict |
| Safety policy | Controller state + capability membrane |

The worker need not be trusted and need not be intelligent.

---

## 4. Shared vocabulary

A common event vocabulary makes variants comparable.

```ash
pub type WorkStep =
  ProposeAction { action: Action }
| ExecuteAction { action: Action }
| ReportResult { result: ResultValue, evidence: Evidence }
| RequestHelp { reason: String }
| Finish { final: ResultValue };

pub type ControlVerdict =
  Authorize { step_id: StepId }
| Reject { reason: String }
| Repair { patch: RepairInstruction }
| Retry { budget: Int }
| Stop { reason: String };

pub type HarnessEvent =
  WorkerProposed { step_id: StepId, action: Action }
| ControllerAuthorized { step_id: StepId }
| WorkerReported { step_id: StepId, result: ResultValue, evidence: Evidence }
| ControllerAccepted { step_id: StepId }
| ControllerRejected { step_id: StepId, reason: String };
```

Useful state carriers:

```ash
pub type WorkerState =
  WorkerState {
    phase: WorkerPhase,
    local_plan: Plan,
    artifact: Artifact,
    step_count: Int,
  };

pub type ControlState =
  ControlState {
    spec: TaskSpec,
    expected: ExpectedModel,
    obligations: ObligationSet,
    budget: Budget,
    accepted_trace: Trace,
  };
```

The relation `R` is usually not exact equality. Common relation shapes:

```text
project_worker(worker_state) == controller.expected
controller.expected covers worker_claim
trace(worker) conforms_to protocol(controller)
evidence_valid(spec, output, evidence)
policy_allows(control, requested_effect)
```

---

## 5. Pattern A: product-state reference semantics

### Idea

Model the worker and controller as one product machine. This establishes reference behavior before introducing real processes.

```text
(CState, WState) -> (CState', WState', TraceEvent)
```

### Sketch

```ash
pub fn harness_reference(worker: WorkerState, control: ControlState) -> Act<HarnessResult> {
  let proposal = worker_propose(worker);
  let verdict = controller_check_proposal(control, proposal);

  match verdict {
    Authorize { step_id } => {
      let report = worker_execute(worker, proposal);
      let checked = controller_check_report(control, report);

      match checked {
        Accept { next_control } => {
          let next_worker = worker_commit(worker, report);
          harness_reference(next_worker, next_control)
        }

        Reject { reason } => {
          fail reason
        }
      }
    }

    Reject { reason } => {
      fail reason
    }
  }
}
```

### What this exercises

- Pure/Act transition functions.
- Reference trace semantics.
- Relation-preservation properties.
- Differential baseline for later process variants.

### Benefits

- Easiest to specify and test.
- No channel/mailbox ambiguity.
- Good first artifact for a future conformance model.
- Provides a baseline trace for process implementations.

### Gaps

- Does not test process communication.
- Does not demonstrate process-boundary memory benefits.
- Less realistic for LLM harness runtime structure.

### Recommended role

Use this as the **reference model**. Process variants should be compared against it.

---

## 6. Pattern B: lockstep two-process protocol

### Idea

Worker and controller are separate `Proc`s. They communicate every step.

Protocol skeleton:

```text
Controller -> Worker: Challenge / Task / Authorization
Worker -> Controller: Proposal
Controller -> Worker: Authorize or Reject
Worker -> Controller: Result + Evidence
Controller -> Worker: Commit / Repair / Stop
```

### Worker sketch

```ash
pub fn worker_proc(initial: WorkerState, chan: WorkerEndpoint) -> Proc<WorkerDone> {
  do:Proc {
    challenge <- harness::recv_challenge(chan);

    proposal <- proc::from_act(worker_propose(initial, challenge));
    _ <- harness::send_proposal(chan, proposal);

    verdict <- harness::recv_verdict(chan);

    match verdict {
      Authorize { step_id } => {
        report <- proc::from_act(worker_execute(initial, proposal));
        _ <- harness::send_report(chan, report);
        commit <- harness::recv_commit(chan);
        next <- proc::from_act(worker_apply_commit(initial, commit));
        return worker_loop(next, chan)
      }

      Reject { reason } => {
        return WorkerStopped { reason }
      }
    }
  }
}
```

### Controller sketch

```ash
pub fn controller_proc(initial: ControlState, chan: ControlEndpoint) -> Proc<ControlDone> {
  do:Proc {
    _ <- harness::send_challenge(chan, next_challenge(initial));

    proposal <- harness::recv_proposal(chan);
    verdict <- proc::from_act(controller_check_proposal(initial, proposal));
    _ <- harness::send_verdict(chan, verdict);

    match verdict {
      Authorize { step_id } => {
        report <- harness::recv_report(chan);
        checked <- proc::from_act(controller_check_report(initial, report));

        match checked {
          Accept { next_control } => {
            _ <- harness::send_commit(chan, Commit { step_id });
            return controller_loop(next_control, chan)
          }

          Reject { reason } => {
            _ <- harness::send_commit(chan, Stop { reason });
            return ControlRejected { reason }
          }
        }
      }

      Reject { reason } => {
        return ControlRejected { reason }
      }
    }
  }
}
```

### What this exercises

- `Proc` communication.
- Channel/endpoint typing.
- Process-local state.
- Authorization and commit protocol.
- Failure/rejection routing.

### Benefits

- Directly models a harnessed LLM workflow.
- Good example for communicating processes.
- Clear trace equivalence with the product-state model.
- Easy to inject faulty worker behavior and verify controller rejection.

### Gaps

- Requires endpoint/channel/mailbox substrate.
- Needs timeout/deadlock behavior.
- Need to decide whether the controller sees worker state directly or only projected claims/evidence.
- Strong lockstep can be too restrictive for some realistic workers.

---

## 7. Pattern C: controller shadow model

### Idea

The controller maintains an independent expected model and checks worker reports against it.

```text
worker state: actual/proposed execution state
controller state: reference model / expected state
```

Worker reports:

```text
action, result, state_digest, evidence
```

Controller computes:

```text
expected_result, expected_digest
```

Accept iff:

```text
result == expected_result
and digest/projection matches
and evidence is valid
```

### Sketch

```ash
pub fn controller_check_report(
  control: ControlState,
  report: WorkReport,
) -> Act<CheckResult> {
  let expected = reference_step(control.expected, report.action);

  if report.result == expected.result
     && report.worker_digest == expected.digest {
    return Accept {
      next_control: ControlState {
        expected: expected.next_model,
        accepted_trace: append(control.accepted_trace, report),
        obligations: discharge(control.obligations, report.evidence),
        budget: spend(control.budget, report.cost),
      }
    }
  } else {
    return Reject {
      reason: explain_mismatch(expected, report)
    }
  }
}
```

### What this exercises

- Differential execution.
- State projection and digest checking.
- Reference-model conformance.
- Trace acceptance.

### Benefits

- The controller does not need to trust the worker.
- Good fit for verification harnesses.
- Excellent for negative tests: faulty worker claims can be rejected.
- Does not require the controller to inspect private worker state.

### Gaps

- Requires a reference model.
- Some realistic tasks cannot be cheaply replayed by the controller.
- Digest/projection design must be honest about what is and is not checked.
- For LLM workflows, verification is often partial, not full equality.

---

## 8. Pattern D: proof-carrying / evidence-carrying worker

### Idea

The worker sends not just a result, but evidence that the controller can check.

```text
Worker: result + evidence
Controller: cheap evidence checker
```

Evidence examples:

```ash
pub type Evidence =
  SortedEvidence { permutation: Permutation, pairwise_order: OrderWitness }
| ArithmeticEvidence { derivation: DerivationTrace }
| ToolEvidence { transcript: ToolTrace, output_hash: Hash }
| InvariantEvidence { before: Digest, after: Digest, obligations: ObligationProof };
```

### Sketch

```ash
pub fn worker_step(state: WorkerState, task: Task) -> Act<WorkReport> {
  let result = perform_work(state, task);
  let evidence = build_evidence(state, task, result);

  return WorkReport {
    task,
    result,
    evidence,
    digest: digest(project_state(state)),
  }
}

pub fn controller_verify(control: ControlState, report: WorkReport) -> Act<ControlVerdict> {
  if evidence_valid(control.spec, report.result, report.evidence) {
    return AuthorizeCommit { next: update_expected(control, report) }
  } else {
    return Reject { reason: "invalid evidence" }
  }
}
```

### What this exercises

- Evidence schemas.
- Obligation discharge.
- Provenance and trace validation.
- Distinction between explanation and checkable evidence.

### Benefits

- Strong bridge to LLM harnesses: “show your work,” “provide a transcript,” “include patch + tests.”
- Verification can be cheaper than generation.
- Good for tasks where full re-execution is expensive but evidence is checkable.
- Natural fit for Ash obligations/provenance.

### Gaps

- Need evidence schemas per task family.
- Natural-language explanations should not count as authoritative evidence by default.
- Need provenance semantics for capability/tool calls.
- Need to decide which evidence is trusted, checked, replayed, or merely diagnostic.

---

## 9. Pattern E: capability membrane / harness-mediated effects

### Idea

The worker cannot perform effects directly. It must request permission from the controller. The controller grants narrow authority, performs the effect on the worker's behalf, or denies the request.

```text
Worker -> Controller: RequestEffect(action)
Controller checks policy/invariants
Controller -> Worker: EffectGranted or EffectDenied
Worker proceeds only with granted effect/result
```

### Sketch

```ash
pub type WorkerRequest =
  NeedRead { path: Path }
| NeedWrite { path: Path, content: String }
| NeedModelCall { prompt: Prompt }
| NeedTool { name: String, args: Args };

pub fn worker_proc(state: WorkerState, chan: HarnessChannel) -> Proc<WorkerDone> {
  do:Proc {
    req <- proc::from_act(worker_next_request(state));
    _ <- harness::send_request(chan, req);

    grant <- harness::recv_grant(chan);

    match grant {
      Granted { token } => {
        result <- harness::perform_granted_effect(token);
        next <- proc::from_act(worker_observe_result(state, result));
        return worker_loop(next, chan)
      }

      Denied { reason } => {
        next <- proc::from_act(worker_repair_plan(state, reason));
        return worker_loop(next, chan)
      }
    }
  }
}
```

Controller side:

```ash
pub fn controller_proc(control: ControlState, chan: HarnessChannel) -> Proc<ControlDone> {
  do:Proc {
    req <- harness::recv_request(chan);
    verdict <- proc::from_act(check_policy(control, req));

    match verdict {
      Allow { narrowed_capability } => {
        token <- harness::grant(req, narrowed_capability);
        _ <- harness::send_grant(chan, Granted { token });
        return controller_loop(update_control(control, req), chan)
      }

      Deny { reason } => {
        _ <- harness::send_grant(chan, Denied { reason });
        return controller_loop(record_denial(control, req), chan)
      }
    }
  }
}
```

### What this exercises

- Capability authority.
- Resource access discipline.
- Policy checks.
- Narrow temporary grants.
- Revocation/denial behavior.

### Benefits

- Very Ash-native.
- Closely models LLM tool-use harnesses.
- Worker cannot bypass the harness if it lacks direct authority.
- Excellent for safety examples.

### Gaps

- Need clear distinction between worker local computation, controller authority, granted tokens, and runtime capability bindings.
- Need expiration/revocation semantics for grants.
- Need to choose whether controller performs effects or delegates narrowed authority.
- More complex than post-hoc verification.

---

## 10. Pattern F: event-sourced audit log + replay verifier

### Idea

The worker appends events to a log. The controller consumes, replays, and checks the event stream. This supports weaker/asynchronous simulation.

```text
Worker -> append event
Controller -> consume/replay/check event stream
```

### Sketch

```ash
pub type WorkLogEvent =
  Planned { action: Action }
| Performed { action: Action, result: ResultValue }
| ClaimedState { digest: Digest }
| Completed { final: ResultValue };

pub fn worker_proc(state: WorkerState, log: LogEndpoint) -> Proc<WorkerDone> {
  do:Proc {
    action <- proc::from_act(worker_choose_action(state));
    _ <- log::append(log, Planned { action });

    result <- proc::from_act(worker_perform(action));
    _ <- log::append(log, Performed { action, result });

    next <- proc::from_act(worker_update(state, result));
    _ <- log::append(log, ClaimedState { digest: digest(next) });

    return worker_loop(next, log)
  }
}

pub fn verifier_proc(model: ExpectedModel, log: LogEndpoint) -> Proc<VerifierDone> {
  do:Proc {
    event <- log::next(log);
    next_model <- proc::from_act(replay_and_check(model, event));
    return verifier_loop(next_model, log)
  }
}
```

### What this exercises

- Append-only traces.
- Replay verification.
- Weak trace simulation.
- Async worker/controller decoupling.

### Benefits

- Good for audit/provenance examples.
- Controller can lag behind worker.
- Natural debugging and differential-test substrate.
- Internal worker steps can be hidden; only observable events are compared.

### Gaps

- Worker may do bad work before rejection if verification lags.
- Need log ordering/durability semantics.
- Need rollback, compensation, or quarantine policy after rejection.
- Weaker control than lockstep authorization.

---

## 11. Pattern G: Workflow contract over worker/controller pair

### Idea

At the workflow layer, model the whole harness as a governed process composition.

```text
Workflow = contract + Proc(worker || controller)
```

### Sketch

```ash
pub workflow controlled_worker(task: TaskSpec) -> VerifiedResult {
  requires: task_is_well_formed(task);
  ensures: result_satisfies_spec(result, task);

  do:Workflow {
    pair <- workflow::from_proc(harness_pair(task));
    result <- workflow::from_proc(harness::await_verified(pair));
    ensures: trace_verified(result.trace);
    return result.value
  }
}
```

Pair construction:

```ash
pub fn harness_pair(task: TaskSpec) -> Proc<HarnessHandle> {
  do:Proc {
    channels <- harness::new_channels();

    worker <- proc::spawn(worker_proc(initial_worker(task), channels.worker));
    control <- proc::spawn(controller_proc(initial_control(task), channels.control));

    return HarnessHandle { worker, control, channels }
  }
}
```

### What this exercises

- Workflow as governed `Proc` envelope.
- `requires`/`ensures` over process-pair behavior.
- Lower process failure reinterpretation at workflow boundary.
- Report/provenance obligations for accepted traces.

### Benefits

- Shows the tower split clearly: `Proc` owns process mechanics; `Workflow` owns governance.
- Good top-level shape for an Ash LLM harness.
- Lets the same process harness appear raw or governed.

### Gaps

- Workflow contracts over long-lived process handles are subtle.
- Need latent handle obligations or monitor contracts.
- Need to define when `ensures` is checked: startup, final result, every committed step, or monitor stream.
- Need to avoid reporting workflow success while child process failures remain unresolved.

---

## 12. Pattern H: typed protocol / session-style harness

### Idea

Encode the worker/controller conversation as a protocol descriptor. The descriptor can later drive endpoint generation, trace checking, diagrams, or full session typing.

### Sketch

```ash
pub protocol HarnessProtocol {
  controller -> worker: Challenge { task: TaskSpec };

  worker -> controller: Proposal { action: Action };

  controller -> worker:
    Authorize { step_id: StepId }
  | Reject { reason: String };

  worker -> controller:
    Report { step_id: StepId, result: ResultValue, evidence: Evidence };

  controller -> worker:
    Commit { step_id: StepId }
  | Repair { instruction: RepairInstruction }
  | Stop { reason: String };
}
```

### What this exercises

- Protocol descriptors.
- Endpoint role/projection concepts.
- Conversation conformance.
- Future distributed/sandboxed participant design.

### Benefits

- Prevents invalid conversations such as report-before-authorization or commit-after-stop.
- Good bridge to type-level protocol design.
- Can generate documentation and test traces.
- Useful for distributed Ash nodes and LLM/tool participants.

### Gaps

- Full session typing likely needs linear/affine endpoint-state machinery.
- Near-term work should likely start as descriptor + runtime trace checker, not full MPST.
- Need to decide what the typechecker proves vs what trace verifier checks.

---

## 13. Pattern I: controller as semantic supervisor

### Idea

The controller supervises the worker, but the restart policy is semantic: not just “worker crashed,” but “worker produced invalid work.”

### Sketch

```ash
pub fn controller_proc(task: TaskSpec) -> Proc<VerifiedResult> {
  do:Proc {
    worker <- proc::spawn(worker_proc(initial_worker(task)));

    result <- harness::monitor_steps(worker, initial_control(task));

    match result {
      Verified { value } => return value,

      Mismatch { reason, checkpoint } => {
        repaired <- harness::restart_from(worker, checkpoint);
        return continue_with(repaired)
      }
    }
  }
}
```

### What this exercises

- Process observation.
- Restart and repair policy.
- Checkpointing.
- Differentiating runtime failure from verification failure.

### Benefits

- Connects this note back to OTP-style supervision.
- Models realistic LLM harness repair loops.
- Good for failure-injection tests.

### Gaps

- Need links/monitors or observation handles.
- Need checkpoint/restart semantics.
- Need to distinguish worker crash, invalid proposal, invalid result, timeout, verifier uncertainty, and controller failure.
- Restart must not preserve invalid authority or stale state accidentally.

---

## 14. Pattern J: N-version / differential worker harness

### Idea

Run multiple workers or a worker plus reference implementation, then compare results.

```text
Worker A result
Worker B result
Reference model result
Controller compares all
```

### Sketch

```ash
pub fn differential_harness(task: TaskSpec) -> Proc<VerifiedResult> {
  do:Proc {
    wa <- proc::spawn(worker_a(task));
    wb <- proc::spawn(worker_b(task));
    ref <- proc::spawn(reference_worker(task));

    results <- proc::gather([wa, wb, ref]);
    verdict <- proc::from_act(compare_results(task, results));

    match verdict {
      Agree { value } => return VerifiedResult { value },
      Disagree { report } => fail report,
    }
  }
}
```

### What this exercises

- `par` / `scatter` / `gather`.
- Aggregate process failure.
- Differential conformance.
- Multi-worker judging patterns.

### Benefits

- Good stress test for process runtime.
- Models multi-agent LLM harnesses: worker, critic, reference checker, judge.
- Useful when no single verifier is complete.

### Gaps

- Agreement is not truth unless one participant is trusted.
- Expensive.
- Correlated implementation bugs can still agree.
- Need robust aggregation/failure semantics.

---

## 15. Strong and weak bisimulation styles

This note uses “bisimulation-like” broadly. We should name the variants precisely when promoting to spec or tests.

### 15.1 Strong lockstep simulation

Every worker step has a controller step.

```text
W --a--> W'
C --a/check--> C'
R(C, W) => R(C', W')
```

Best for:

- authorization;
- safe tool use;
- deterministic reference examples;
- simple process-communication tests.

### 15.2 Weak trace simulation

Worker may do internal steps before observable checkpoints.

```text
W --τ* a τ*--> W'
C --a/check--> C'
R(C, W) => R(C', W')
```

Best for:

- event logs;
- asynchronous verification;
- realistic agent execution;
- audit/replay.

### 15.3 Forward simulation

Controller has a spec model; worker must conform to it.

```text
project(WState) conforms_to CState.expected
```

Best for:

- LLM-like execution control;
- partial correctness;
- verifier/checker harnesses.

### 15.4 Symmetric bisimulation

Both sides can challenge each other and either side can reject divergence. This is useful for peer protocols and consensus-like examples, but is probably too heavy for the initial LLM-harness use case.

---

## 16. Candidate example domains

The worker should perform useful but checkable work. It need not be intelligent.

### 16.1 Sorting worker

Worker receives a list and claims sorted output.

Controller checks:

- output is sorted;
- output is a permutation of input.

Evidence can be:

- permutation witness;
- pairwise ordering witness;
- or just output, because verification is cheap.

Good for: evidence-carrying worker, reference/product model, negative faulty-worker tests.

### 16.2 Arithmetic expression simplifier

Worker rewrites an expression.

Controller checks:

- each rewrite is legal;
- final expression is equivalent under evaluator.

Good for: stepwise proof traces, repair/reject behavior, shadow reference model.

### 16.3 Patch/edit worker

Worker edits an artifact.

Controller checks:

- patch applies;
- changed regions are allowed;
- expected invariants/tests pass;
- reported digest matches actual artifact state.

Good for: LLM coding harness analogy, capability membrane around file writes, evidence = diff + test transcript.

### 16.4 Planning worker

Worker proposes a plan.

Controller checks:

- all requirements are covered;
- no forbidden action appears;
- dependencies are ordered;
- budget constraints hold.

Good for: pure Ash harness without external effects, obligation checking, workflow contracts.

### 16.5 Tool-use worker

Worker requests tool calls.

Controller checks:

- tool is allowed;
- arguments are valid;
- result updates state consistently;
- authority was not widened.

Good for: capability membrane, LLM agent harness, policy examples.

---

## 17. Recommended first exemplar: controlled patch worker

A strong Ash-native example is a “controlled patch worker” with no actual LLM.

Task:

```text
Transform an input artifact into an output artifact.
```

Worker:

```text
proposes edits
requests permission
applies allowed edit
reports diff + state digest + evidence
```

Controller:

```text
checks edit scope
checks invariant/test
updates expected model
accepts, rejects, repairs, or stops
```

Protocol:

```text
Controller -> Worker: TaskSpec
Worker -> Controller: ProposedEdit
Controller -> Worker: EditAllowed | EditDenied
Worker -> Controller: EditResult { diff, digest, evidence }
Controller -> Worker: Commit | Repair | Stop
```

Mapping to an LLM coding harness:

| LLM harness concept | Pure Ash controlled patch worker |
|---|---|
| LLM proposes patch | Dumb worker proposes edit |
| Harness checks scope | Controller checks allowed region |
| Harness runs tests | Controller runs pure invariant/checker |
| LLM receives feedback | Worker receives `Repair` |
| Harness accepts final | Controller returns `VerifiedResult` |

This example can start simple and grow into capability-mediated file/tool authority later.

---

## 18. Comparison with GenServer-like patterns

| Aspect | GenServer-like pattern | Harnessed-worker pattern |
|---|---|---|
| Main relation | client(s) ↔ server | controller ↔ worker |
| Trust model | server trusted to own/update state | worker not fully trusted |
| Generic logic | message loop routes calls/casts | harness controls proposal/check/commit |
| User logic | callback/reducer handles requests | worker proposes/executes; controller verifies |
| State relation | server has authoritative state | relation `R(CState, WState)` must be preserved |
| Failure focus | server crash/restart | invalid work, failed evidence, unauthorized effect, crash/restart |
| Best examples | cache, counter, router, registry | patch worker, verifier, tool-use harness, proof-carrying worker |

GenServer demonstrates decomposition into stateful services. Harnessed workers demonstrate decomposition into controlled, verified process pairs.

---

## 19. Ash feature coverage matrix

| Variant | Main Ash features exercised | Good for |
|---|---|---|
| Product-state reference model | pure/Act state transitions | baseline semantics |
| Lockstep two-process protocol | `Proc`, channels, call/reply, failure | process communication |
| Shadow controller | pure reference model, digest/projection relation | differential testing |
| Evidence-carrying worker | obligations, provenance, checkers | LLM “show work” harness |
| Capability membrane | capability authority, resource access, policy | tool-use safety |
| Event log/replay | append-only traces, replay verification | audit/provenance |
| Workflow-governed pair | `Workflow<A>` over `Proc` pair | governed harness |
| Typed protocol | protocol descriptors/session ideas | future distributed/sandboxed participants |
| Semantic supervisor | process failure/restart/monitor | robust harnesses |
| N-version differential | `par`, `gather`, aggregate failure | multi-worker evals |

---

## 20. Differential and bisimulation-style test plan

### 20.1 Canonical trace format

Candidate trace events:

```text
HarnessStarted(run_id, task)
ChallengeSent(step_id, task_projection)
WorkerProposed(step_id, action)
ControllerAuthorized(step_id)
WorkerReported(step_id, result, evidence_digest, state_digest)
ControllerAccepted(step_id)
ControllerRejected(step_id, reason)
RepairSent(step_id, instruction)
WorkerStopped(reason)
HarnessCompleted(result_digest)
```

Each event should eventually carry:

```text
event_id
actor
process_id
step_id
message/action
pre_digest
post_digest
evidence_digest
verdict
```

### 20.2 Comparable implementations

A future example/test packet should implement the same scenario as:

1. Product-state reference semantics.
2. Lockstep worker/controller processes.
3. Evidence-carrying worker.
4. Capability-membrane worker.
5. Event-log replay verifier.
6. Workflow-governed harness.
7. N-version differential harness.

### 20.3 Properties

- Accepted traces match the reference model modulo internal event labels.
- Worker cannot commit a step not accepted by controller.
- Rejected steps do not update controller accepted trace.
- Unauthorized effects cannot occur in membrane variants.
- Evidence-carrying variants reject invalid evidence.
- Replay variants reject traces that violate order or invariant checks.
- Workflow-governed variants do not widen authority relative to declared requirements.

### 20.4 Failure injection cases

- Worker proposes forbidden action.
- Worker reports result inconsistent with action.
- Worker reports valid result with invalid evidence.
- Worker times out after authorization.
- Controller cannot decide and returns inconclusive verdict.
- Capability request is denied.
- Worker crashes before report.
- Worker crashes after report before commit.
- Restarted worker tries to reuse stale grant token.

---

## 21. Current gaps and questions

### 21.1 Process/channel substrate

- What is the exact taxonomy of `P<A>`, communication endpoint, mailbox/channel resource, monitor handle, and harness handle?
- Are worker/controller channels linear, affine, duplicable, capability-backed, resource-backed, or some combination?
- What are the ordering, timeout, cancellation, and backpressure semantics?

### 21.2 State visibility

Possible controller visibility models:

1. direct shared state — simple but weak isolation;
2. worker sends full state — expensive and potentially leaky;
3. worker sends projection/digest — realistic and checkable;
4. worker sends evidence — best for harness semantics.

For LLM-harness modeling, prefer **projection + evidence**, not direct state access.

### 21.3 Verification relation

Need to distinguish:

```text
exact state equality
projected state equality
trace conformance
invariant preservation
evidence-check validity
policy admissibility
obligation discharge
```

These should not collapse into a single generic “verified” boolean.

### 21.4 Failure taxonomy

Controller should distinguish:

```text
worker runtime failure
worker invalid proposal
worker invalid result
worker timeout
controller/checker failure
harness protocol violation
capability/resource denial
verification inconclusive
```

These should not all become one `Err`.

### 21.5 Authority and capability membranes

- Should the controller perform effects on behalf of the worker, or grant narrowed temporary authority?
- How are grants scoped, consumed, expired, and revoked?
- How does a restarted worker avoid inheriting stale authority?
- Is every effect request a controller message, or can some pre-admitted safe effects bypass lockstep control?

### 21.6 Workflow contracts over process pairs

- Does `ensures` apply only to final `VerifiedResult`, or to every accepted step?
- Do harness handles need latent contracts?
- How are later child process failures represented in workflow reports?
- How does the workflow avoid claiming success before verifier/controller termination is known?

### 21.7 Protocol descriptors

- Should the first slice be protocol descriptor + runtime trace checker rather than full session typing?
- What linear/affine endpoint machinery is necessary for type-level conversation state?
- How do protocol descriptors interact with distributed nodes, sandboxed workers, and external LLM/tool participants?

---

## 22. Suggested next artifacts

1. **Reference model note:** Define a tiny product-state semantics for a controlled sorting or controlled patch worker.
2. **Trace schema:** Define canonical `HarnessEvent` fields for differential/bisimulation-style tests.
3. **Endpoint taxonomy note:** Separate `P<A>`, communication endpoints, server handles, monitor handles, and capability grants.
4. **Example packet:** Implement the same controlled sorting scenario as product-state, lockstep two-process, and evidence-carrying variants.
5. **Future spec candidate:** A narrow `harness::lockstep` / `harness::verify` stdlib design over `Proc`, explicitly non-normative about full session types.

---

## 23. Working recommendation

Start with three layers:

### Layer 1: reference product semantics

```text
step_pair : (CState, WState) -> (CState, WState, TraceEvent)
```

This is the baseline.

### Layer 2: lockstep Proc protocol

```text
worker_proc || controller_proc
```

The controller authorizes and commits worker steps.

### Layer 3: workflow-governed harness

```text
Workflow<VerifiedResult>
  = WorkflowContract + Proc<VerifiedResult>
```

The contract states:

```text
requires task_is_valid
ensures result_is_verified
ensures trace_conforms_to_harness_protocol
ensures no_unauthorized_effects
```

Treat other variants as adapters:

```text
event log      = weak/asynchronous controller observation
evidence       = cheaper check function
cap membrane   = stronger authority control
typed protocol = static conversation discipline
supervisor     = restart/failure extension
N-version      = differential confidence extension
```

The first concrete example should probably be either:

1. **controlled sorting**, because it is small and verifier-complete; or
2. **controlled patch worker**, because it better models an LLM coding harness.

A good sequence is sorting first for machinery, then patch worker for realism.
