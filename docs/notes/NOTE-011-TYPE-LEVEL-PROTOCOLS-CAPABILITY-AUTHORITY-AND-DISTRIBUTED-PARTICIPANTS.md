# NOTE-011: Type-Level Protocols, Capability Authority, and Distributed Participants

**Status:** Exploratory design note / initial discussion capture
**Date:** 2026-05-06
**Related:** [NOTE-007](NOTE-007-RUNTIME-ENVIRONMENT-IDENTITY-AND-COMPONENTS.md), [NOTE-009](NOTE-009-CAPABILITY-INTERFACES-IMPLEMENTATIONS-AND-INTERNAL-AUTHORITY.md), [SPEC-048](../spec/SPEC-048-PROC-LIBRARY.md), [SPEC-049](../spec/SPEC-049-PROCESS-RUNTIME-SEMANTICS.md), [SPEC-050](../spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md), [SPEC-051](../spec/SPEC-051-WORKFLOW-SEMANTICS.md), [SPEC-052](../spec/SPEC-052-CAPABILITY-INTERFACES-AND-IMPLEMENTATIONS.md), [SPEC-053](../spec/SPEC-053-RUNTIME-RESOURCES-AND-AUTHORITY-PROVENANCE.md), [SPEC-056](../spec/SPEC-056-FIRST-CLASS-WORKFLOW-CARRIER.md), [DESIGN-034](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)

## 1. Purpose

This note captures an initial architecture discussion about how Ash capability, authority, distributed computation, sandboxing, inter-workflow communication, LLM/tool interaction, and workflow-synchronized external actors should fit together.

The central preference recorded here is:

> Model CSP / restricted π-calculus / MPST-style interaction mostly in the Ash type system, using rich type expressions plus native capability, authority, resource, Proc, and Workflow concepts. Runtime support should be minimal: endpoint/session identity, message routing, residual validation at external boundaries, host authority enforcement, failures, and provenance. Surface syntax is deferred.

This note is intentionally not a normative spec and not a syntax proposal. It is a map of design dimensions and candidate invariants to guide later spec packets.

## 2. Starting Context

The discussion began from three scenarios:

1. multiple Ash nodes running workflows that message each other;
2. starting a new Ash node with a specific workflow from an Ash program;
3. starting such a node in a sandboxed environment.

The initial framing distinguished:

```text
Capability interface       = what effectful operations exist
Capability implementation  = how those operations are realized from dependencies
Capability binding         = admitted association of interface + implementation + concrete authority
Authority                  = actual right/power over resources, providers, nodes, endpoints, etc.
```

Existing Ash work already has the right local substrate direction:

- capability interfaces / implementations / bindings in SPEC-052;
- runtime resources and authority provenance in SPEC-053;
- Proc/process identity and child environment projection in SPEC-048/SPEC-049;
- operational failure identity in SPEC-050;
- workflow governance and contract-indexed Proc in SPEC-051/SPEC-056;
- rich type-expression substrate work in DESIGN-034 and later specs.

The discussion then widened from direct distributed execution into remote discovery, LLM/tool surfaces, workflow-synchronized external actors, and finally type-level protocol models.

## 3. Design Axes

The concepts discussed are better treated as orthogonal axes that combine, not as one feature.

| Axis | Question | Initial direction |
| --- | --- | --- |
| Capability / authority | What authority backs an operation? | Interfaces are stateless; bindings carry authority provenance. |
| Resources | Where does state/identity/lifetime live? | Resource instances are identity-bearing runtime components, not hidden impl state. |
| Proc / distributed placement | Where does computation execute? | Nodes/processes/sandboxes are placement and failure boundaries. |
| Workflow protocol | What interaction is being followed? | Workflow contracts/protocols govern participants and evidence. |
| Advertisement / discovery | What does an external peer see? | Caller- and state-specific protocol/tool manifests. |
| External actors | How do LLMs/non-Ash processes participate? | They join protocol sessions through projected endpoints/tools. |
| Formal model | How is communication specified? | MPST + CSP traces/failures + restricted π-style fresh endpoints. |
| Type/runtime split | Where does meaning live? | Mostly in type/contract artifacts; minimal runtime witness/checker. |

## 4. Capability, Authority, and Binding

The basic invariant remains NOTE-009/SPEC-052/SPEC-053 aligned:

```text
Capability declarations do not grant authority.
Capability implementations do not own hidden mutable state.
Capability bindings connect operation surfaces to concrete admitted authority.
```

Every operation available to a workflow, process, node, sandbox, LLM peer, or third-party process should ultimately trace back to an admitted binding or resource authority.

### 4.1 No Ambient Authority Transfer

New processes, workflows, nodes, sandboxes, or external actor sessions must not inherit an ambient clone of the parent runtime context.

They receive an explicit projection:

```text
project_authority(parent_env, requested_contract, placement, sandbox_policy)
  -> child_env | OperationalFailure
```

Projection may narrow, wrap, adapt, log, cache, simulate, or delegate. It must not widen external authority.

### 4.2 Authority Dimensions

Authority may include:

- provider operation authority;
- resource access authority;
- filesystem/network/process/secrets authority;
- node-spawn or sandbox-spawn authority;
- endpoint communication authority;
- discovery/disclosure authority;
- LLM/model-call authority and budget authority;
- evidence/report/provenance append authority.

These should remain explicit enough for typechecking, workflow coverage, and runtime provenance.

## 5. Distributed Computation as Complementary Substrate

Distributed computation is not the whole model for LLM/workflow coordination. It supplies the placement, isolation, messaging, and failure substrate for participants that are not co-located.

Useful first-slice meaning:

```text
Distributed computation = protocol-capable placement and failure semantics,
not transparent distributed shared memory or a full actor calculus.
```

In scope for the useful substrate:

1. node/process/workflow identities;
2. endpoint addressing;
3. typed/session-scoped messages;
4. authority projection;
5. timeout/cancel/crash/sandbox failure reporting;
6. causal/provenance links;
7. lifecycle sufficient to start/stop sandboxed participants.

Deferred:

- consensus;
- transparent distributed resource migration;
- arbitrary channel/link sending;
- general supervision trees;
- exactly-once delivery;
- distributed GC;
- public capability marketplaces.

## 6. Sandboxed Participants

A sandboxed node/process should be viewed as a constrained protocol participant:

```text
SandboxedParticipant {
  node/process identity,
  protocol endpoint,
  sandbox spec,
  admitted role,
  authority envelope,
  failure boundary,
  provenance obligations,
}
```

Example: in a TDD workflow, a sandboxed verifier may implement only the `Verifier` role:

```text
receive RunTests(patch)
send FailingAsExpected(evidence)
  or InvalidFailure(reason)
  or SandboxFailure(failure)
```

The sandbox does not need broad repository or workflow authority. It receives only the endpoint, test-runner capability, scratch/worktree resource, and OS/runtime restrictions required for its role.

Ash semantic sandboxing and OS/process sandboxing are complementary:

```text
Ash envelope says what authority exists.
OS/container sandbox enforces external resource boundaries.
Provider/resource admission keeps Ash-level execution honest.
```

## 7. Remote Discovery and Advertisement

A missing dimension was remote capability/resource discovery, especially for LLM/intelligence providers and non-Ash processes.

The important distinction:

```text
CapabilityBinding       = internal authority-bearing binding
CapabilityAdvertisement = external/caller-visible projection of selected authority
ToolDescriptor          = protocol-specific representation, e.g. MCP/OpenAPI/function-call schema
```

Discovery must not be a raw list of providers. It should be contextual:

```text
discover(caller_identity, session, protocol_state, disclosure_policy)
  -> manifest
```

The manifest may include:

- currently enabled protocol actions;
- tool descriptors for external actors;
- resource advertisements;
- input/output schemas;
- observe/execute and safety/effect class;
- call policy such as auto, require approval, dry-run first, disabled;
- authority summary and constraints;
- evidence requirements.

### 7.1 Discovery Is Separate Authority

There is separate authority to:

1. know that a capability/resource/tool exists;
2. see its schema or description;
3. invoke it;
4. see results or evidence.

Discovery itself may need audit/provenance when it discloses sensitive topology or authority information.

## 8. LLMs and External Actors

The discussion distinguished several cases:

1. **Ash calls an LLM provider** — ordinary external authority provider, e.g. `Llm.complete`, with data disclosure, model, budget, and tool-use policy.
2. **An LLM calls Ash tools** — external actor invokes advertised affordances through a gateway, e.g. MCP.
3. **An LLM participates in an Ash workflow protocol** — richer case where tool calls synchronize with workflow steps and evidence.

The third case is the most important design point.

The LLM should not receive a global bag of tools. It should receive a role in a workflow/protocol session:

```text
LLM joins protocol session as role R.
Current local protocol state determines enabled actions.
Enabled actions project to tools.
Tool calls become protocol events/evidence.
Ash workflow validates and advances or rejects.
```

## 9. Workflow-Synchronized External Actor Protocols

The TDD example motivated a richer protocol than direct tool mapping.

An LLM prompt may describe a procedure:

```text
1. Write a failing test.
2. Verify it fails for the intended reason.
3. Implement the smallest fix.
4. Verify tests pass.
5. Refactor while preserving behavior.
```

Ash can mirror this with an authoritative workflow/protocol contract:

```text
Red -> Green -> Refactor -> Complete
```

Each step has:

```text
StepContract {
  preconditions,
  admitted actions/tools,
  required evidence,
  verifier obligations,
  transition rules,
  failure/divergence policy,
}
```

Tool calls become workflow/protocol events, not opaque RPCs:

```text
ToolInvocation
  -> WorkflowEvent
  -> evidence/provenance update
  -> verifier check
  -> transition / refusal / divergence
```

### 9.1 Differential-Testing Analogy

There are two executions:

1. the external actor's claimed procedure;
2. Ash's authoritative workflow/protocol state.

Ash compares claims/actions against observed evidence:

```text
LLM claims Red is complete
Ash checks: test diff exists, implementation unchanged, test fails as intended
```

States such as `Aligned`, `NeedsEvidence`, `ExternalAhead`, `Diverged`, `RejectedStep`, and `Blocked` become meaningful protocol/session states.

## 10. Formal Model Preference

The preferred formal basis is:

```text
MPST-inspired global protocols and local role projections
+ CSP-style traces, refusals, failures, and divergence observations
+ restricted π-calculus-style fresh session/endpoint creation
```

Explicitly deferred:

- full process passing;
- arbitrary channel/link sending;
- unrestricted endpoint mobility;
- dynamic role topology as the first model.

### 10.1 MPST Role Projection

A global protocol defines participants and messages:

```text
Coordinator -> LLM: RequestRed(requirement)
LLM -> Coordinator: ProposedTestPatch(patch)
Coordinator -> Verifier: RunTests(patch)
Verifier -> Coordinator:
    FailingAsExpected(evidence)
  | InvalidFailure(reason)
  | SandboxFailure(failure)
```

Each role receives a local projection. The LLM local projection might say:

```text
recv RequestRed
send ProposedTestPatch
recv RedAccepted | RedRejected
...
```

The verifier projection might say:

```text
recv RunTests
send FailingAsExpected | InvalidFailure | SandboxFailure
```

### 10.2 CSP Trace/Refusal/Failure View

CSP contributes behavioral observations:

```text
trace = <request_red, proposed_test_patch, run_tests, failing_as_expected, red_accepted>
```

and refusal explanations:

```text
edit_implementation refused during Red
advance_green refused before FailingAsExpected evidence
verifier_result refused after session closed
```

This directly supports dynamic tool manifests:

```text
available actions = protocol-enabled events not refused in current local state
```

### 10.3 Restricted π Role

Restricted π-calculus contributes fresh scoped names:

```text
new session s in Coordinator[s] | LLM[s] | Verifier[s]
```

But first-slice Ash should allow only controlled fresh session/endpoint creation. It should not permit arbitrary process/link sending.

## 11. Type-System-First Direction

The strongest design preference recorded in the discussion is type-system-first:

```text
Protocol meaning is static.
Runtime endpoint/session state is only a residual witness of an elaborated protocol projection.
```

The type system should own as much as possible:

- global protocol well-formedness;
- role projection;
- local protocol conformance;
- endpoint state transitions in Ash code;
- capability/resource/authority requirements;
- workflow contract coverage;
- allowed event/tool surface from protocol state.

Runtime should own only residual facts:

- fresh `SessionId`, `EndpointId`, `ProcessId`, `NodeId` allocation;
- message delivery;
- external actor validation;
- host authority enforcement;
- timeouts, cancellation, crash, sandbox denial;
- trace/provenance recording.

## 12. Type-Level Protocol Shape

The user-facing type should probably remain small, following the `Workflow<A>` / `WorkflowForm` pattern.

Candidate public/simple type:

```text
Endpoint<P, R>
```

or:

```text
Session<P, R>
```

Rich internal/compiler artifact:

```text
EndpointArtifact {
  protocol,
  role,
  state,
  required_authority,
  required_resources,
  latent_obligations,
  evidence_requirements,
}
```

A more precise internal form may include state:

```text
Endpoint<P, R, S>
```

but the note records a caution: avoid public type explosion unless implementation pressure justifies it. Prefer rich typechecker sidecars/artifacts where possible.

### 12.1 Protocol Operations as Typestate Transitions

Conceptual operation shapes:

```text
send : Endpoint<P, R, Send<label, A, SNext>> -> A -> Proc<Endpoint<P, R, SNext>>
recv : Endpoint<P, R, Recv<branches>> -> Proc<(Message, Endpoint<P, R, SNext>)>
```

These are not syntax proposals. They express the desired static relation:

```text
using an endpoint consumes one protocol state and yields the next.
```

For external actors, the generated local projection becomes a runtime manifest/checker because the actor is not Ash-typechecked.

## 13. Authority-Indexed Protocol Events

An Ash protocol event is enabled only when both protocol and authority allow it:

```text
enabled(event) =
  protocol_enabled(event, local_state)
  ∧ authority_enabled(event, endpoint_authority)
  ∧ resource_policy_enabled(event, resources)
```

Example:

```text
Coordinator -> Verifier: RunTests(patch)
  requires TestRunner.execute
  requires ScratchWorktree exclusive/isolated
  requires SandboxAuthority(network = none, fs = scratch_only)
```

MPST controls communication order and branch structure. Ash capability/resource authority controls whether the event can actually be performed.

## 14. Tool Manifests as Local Protocol Projections

Earlier direct mapping:

```text
CapabilityBinding -> Tool
```

is insufficient. The refined model is:

```text
LocalProtocolState + endpoint authority + disclosure policy
  -> tool/resource/protocol manifest
```

For the LLM during TDD Red phase, projected tools may include:

- inspect requirement;
- read relevant code;
- submit test patch;
- run or request test verification;
- submit evidence.

They should not include implementation-edit or completion tools until the protocol state admits them.

## 15. Failure Dimensions

The protocol/failure taxonomy should distinguish at least:

| Failure | Interpretation |
| --- | --- |
| Protocol violation | Message/action not allowed by local projection state. |
| CSP refusal | Action known but refused in current state/policy. |
| Divergence | Participant loops, stalls, or cannot make required progress. |
| Timeout | Expected event/evidence not received in time. |
| Transport failure | Message delivery failed. |
| Node/process failure | Participant runtime crashed or ended. |
| Sandbox denial | Operation exceeded sandbox authority. |
| Capability denied | Binding/action not admitted. |
| Verifier rejected | Evidence insufficient or invalid. |
| Expected domain failure | e.g. failing test in Red, may satisfy protocol. |
| Unexpected domain failure | e.g. syntax/setup failure instead of intended failing test. |
| Operational failure | Ash runtime/provider failure with identity. |

These categories should not collapse into generic tool errors.

## 16. Candidate Future Spec Packets

This note suggests several future specs rather than one mega-spec.

### 16.1 Protocol Types and Endpoint Projections

Owns:

- global protocols;
- roles;
- local projections;
- endpoint/session type artifacts;
- protocol typestate transitions;
- MPST/CSP formal positioning;
- restrictions on endpoint/process mobility.

### 16.2 Capability Advertisement and Tool Manifests

Owns:

- advertisement vs binding distinction;
- caller-specific discovery;
- tool/resource manifests;
- MCP/OpenAPI/function-call projection;
- disclosure policy;
- no-widening from binding to advertisement.

### 16.3 Workflow-Synchronized External Actor Protocols

Owns:

- external actor sessions;
- step contracts;
- claims/evidence;
- dynamic tool manifests by protocol state;
- divergence states;
- LLM/non-Ash actor participation.

### 16.4 Workflow Endpoint and Inter-Workflow Messaging

Owns:

- endpoint addressing;
- session-scoped messages/events;
- causal/correlation IDs;
- inter-workflow protocol negotiation;
- endpoint authority.

### 16.5 Sandboxed Participant Execution

Owns:

- sandboxed participant start;
- authority envelope projection;
- OS/runtime sandbox alignment;
- sandbox-specific failure modes;
- endpoint bootstrap.

### 16.6 Distributed Runtime Placement

Owns later:

- `NodeId`;
- remote process/workflow handles;
- transport adapters;
- node registries;
- remote failure aggregation.

## 17. Non-Goals for the First Model

The first model should avoid:

1. public surface syntax commitments;
2. full process/link/channel sending;
3. arbitrary dynamic role topology;
4. transparent distributed shared memory;
5. public marketplace-style capability discovery;
6. making runtime the source of protocol meaning;
7. exposing every protocol state as a public type parameter unless necessary;
8. collapsing LLM/tool interaction into a flat list of tools;
9. allowing provider registration to imply advertisement or invocation authority.

## 18. Current Working Invariants

1. **Protocol meaning is static.** Runtime state is a residual witness/checker of typechecked protocol artifacts.
2. **Participants get protocol-scoped affordances, not ambient tools.**
3. **Advertisement is not authority.** It is an attenuated, caller-specific projection of admitted authority.
4. **Discovery is itself authority-controlled.** Knowing a tool/resource exists may be sensitive.
5. **Protocol events require both protocol permission and capability/resource authority.**
6. **Distributed computation is a placement/failure substrate, not the primary abstraction.**
7. **Sandboxed computation is a constrained participant role.**
8. **External actors are checked at the boundary.** Their manifests/checkers are generated from type-level protocol artifacts.
9. **Runtime must not invent allowed protocol behavior.** It may only execute/check behavior derived from static artifacts.
10. **Surface syntax is deferred.** Keep this as semantic/type-system design until implementation pressure clarifies the right syntax.

## 19. Open Questions

1. Should public endpoint types expose state (`Endpoint<P, R, S>`) or keep state in compiler artifacts behind `Endpoint<P, R>`?
2. What is the minimal type-expression substrate needed to represent global protocols, local projections, branches, and authority constraints?
3. Should protocol projection live in `ash-core` semantic carriers, `ash-typeck` artifacts, or both?
4. How should imported protocol summaries work across modules without leaking private authority/resource details?
5. What is the first external protocol projection target: MCP, JSON-RPC, OpenAPI, or an Ash-native debug protocol?
6. How should evidence obligations integrate with existing `requires` / `ensures` / `WorkflowForm` projection machinery?
7. Which failure categories should be domain-level protocol messages versus operational bottom?
8. What is the first smoke-test scenario: TDD workflow with LLM actor, sandboxed verifier, or two Ash workflows exchanging evidence?
9. How much of CSP refusal/failure semantics must be static versus runtime-derived from compiled automata?
10. What are the exact no-widening checks from capability binding to advertisement to protocol event?

## 20. Suggested Next Step

Before implementation, write a narrower design packet for **Protocol Types and Endpoint Projections**.

That packet should answer only:

1. what a global protocol artifact is;
2. what a role projection artifact is;
3. what an endpoint type/artifact carries;
4. how capability/resource/authority obligations attach to protocol events;
5. what runtime witness/checker remains;
6. what is explicitly deferred.

The first executable smoke test should be small and local: one coordinator workflow, one external/LLM-like actor adapter, one verifier participant, and a TDD-style Red transition with evidence acceptance/refusal. Network distribution and OS sandboxing can be added after the type/protocol artifact boundary is stable.
