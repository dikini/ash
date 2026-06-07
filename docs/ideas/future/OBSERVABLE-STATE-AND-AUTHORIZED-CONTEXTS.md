---
status: drafting
created: 2026-06-07
last-revised: 2026-06-07
related-plan-tasks: []
tags:
  - observability
  - comonad
  - policy
  - proc
  - workflow
  - runtime-state
---

# FUTURE-006: Observable state and authorized contexts

## Problem statement

Ash already treats execution as a tower of increasing power: `Pure < Act < Proc < Workflow`.
That tower explains how programs transform values, perform effects, compose processes, and govern workflows.
The open question in this note is the dual direction: how an Ash program, tool, workflow, or external observer may observe runtime state without breaking opacity, policy, or authority boundaries.

This note explores the idea that runtime objects can expose explicitly authorized observable views.
Those views may later support comonadic/contextual composition: not by making `Act`, `Proc`, or `Workflow` themselves comonads, but by making point-in-time observation contexts lawfully inspectable.
The idea is raw. It is not a spec, not a syntax proposal, and not an implementation plan.

## Scope

- **In scope:**
  - Observable state as a possible semantic dimension of values.
  - Explicit observable view declarations as a more concrete mechanism.
  - Observer-subject-view authorization, policy, and redaction.
  - Mailboxes, `Act`, `Proc`, and `Workflow` as runtime subjects with point-in-time observable states.
  - Comonadic interpretation over authorized observation contexts.

- **Out of scope:**
  - New Ash surface syntax.
  - A normative `Observable` or `Comonad` implementation.
  - A policy language redesign.
  - Runtime debugger, monitor, or UI implementation.
  - A claim that raw `Mailbox`, `Act`, `Proc`, or `Workflow` is a lawful comonad.

- **Related but separate:**
  - Standard algebra `Comonad`/Kleisli work.
  - RuntimeKernel reports and canonical runtime-observable result formats.
  - Workflow reporting and obligation evidence.
  - Process supervision and OTP-like patterns.
  - Small-world/test-runner state snapshots.

## Current intuition

A value or runtime object may be hidden, partially observable, or observable through a specific view.
That view is not the object itself. It is a projection that preserves policy and opacity.

For example, a workflow might expose several views:

```text
Workflow W observed by owner       -> FullWorkflowView
Workflow W observed by child proc  -> ParentStatusView
Workflow W observed by auditor     -> ProvenanceSummaryView
Workflow W observed by UI          -> ProgressView
Workflow W observed by public API  -> RedactedPublicView
```

All of these views refer to the same underlying workflow, but they are not interchangeable.
Each view reveals different facts, has different policy requirements, and may redact different internals.

The central rule is:

```text
No object is globally observable in itself.
Only declared views are observable, and only to authorized observers.
```

## Empty versus blocked

A useful starting example is a mailbox.
An empty list is extensionally empty. There is no current element, and waiting does not change that list.

A mailbox with no immediately available message is different.
It is blocked at the current time, but it may receive a message later.
The absence of an immediately available message is itself a runtime state.

That suggests this is the wrong extraction shape:

```text
extract : Mailbox<Event> -> Event
```

A blocked mailbox cannot return an `Event` now.
A better observation shape is:

```text
extract : MailboxContext<Event> -> MailboxState<Event>
```

where:

```text
MailboxState<Event> =
  | Ready(Event)
  | Blocked
  | Closed
  | Failed(OperationalFailure)
```

`Blocked` is not absence. It is the current observable state.

## Computations versus observation contexts

The same distinction applies to `Act`, `Proc`, and `Workflow`.
The computation carrier is not the observation carrier.

```text
Act<A>       = effectful computation
Proc<A>      = process computation
Workflow<A>  = governed process computation

ActContext<A>       = observation context over an effectful runtime point
ProcContext<A>      = observation context over a process runtime point
WorkflowContext<A>  = observation context over a workflow runtime point
```

The likely extraction target is not the final result `A`.
It is a point-in-time state value.

```text
extract : ActContext<A> -> ActState<A>
extract : ProcContext<A> -> ProcState<A>
extract : WorkflowContext<A> -> WorkflowState<A>
```

Possible state shapes:

```text
ActState<A> =
  | BeforeInvoke(EffectScopeId, CapabilityRef)
  | WaitingProvider(EffectScopeId, CapabilityRef)
  | Returned(A)
  | Failed(OperationalFailure)
```

```text
ProcState<A> =
  | Running(ProcessId)
  | Yielded(ProcessId)
  | Blocked(ProcessId, BlockReason)
  | WaitingOn(ProcessId, List<ProcessId>)
  | MailboxReady(ProcessId, MessageSummary)
  | Completed(ProcessId, A)
  | Failed(ProcessId, OperationalFailure)
```

```text
WorkflowState<A> =
  | Admitting(WorkflowId)
  | Running(WorkflowId, RootProcessId)
  | RequirementBlocked(WorkflowId, RequirementRef)
  | ObligationPending(WorkflowId, ObligationRef)
  | Reporting(WorkflowId)
  | Succeeded(WorkflowId, A)
  | Failed(WorkflowId, WorkflowFailure)
```

These examples are illustrative only. A future design must decide which states are stable enough to expose and which remain runtime-private.

## Observable state as a value dimension

One way to model this is to treat observability as a semantic dimension of values, similar to effect classification.
A value would have a type and an observability level.

Possible levels might include:

```text
Hidden
IdentityOnly
StatusOnly
Summary
Snapshot
Payload
Full
```

This dimension could explain rules such as:

- A process identifier may be observable while its private mailbox payload is hidden.
- A workflow may expose progress while hiding capability arguments.
- A completed process may reveal `Completed(Redacted)` unless its result type is observable to the current observer.
- A failure may expose a public failure class while hiding host paths, provider secrets, or sensitive prompts.

This is attractive because it gives the type checker and runtime a common vocabulary.
It is also expensive. It touches type checking, runtime metadata, policy, redaction, serialization, reports, and tool output.

## Explicit observable views

The more concrete first mechanism may be explicit observable views.
A view declaration would say which projection of a subject exists, who may request it, and how redaction works.

Abstractly:

```text
observable view V for Subject S
  observer: O
  requires: CanObserve(O, S, V)
  project: S -> V
```

This is not proposed Ash syntax. It is a semantic sketch.

A capability-shaped version may fit the current architecture better:

```text
RuntimeObserve.workflow_status(observer, workflow_id) -> WorkflowStatusView
RuntimeObserve.proc_status(observer, process_id) -> ProcStatusView
RuntimeObserve.mailbox_status(observer, mailbox_id) -> MailboxStatusView
```

The operation would require both:

1. capability authority to use the runtime observation gateway; and
2. an object-specific grant to observe this subject through this view.

The latter is crucial. Having a general runtime observation capability must not mean the observer can inspect every workflow or process.

## Observer-subject-view authorization

The core policy relation is ternary:

```text
CanObserve(observer, subject, view)
```

The observer can be a human principal, role, workflow, process, capability binding, external client, or tool.
The subject can be a workflow, process, mailbox, effect scope, resource, trace, or other runtime object.
The view names the exact projection being requested.

Examples:

```text
CanObserve(WorkflowOwner, WorkflowId, FullWorkflowView)
CanObserve(ChildProcess, ParentWorkflowId, ParentStatusView)
CanObserve(AuditorRole, WorkflowId, ProvenanceSummaryView)
CanObserve(UIClient, WorkflowId, PublicProgressView)
CanObserve(SupervisorProc, WorkerProc, WorkerStatusView)
```

This keeps observation object-relative and audience-relative.
The same subject may expose different views to different observers.

## Policy and capability interaction

Observation should pass through layered checks.

```text
Capability layer:
  May this actor call the observation gateway at all?

Subject policy layer:
  May this observer inspect this subject through this view?

Projection layer:
  Given permission, compute the redacted view.

Contextual algebra layer:
  Compose pure/contextual projections over the authorized view.
```

This prevents two common mistakes.

First, it prevents global introspection:

```text
has RuntimeObserve capability => can observe every workflow
```

That rule is too broad and should be rejected.

Second, it prevents view widening during derived observations.
If an observer has only `PublicProgressView`, comonadic/contextual composition over that context must not reveal `FullWorkflowView`.

## Contexts and comonads

Once an observation has been authorized, it may produce a context carrier:

```text
WorkflowContext<Observer, Subject, View>
ProcContext<Observer, Subject, View>
MailboxContext<Observer, Subject, View>
```

The context carries opaque runtime internals plus observation evidence.
The public operation is total over the authorized view:

```text
extract : WorkflowContext<O, S, V> -> V
```

A comonadic `extend`-like operation can derive new local observations from the authorized context:

```text
extend :
  WorkflowContext<O, S, V>
  -> (WorkflowContext<O, S, V> -> B)
  -> WorkflowContext<O, S, B>
```

The safety rule is:

```text
Contextual composition may preserve or narrow visibility. It must not widen visibility.
```

Derived local observations inherit the authority of their source context.
Published observations require explicit view declarations and grants.

## Local derived views versus published views

A workflow monitor might derive a local value:

```text
is_stuck : WorkflowContext<O, W, ProgressView> -> Bool
```

If the observer already has `ProgressView`, this local `Bool` is just a pure function of what the observer can see.
No new published view is necessarily created.

Publishing `StuckView` to other observers is different.
That requires a declaration:

```text
StuckView for Workflow W
  visible to: SupervisorRole, AuditorRole
  projection: derived from authorized workflow state
```

This distinction prevents accidental authority expansion.

## View preorders and noninterference

Some views may be ordered by information content:

```text
FullWorkflowView >= AuditView >= PublicProgressView
```

But the design should not assume every view fits into one neat lattice.
Some views are incomparable:

```text
BillingView
SecurityAuditView
UserProgressView
```

A safer model is explicit projection edges:

```text
FullWorkflowView -> AuditView
FullWorkflowView -> PublicProgressView
AuditView        -> PublicProgressView
```

A noninterference criterion should hold:

```text
If two hidden subject states produce the same authorized view V for observer O,
then O cannot distinguish those hidden states through any allowed observation over V.
```

This is the representation-opacity rule for observability.
It says that observable contexts must not leak hidden internals through derived projections.

## Opaque internals

Observation contexts may hide existential runtime state.
That opacity is a feature, not a compromise.

For example:

```text
ProcContext<O, P, V> = exists Internal.
  {
    observer: O,
    subject: P,
    view: V,
    evidence: ObservationEvidence,
    internal: Internal
  }
```

`Internal` may include scheduler queues, cancellation tokens, provider handles, mailbox buffers, provenance sinks, or host resources.
The observer receives only the authorized operations and view projections.

This lets Ash expose stable monitoring and reporting contracts without freezing runtime implementation details.

## Useful consequences

### Workflow reports

A workflow report can be understood as an audience-specific view over a workflow run.

```text
WorkflowReport<Audience, View>
```

Different audiences may receive different reports from the same run:

```text
operator report
user report
audit report
debug report
public report
```

The report is not merely the workflow result. It is an authorized projection of the run.

### Test runner and small-world exploration

Generated tests and small-world exploration need honest observable state.
Observable view declarations could tell the runner which state is real evidence and which state is unavailable.

This matches the existing preference for explicit `skip` or `deferred` outcomes instead of false success.

### Debugging and monitoring

A debugger or monitor could inspect authorized views:

```text
ProcState
MailboxState
ActState
CapabilityTraceView
WorkflowProgressView
ObligationProgressView
```

The tool would not need raw access to runtime internals.
It would use declared views and observation evidence.

### Reactive UI

A UI could subscribe to authorized observable contexts:

```text
Observable<WorkflowProgressView>
Observable<MailboxBacklogView>
Observable<ProcessTreeView>
```

Comonadic/contextual projections could then derive rendered views without gaining mutation authority.
Event handling would still cross back into `Act`, `Proc`, or `Workflow` through capabilities and mailboxes.

## Design dimensions

| Dimension | Option A | Option B | Option C |
| --- | --- | --- | --- |
| Observability model | Type/value dimension | Explicit view declarations | Hybrid: declarations first, type dimension later |
| Authorization shape | Capability-only | `CanObserve(observer, subject, view)` | Capability plus subject-specific grants |
| View relation | One global lattice | Explicit projection graph | Domain-specific partial orders |
| Runtime exposure | Raw internals | Stable redacted views | Opaque contexts with evidence |
| Comonadic carrier | Raw computation | Snapshot/timeline/context carrier | Multiple carrier families by focus |
| Publishing | Any derived value visible | Explicit published views only | Local derivation inherits, publication declares |

The hybrid path is the most promising.
Explicit declarations are concrete enough for early implementation, while a latent observability dimension keeps the semantics coherent.

## Candidate staged path

1. **Vocabulary note:** define subject, observer, view, observation evidence, redaction, and noninterference.
2. **Explicit view declarations:** design a declaration or capability-shaped mechanism for authorized projections.
3. **Observation evidence:** carry evidence that observer `O` may observe subject `S` through view `V`.
4. **Runtime view carriers:** define stable `ActState`, `ProcState`, `WorkflowState`, and `MailboxState` view families.
5. **Context carriers:** define `ActContext`, `ProcContext`, `WorkflowContext`, and mailbox/timeline contexts over authorized views.
6. **Comonadic profiles:** evaluate lawful `extract`/`extend` for specific contexts, not raw computations.
7. **Optional type dimension:** promote observability into type/effect-like checking only if the declaration/evidence model proves useful.

## Open questions

1. What is the minimal useful observer identity model?
   Should observer identity start with `WorkflowId` and `ProcessId`, or include principals and external clients immediately?

2. Should observation be represented as a capability operation, a policy judgment, a typeclass-like interface, or a runtime service?

3. How should views compose?
   Is an explicit projection graph enough, or does Ash need a richer visibility preorder?

4. What is the first stable subject family?
   Mailboxes, process status, workflow progress, and workflow reports are all plausible first targets.

5. How should observation evidence be stored?
   It may belong in workflow reports, runtime traces, capability provenance, or a separate observation log.

6. What does redaction mean for successful values?
   If `Proc<A>` completes but `A` is not observable to the current observer, should the state be `CompletedOpaque`, `Completed(Redacted)`, or a distinct view-specific state?

7. Which contexts are actually comonadic?
   Snapshot, timeline, process-tree, mailbox-stream, trace, and workflow-evidence foci may all have different lawful structures.

8. How does cancellation or garbage collection affect observable contexts?
   A context may outlive the live process as a retained snapshot or report, but that changes its focus semantics.

9. How does this interact with small-step/runtime-observable conformance?
   Existing runtime-observable result formats may be an early concrete substrate, but they are not audience-specific yet.

10. What is the noninterference test strategy?
    Future tasks will need negative leakage assertions, not only positive visibility examples.

## Non-goals for now

- Do not add a public `Observable` type just because the concept is useful.
- Do not add `Comonad` instances for `Act`, `Proc`, `Workflow`, or `Mailbox` directly.
- Do not expose runtime internals as a shortcut for tools.
- Do not assume every runtime status is safe to publish.
- Do not make observation ambient or global.
- Do not introduce UI/rendering syntax as part of this idea.

## Working thesis

Ash execution is monadic/process-oriented in the direction of doing work.
Ash observation may be comonadic/contextual in the direction of interpreting authorized state.

The bridge between them is not raw introspection.
It is explicit, policy-governed, audience-relative observable views.

A compact formulation:

```text
Subject owns hidden state.
Subject declares views.
Policy grants observer access to a specific view.
Observation produces an authorized context.
Comonadic projection composes inside that context without widening visibility.
Publishing a new view requires a new declaration and grant.
```

This is the idea to preserve.
