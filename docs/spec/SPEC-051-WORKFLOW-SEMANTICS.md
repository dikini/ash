# SPEC-051: Workflow Semantics

**Status:** Draft
**Date:** 2026-04-24
**Related:** DESIGN-030, SPEC-019, SPEC-022, SPEC-025, SPEC-049, SPEC-050, SPEC-048, SPEC-047, NOTE-006, NOTE-007, NOTE-008
**Promotes:** DESIGN-030 workflow-semantics follow-on direction and NOTE-006 workflow failure-boundary direction for overlapping workflow governance semantics

## Summary

This specification defines the initial normative workflow-semantics layer above `Proc<A>` in the current Ash semantic tower:

```text
Pure < Effectful / Act < Proc < Workflow
```

Workflow is not merely syntax around process execution. Workflow adds governance semantics over admitted process execution:

- workflow identity and run admission;
- role/capability context admission;
- `requires` and `ensures` governance;
- obligation completion boundaries;
- workflow reporting;
- `WorkflowFailure` construction;
- reinterpretation of unhandled lower-level operational failures.

This spec is intentionally initial. It defines the semantic boundary and first invariants without redesigning the full legacy workflow small-step corpus in [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md).

## 1. Scope and Authority

### 1.1 In scope

This spec defines:

1. workflow as the governance stratum above `Proc`;
2. workflow identity and run admission;
3. admitted role/capability context;
4. semantic status of `requires` and `ensures`;
5. workflow body execution as governed process execution;
6. workflow completion and reporting;
7. `WorkflowFailure` shape and lower-failure reinterpretation;
8. relation to role runtime semantics and workflow typing;
9. compatibility with existing SPEC-004/SPEC-025 workflow outcomes.

### 1.2 Out of scope

This spec does not define:

1. parser syntax for workflow declarations; see existing surface/core specs;
2. all workflow typing rules; see [SPEC-022](SPEC-022-WORKFLOW-TYPING.md);
3. role authority details; see [SPEC-019](SPEC-019-ROLE-RUNTIME-SEMANTICS.md);
4. process runtime details; see [SPEC-049](SPEC-049-PROCESS-RUNTIME-SEMANTICS.md);
5. general operational bottom and `with_error`; see [SPEC-050](SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md);
6. concrete CLI/host API;
7. full supervisor/orchestration policy.

### 1.3 Normative vs informative

Unless marked informative, sections are normative. Examples and representation sketches are informative unless phrased as conformance requirements.

## 2. Workflow Position in the Tower

Workflow is the governance stratum above `Proc`:

```text
Workflow
  governs/adopts Proc execution
Proc
  structures process execution and process observation
Effectful / Act
  performs capability/provider/policy-mediated effectful operations
Pure
  computes values without effect environment
```

Workflow does not introduce the first capability semantics. Capability/provider/policy admissibility begins in the Effectful/Act stratum and may be projected through Proc. Workflow governs which roles/capabilities/processes are admitted, what contracts must hold, and how completion/failure is reported.

## 3. Workflow Identity and Run Admission

### 3.1 Identities

A workflow execution has at least:

```text
WorkflowId
RunId
```

A `WorkflowId` identifies the workflow execution entity. A `RunId` identifies the host/runtime run that may contain one or more workflow/process identities.

A workflow may own or adopt one root `ProcessId` for its body execution and may admit child `ProcessId`s through process operations.

### 3.2 Admission boundary

A workflow run starts only when admitted by an outside runtime, host command, scheduler, or another workflow boundary.

Admission creates or validates:

1. `WorkflowId`;
2. `RunId` linkage;
3. input argument values;
4. admitted role context;
5. admitted capability surface;
6. admitted provider/policy context;
7. workflow contract context;
8. report/audit sinks;
9. root process environment for the workflow body.

Admission failure is a workflow-level operational failure. It may occur before the workflow body starts.

### 3.3 No ambient leakage

Lexical definitions of roles, capabilities, and contracts may be referenced when elaborating workflow headers, but workflow body visibility is limited to explicitly admitted context.

A workflow body must not gain access to ambient roles/capabilities merely because they are lexically defined in surrounding modules.

## 4. Workflow Context

A workflow context contains or indexes:

1. workflow identity;
2. run identity;
3. input bindings;
4. admitted role context;
5. admitted capability surface;
6. admitted provider/policy context;
7. workflow contract state;
8. obligation state;
9. provenance/audit/report state;
10. lower process root identity;
11. lower failure reinterpretation policy.

Conceptual lookup follows the identity-indexed model from NOTE-007:

```text
(Workflow, WorkflowId, ComponentType, Key) -> Component
```

Workflow context projects lower process/effect contexts; lower strata must not manufacture workflow authority.

## 5. Roles and Capabilities

### 5.1 Role admission

Workflow role semantics are governed by [SPEC-019](SPEC-019-ROLE-RUNTIME-SEMANTICS.md). This spec fixes the boundary relation. The initial conformance profile admits zero or one active role context, matching SPEC-019's `Option<RoleContext>` / single `active_role` model. Plural role wording in this spec is future-facing only; in the initial profile it denotes a singleton-or-empty admitted role set.

1. Role assignment/admission occurs at workflow admission or explicit workflow spawn/adoption boundaries.
2. Role authority restricts effect/capability use through lower effectful execution.
3. Role obligations from the active role contribute to workflow completion requirements.
4. Role context is stable for the admitted workflow execution unless a future spec introduces explicit role transition semantics.

### 5.2 Capability admission

A workflow may declare or require a capability surface. Admission validates that the runtime can provide the required capability surface to the workflow and its lower process/effect environments.

The workflow boundary may narrow capability availability before projecting it into `Proc` and `Act`. It must not widen lower-stratum authority beyond the admitted workflow context.

### 5.3 Authority failure

If role/capability admission fails, the workflow fails before body execution with a workflow-level admission failure.

If a lower effect invocation violates admitted authority during body execution, the immediate failure may be effect-level, but an unhandled failure escaping the workflow body is reinterpreted at the workflow boundary as `WorkflowFailure` with the lower failure preserved as cause/evidence.

## 6. `requires` Semantics

### 6.1 Admission-time requirements

Workflow `requires` clauses are preconditions over admitted inputs and context.

For internal statically typed workflow calls, [SPEC-022](SPEC-022-WORKFLOW-TYPING.md) owns this check: unsatisfied `requires` predicates are call-site type/proof errors, not runtime workflow failures.

For external host admission, dynamic adoption, unchecked imported workflows, or explicit non-strict conformance modes, the runtime admission boundary validates `requires` predicates. If a required predicate fails in those modes, the workflow does not start normally. It produces a workflow admission failure.

### 6.2 Call-site requirements

When one workflow or function calls/adopts another workflow, callee `requires` predicates become obligations at the call/admission site. This is compatible with [SPEC-022](SPEC-022-WORKFLOW-TYPING.md), where requirements are checked at call sites.

### 6.3 Runtime evidence

A workflow report should record which `requires` predicates were checked, whether each passed, and what evidence/context was used where available.

## 7. `ensures` Semantics

### 7.1 Completion-time requirements

Workflow `ensures` clauses are postconditions over the workflow's normal result and final admitted/reportable context.

A workflow that reaches normal body completion must still validate `ensures` before it is reported as successful.

If an `ensures` predicate fails, the workflow completes unsuccessfully with a workflow-level `EnsuresViolation` failure. The lower body may have completed normally, but the workflow boundary reports failure because governance completion failed.

### 7.2 Evidence

A workflow report should record each `ensures` predicate, result, and evidence/context where available.

## 8. Obligations and Completion Boundary

Workflow-local obligations and role obligations must be discharged according to [SPEC-022](SPEC-022-WORKFLOW-TYPING.md) and [SPEC-019](SPEC-019-ROLE-RUNTIME-SEMANTICS.md).

At workflow completion, the workflow boundary checks:

1. all local workflow obligations are discharged;
2. all active role obligations are discharged;
3. all required completion evidence is present;
4. `ensures` predicates hold.

For workflows accepted under SPEC-022 strict typing, undischarged local workflow obligations are statically impossible. A runtime `LocalObligationsUndischarged` completion failure is reserved for non-strict, dynamic, host-admitted, legacy/adopted workflows, or implementation invariant breaches. Active-role obligation failures remain runtime completion failures according to SPEC-019's role runtime model.

## 9. Workflow Body as Governed Process Execution

### 9.1 Root process

A workflow body executes through a root process or process-compatible runtime entity admitted by the workflow boundary.

Conceptual shape:

```text
admit_workflow_body(workflow_ctx, body) -> Proc<A>
run_root_process(workflow_ctx, Proc<A>) -> ProcessOutcome<A>
```

This does not require the surface workflow syntax to become explicit `proc` syntax. It states the semantic relation: workflow governs process-capable execution below it.

### 9.2 Lower context projection

The workflow boundary projects a lower process environment that is equal-or-less-authorized than the workflow's admitted context.

Projection must include:

1. root `ProcessId`;
2. admitted capability/effect surface;
3. role/policy context needed for effect invocation;
4. workflow report/audit sinks;
5. process failure/reporting hooks;
6. obligation/provenance linkage.

### 9.3 Process failures inside workflow

A process failure inside the workflow body remains a process-level operational failure until one of the following occurs:

1. it is handled by a `with_error` scope inside the body;
2. it is observed by Proc-layer operations such as `await`, `join`, or `gather` through the `proc` library/lowering boundary defined by SPEC-048/SPEC-049;
3. it escapes the root workflow body and reaches the workflow boundary.

This does not add new SPEC-025 workflow small-step forms or user-visible workflow `await` syntax. Legacy workflow spawn/control-link observation remains governed by SPEC-004/SPEC-025 until explicitly revised.

If it escapes the workflow boundary, it is reinterpreted as `WorkflowFailure` with lower cause/evidence preserved.

## 10. Workflow Outcomes

A workflow execution reports one of:

```text
WorkflowSucceeded(value, WorkflowReport)
WorkflowFailed(WorkflowFailure, WorkflowReport)
```

This is compatible with existing `Return(...)` / `Reject(...)` terminology in [SPEC-004](SPEC-004-SEMANTICS.md) and [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) because SPEC-051 outcomes are outer workflow-boundary outcomes. The governed body still terminates using the SPEC-004/SPEC-025 `Return`/`Reject` vocabulary. `WorkflowSucceeded`/`WorkflowFailed` are constructed by the workflow boundary around admission and completion governance:

1. admission failure can construct `WorkflowFailed` before body execution;
2. body `Return(...)` plus successful governance checks constructs `WorkflowSucceeded`;
3. body `Reject(err, ...)` constructs `WorkflowFailed` with lower cause/evidence;
4. body `Return(...)` plus failed completion governance constructs `WorkflowFailed`.

This spec does not add a new SPEC-025 terminal configuration form.

## 11. `WorkflowFailure`

Conceptual shape:

```text
WorkflowFailure {
  workflow_id: WorkflowId,
  run_id: RunId,
  kind: WorkflowFailureKind,
  cause: Option<OperationalFailure>,
  evidence: WorkflowFailureEvidence,
}
```

Minimum failure kinds:

```text
AdmissionFailure
RequiresViolation
RoleAdmissionFailure
CapabilityAdmissionFailure
BodyFailureEscaped
EnsuresViolation
LocalObligationsUndischarged
RoleObligationsUndischarged
ReportCommitFailure
RuntimeFailure
```

### 11.1 Cause preservation

If the workflow failure is caused by a lower operational failure, `cause` must preserve that lower failure.

Examples:

1. unhandled effect provider failure escaping workflow body;
2. unhandled process failure observed by root process;
3. cancellation propagated from process runtime;
4. policy denial during effect invocation.

Workflow reinterpretation must not erase lower process/effect identity.

### 11.2 Governance-owned failures

Some failures originate at workflow governance level and may have no lower cause:

1. `RequiresViolation` at admission;
2. `EnsuresViolation` at completion;
3. undischarged workflow/role obligations;
4. admission failure before root process creation;
5. report commit failure.

These failures still carry workflow identity and report evidence.

## 12. Workflow Report

A workflow report is the governed execution record visible at the workflow boundary.

Minimum conceptual contents:

```text
WorkflowReport {
  workflow_id: WorkflowId,
  run_id: RunId,
  status: Succeeded | Failed,
  admitted_role_context: ...,
  admitted_capabilities: ...,
  requires_evidence: ...,
  ensures_evidence: ...,
  obligation_evidence: ...,
  lower_process_summary: ...,
  lower_failure_causes: ...,
  provenance: ...,
}
```

Report commit failure refers to failure to commit the report to an external report/audit sink. It does not mean no workflow boundary outcome can be returned. If external report commit fails, the runtime must still construct a minimal local/in-memory `WorkflowReport` containing the commit failure evidence, unless the host boundary itself suffers a catastrophic failure outside workflow semantics.

This shape is conceptual. A conforming implementation must preserve enough report information for auditability and runtime observable behavior, but this spec does not require an exact serialized format.

## 13. Boundary with `with_error`

A `with_error` inside a workflow body can handle operational failures routed through its dynamic scope, including observed process failures from Proc-layer `await`, `join`, or `gather` operations when those operations are present through the Proc library/lowering boundary.

However, failures at the workflow admission boundary occur before the body runs and therefore are not caught by handlers inside the body.

Completion-boundary failures such as `EnsuresViolation` and undischarged role obligations occur after body normal completion. Whether source-level handlers can be placed around an entire workflow invocation from the caller side is a caller/admission semantics question; body-internal handlers do not catch completion-boundary failures that are created after the body has returned.

## 14. Compatibility with SPEC-004 and SPEC-025

[SPEC-004](SPEC-004-SEMANTICS.md) and [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) remain the existing workflow operational semantics backbone.

This spec refines the workflow boundary by making explicit:

1. workflow governance above process execution;
2. admission and completion failure boundaries;
3. `WorkflowFailure` construction;
4. lower-failure cause preservation;
5. workflow reporting requirements.

This spec does not silently replace the accepted small-step configuration vocabulary in SPEC-025. Future alignment work may revise SPEC-004/SPEC-025 terminology to use the `WorkflowFailure` vocabulary directly.

## 15. Conformance Requirements

A conforming implementation of this initial workflow semantics model must:

1. create or validate workflow/run identity at admission;
2. enforce explicit role/capability admission without ambient leakage;
3. check workflow `requires` at admission/call boundary;
4. check workflow `ensures` after normal body completion and before success reporting;
5. verify local and role obligations at completion;
6. execute the workflow body under a projected lower process/effect context;
7. preserve lower operational failure cause/evidence when constructing workflow failure;
8. distinguish body failures from workflow admission/completion governance failures;
9. produce an auditable workflow report for success and failure;
10. remain compatible with SPEC-004/SPEC-025 outcome reconstruction until those specs are explicitly revised.

## 16. Deferred Questions

1. Exact syntax and type shape for workflow invocation as a first-class value, if any.
2. Whether caller-side handlers can catch workflow admission and completion failures uniformly.
3. Exact serialized `WorkflowReport` schema.
4. Exact taxonomy and typed payloads for `WorkflowFailureKind`.
5. How workflow spawning/adoption relates to future supervisors.
6. How workflow cancellation is initiated, propagated, and reported.
7. How much of SPEC-004/SPEC-025 should be rewritten around the semantic tower once implementation pressure justifies it.

## Changelog

### 2026-04-24

- Initial draft defining workflow as the governance stratum above `Proc`, including admission, roles/capabilities, `requires`/`ensures`, obligation completion, workflow reporting, `WorkflowFailure`, and lower-failure reinterpretation.
