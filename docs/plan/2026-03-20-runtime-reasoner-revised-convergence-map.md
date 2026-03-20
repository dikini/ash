# Revised Runtime-Reasoner Convergence Map

Date: 2026-03-20
Status: TASK-198 output

## Purpose

This map synthesizes the runtime-reasoner impact audit, the implementation-planning surface note,
and the phase handoff into one convergence view.

Its purpose is to say:

- which existing planned tasks stay unchanged
- which existing planned tasks should be updated in place
- which later code-facing task clusters should be created only after a separate implementation-planning pass

It does not open implementation tasks.

## Inputs

- [Planned Convergence Tasks Runtime-Reasoner Impact Review](../audit/2026-03-20-planned-convergence-tasks-runtime-reasoner-impact-review.md)
- [Runtime-Reasoner Implementation-Planning Surface](../reference/runtime-reasoner-implementation-planning-surface.md)
- [Runtime-Reasoner Spec Handoff](2026-03-20-runtime-reasoner-spec-handoff.md)

## Revised Convergence Outcome

### Existing planned tasks that remain unchanged

These tasks do not need scope changes, blocking changes, or reinterpretation under the new
runtime-reasoner corpus:

- [TASK-164](tasks/TASK-164-route-receive-through-main-parser.md)
- [TASK-165](tasks/TASK-165-align-check-decide-ast-contracts.md)
- [TASK-166](tasks/TASK-166-replace-placeholder-policy-lowering.md)
- [TASK-167](tasks/TASK-167-lower-receive-into-canonical-core-form.md)
- [TASK-168](tasks/TASK-168-align-type-checking-for-policies-and-receive.md)
- [TASK-169](tasks/TASK-169-unify-runtime-verification-context-and-obligation-enforcement.md)
- [TASK-170](tasks/TASK-170-implement-end-to-end-receive-execution.md)
- [TASK-171](tasks/TASK-171-align-runtime-policy-outcomes.md)

The audit result is stable: these tasks still make sense without any reasoner present, so they
remain runtime-first convergence work.

### Existing planned tasks that should be updated in place

These tasks do not change scope, but they should later reference the new runtime-reasoner docs
corpus so implementation planning can describe human-facing wording and runtime-observable output
without conflating it with projection or monitorability:

- [TASK-172](tasks/TASK-172-unify-repl-implementation.md) - reference update only
- [TASK-173](tasks/TASK-173-implement-repl-type-reporting.md) - reference update only

The update is informational, not contractual. These tasks remain runtime-observable behavior work.

### Later code-facing task clusters to create after implementation planning

The new planning surface suggests future implementation work should be grouped into separate
clusters rather than opened as a single blended task.

#### 1. Runtime execution boundary cluster

Likely surfaces:

- `ash_engine::Engine::run`
- `ash_engine::Engine::run_file`
- `ash_engine::Engine::execute`
- `ash_interp::interpret`
- `ash_interp::execute_workflow`
- `ash_interp::execute_workflow_with_behaviour`
- `ash_interp::execute_workflow_with_stream`
- `ash_interp::execute_observe`
- `ash_interp::execute_send`
- `ash_interp::execute_set`
- `ash_interp::execute_stream`

This cluster would cover authoritative execution boundaries where later implementation planning may
need to decide how interaction-aware handling is surfaced, accepted, or rejected.

#### 2. Tooling and user-facing output cluster

Likely surfaces:

- `ash-cli::commands::run`
- `ash-cli::commands::trace`
- `ash-repl::Repl::run`

This cluster would cover wording, ordering, and presentation of runtime-observable output, including
any later distinction between advisory, gated, and committed stages.

#### 3. Provenance and trace cluster

Likely surfaces:

- `ash_provenance::TraceRecorder`
- `ash_provenance::TraceEvent`
- provenance export helpers
- workflow wrapper enter/exit hooks

This cluster would cover traceability and stage framing around accepted effects without redefining
runtime-only observability as reasoner projection.

## Decision Rules

The revised convergence map uses three simple rules:

1. If a task still makes complete sense without any reasoner present, it stays runtime-first and
   remains unchanged.
2. If a task only needs wording or reference alignment with the new docs corpus, it is updated in
   place.
3. If a task requires actual code changes to runtime boundaries, tooling, or trace surfaces, it
   belongs to a later code-facing cluster and should not be opened yet.

## Closure

The convergence queue is now split cleanly:

- unchanged runtime-first tasks
- in-place reference-only REPL follow-up
- deferred code-facing clusters for later implementation planning

No implementation task is opened by this map.
