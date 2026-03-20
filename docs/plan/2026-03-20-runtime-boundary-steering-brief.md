# Runtime Boundary Steering Brief

Date: 2026-03-20
Status: TASK-201 output

## Purpose

This brief synthesizes:

- [Runtime Execution Boundaries Interaction Planning Review](../audit/2026-03-20-runtime-execution-boundaries-interaction-planning-review.md)
- [Runtime Trace and Provenance Planning Review](../audit/2026-03-20-runtime-trace-and-provenance-planning-review.md)
- [Runtime Boundary Implementation Planning Plan](2026-03-20-runtime-boundary-implementation-planning-plan.md)

Its purpose is to define the next runtime code-facing task clusters that should later exist,
without opening them yet, and to make the runtime-boundary review checkpoint explicit.

## Steering Outcome

The runtime-boundary phase confirms a clean separation:

- authoritative runtime execution boundaries are `runtime-only`
- trace, provenance, export, and workflow-wrapper surfaces are `runtime-only`
- no runtime execution or trace surface needs to be reclassified as projection or advisory
  interaction machinery

The remaining work is ordinary runtime hardening and boundary surfacing, not runtime/reasoner
redesign.

## Runtime-Boundary Task Clusters To Open Later

### 1. Runtime execution completeness cluster

This cluster should cover the runtime branches that are already clearly authoritative but remain
incomplete or partially stubbed.

Expected focus:

- `Workflow::Act` execution path
- control-link branches such as `Check`, `Kill`, `Pause`, `Resume`, and `CheckHealth`
- any remaining runtime `ActionFailed` or continuation-only placeholders that should converge to
  the hardened specs

This is runtime completeness work, not interaction-layer work.

### 2. Runtime acceptance and commitment visibility cluster

This cluster should cover places where the runtime already owns acceptance, rejection, admission,
and commitment, but later implementation work may need to make those boundaries more explicit in
code or traces.

Expected focus:

- engine and interpreter execution entry points
- observation, set, send, and receive admission points
- policy-gated branches and explicit rejection surfaces
- provenance capture around accepted effects

This cluster should stay authoritative and runtime-first.

### 3. Runtime trace/provenance hardening cluster

This cluster should cover the runtime-owned trace and provenance surfaces that may later need
better consistency or stronger linkage to accepted runtime progression.

Expected focus:

- `TraceRecorder` and `TraceEvent` usage consistency
- workflow wrapper entry/exit framing
- provenance export helpers
- alignment between runtime execution boundaries and trace recording points

This cluster must preserve the rule that observability is not projection.

## Explicit Non-Goals For Later Runtime Work

The runtime-boundary review does not authorize:

- projection redesign
- new runtime-to-reasoner transports
- CLI or REPL wording changes
- monitor or `exposes` reinterpretation
- using trace/provenance as injected reasoner context

Those belong either to the separate interaction contract or to later tooling/surface work.

## Review Checkpoint

Review this brief before opening any runtime code-facing tasks.

The steering questions are:

1. Are the identified runtime boundaries still fully meaningful without any reasoner present?
2. Are runtime acceptance, rejection, commitment, and provenance responsibilities still explicit
   and runtime-owned?
3. Did any proposed runtime cluster accidentally smuggle in tooling or interaction-layer work?

If the answer to any question is no, revise the task cluster boundary before opening tasks.

## Recommendation

The next runtime implementation-planning move should be to open a small runtime-first task set
derived from these three clusters, with the first tasks aimed at runtime completeness rather than
interaction-specific behavior.

In practical terms:

- open runtime completeness tasks first
- keep trace/provenance hardening as a separate cluster
- leave wording, presentation, and stage-label surfacing to the tooling/surface phase

## Closure

The runtime-boundary phase is complete.

It found no runtime/reasoner conflation in the authoritative execution path.
The only remaining runtime work is implementation hardening and explicit boundary surfacing inside
the runtime itself.
