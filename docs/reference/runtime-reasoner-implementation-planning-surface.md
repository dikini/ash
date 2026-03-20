# Runtime-Reasoner Implementation Planning Surface

## Status

TASK-197 planning reference.

## Purpose

This note defines the concrete implementation-planning surface implied by the current
runtime-reasoner docs corpus.

It is a planning reference, not a code contract. It exists so later implementation planning can
decide where the current Rust implementation, tooling, and docs need alignment work without
reopening the docs-only interaction model or introducing new syntax work by accident.

## Relationship to the Docs Corpus

This note is derived from:

- [Runtime-Reasoner Spec Handoff](../plan/2026-03-20-runtime-reasoner-spec-handoff.md)
- [Runtime-to-Reasoner Interaction Contract](runtime-to-reasoner-interaction-contract.md)
- [Surface Guidance Boundary](surface-guidance-boundary.md)
- [Ash Language Terminology Guide](../design/LANGUAGE-TERMINOLOGY.md)
- [Planned Convergence Tasks Runtime-Reasoner Impact Review](../audit/2026-03-20-planned-convergence-tasks-runtime-reasoner-impact-review.md)

Those documents establish the stable split between:

- runtime authority,
- runtime-to-reasoner projection and advisory output,
- runtime-only observability and monitoring,
- and explanatory-only surface guidance.

## Planning Rule

A concern is in scope for this planning surface if later implementation planning may need to decide
whether it needs:

- runtime code changes,
- tooling or CLI/REPL handling,
- docs or reference updates,
- or no action at all.

A concern is out of scope here if it is really:

- new syntax design,
- grammar redesign,
- runtime-to-reasoner protocol redesign,
- or a reclassification of runtime-only observability as reasoner projection.

The test remains the same: if the feature still makes complete sense without any reasoner present,
it is runtime-first and may only need a reference or tooling alignment decision later.

## Likely Runtime Entry-Point Classes

The following runtime-facing classes are the most likely places where later implementation planning
will need interaction-aware handling:

### 1. Workflow execution entry points

Examples:

- `ash_engine::Engine::run`
- `ash_engine::Engine::run_file`
- `ash_engine::Engine::execute`
- `ash_interp::interpret`
- `ash_interp::execute_workflow`
- `ash_interp::execute_workflow_with_behaviour`
- `ash_interp::execute_workflow_with_stream`

These are the main authoritative execution boundaries. Later planning may need to decide where
projection is surfaced, where advisory output returns are accepted, and where runtime rejection or
commitment is observable.

### 2. Primitive interpreter boundaries

Examples:

- `ash_interp::execute_observe`
- `ash_interp::execute_send`
- `ash_interp::execute_set`
- `ash_interp::execute_stream`

These are the most likely boundaries for later interaction-aware handling because they already sit
close to runtime admission, capability use, and observable state transition.

### 3. Workflow wrapper boundaries

Examples:

- `ash_macros::workflow`
- generated `enter_workflow` / `exit_workflow` wrapper code

These are relevant because they wrap execution entry and exit, making them natural places where
later planning may want to preserve interaction-aware tracing, provenance, or stage framing.

### 4. CLI and REPL entry points

Examples:

- `ash-cli::commands::run`
- `ash-cli::commands::trace`
- `ash-repl::Repl::run`

These are user-facing surfaces. They may need later handling for wording, output ordering, or
presentation of advisory versus authoritative results, but they do not define the runtime contract
themselves.

### 5. Provenance and trace surfaces

Examples:

- `ash_provenance::TraceRecorder`
- `ash_provenance::TraceEvent`
- `ash_provenance` export helpers
- `ash-cli` trace output formatting

These surfaces may need later planning if stage-level or interaction-level distinctions should be
visible in traces without changing runtime authority.

## Likely Work Areas

Later implementation planning should classify follow-up work into one of these buckets:

### Runtime

Work that changes authoritative runtime behavior or needs explicit interaction-aware handling at a
runtime boundary:

- acceptance and rejection boundaries
- projection or injected-context hooks
- provenance capture around accepted effects
- runtime reporting of advisory versus committed progression

### Tooling

Work that changes how users or developers observe or drive the system:

- CLI `run` and `trace` wording or output
- REPL prompts, `:type` / `:ast` / inspection outputs, or session summaries
- trace formatting that may later distinguish advisory, gated, and committed stages

### Docs-facing

Work that stays in documentation or reference notes:

- terminology clarification
- interaction contract references
- implementation-planning guidance
- later `SPEC-002` explanatory text

### Surface-docs-only

Work that explains how humans should read the workflow without adding syntax:

- advisory / gated / committed stage guidance
- runtime authority explanation
- separation between monitorability and projection

## Explicit Non-Overlap

The following remain runtime-only or otherwise protected from being repurposed as reasoner
projection:

- monitor views
- `exposes` clauses
- workflow observability
- `MonitorLink`
- capability verification
- approval routing
- mailbox scheduling
- effect execution
- provenance capture
- official workflow history

These features may be relevant to later implementation planning, but only as runtime or tooling
surfaces. They do not become reasoner-context machinery by default.

## Out of Scope

This note does not authorize:

- parser redesign
- lowering redesign
- type-system redesign
- syntax changes
- new stage annotations
- new runtime/reasoner placement markers in the language
- new interaction transports
- code-facing task creation

Those belong to later planning phases, if and only if the docs corpus shows they are really needed.

## Use in Later Planning

When later implementation planning begins, it should use this note to decide:

- which existing runtime entry points need interaction-aware treatment,
- which surfaces only need wording or reference updates,
- which areas stay runtime-only,
- and which apparent gaps are actually out of scope for now.

The intended output is a clean implementation-planning map, not a redesign of the language surface.

