# Runtime Execution Boundaries Interaction Planning Review

## Scope

Reviewed the authoritative runtime execution boundaries named in the implementation-planning
surface:

- [docs/plan/2026-03-20-runtime-reasoner-implementation-planning-surface.md](/home/dikini/Projects/ash/docs/plan/2026-03-20-runtime-reasoner-implementation-planning-surface.md)
- [crates/ash-engine/src/lib.rs](/home/dikini/Projects/ash/crates/ash-engine/src/lib.rs)
- [crates/ash-interp/src/lib.rs](/home/dikini/Projects/ash/crates/ash-interp/src/lib.rs)
- [crates/ash-interp/src/execute.rs](/home/dikini/Projects/ash/crates/ash-interp/src/execute.rs)
- [crates/ash-interp/src/execute_observe.rs](/home/dikini/Projects/ash/crates/ash-interp/src/execute_observe.rs)
- [crates/ash-interp/src/execute_set.rs](/home/dikini/Projects/ash/crates/ash-interp/src/execute_set.rs)
- [crates/ash-interp/src/exec_send.rs](/home/dikini/Projects/ash/crates/ash-interp/src/exec_send.rs)

Review protocol:

- [Runtime-Reasoner Separation Rules](/home/dikini/Projects/ash/docs/reference/runtime-reasoner-separation-rules.md)

Constraints of this audit:

- Reporting only
- No normative spec files changed
- Classify each reviewed area as `runtime-only`, `interaction-layer`, or `split`
- Distinguish `Aligned`, `Silent`, and `Tension`

Review date: 2026-03-20

## Summary

The reviewed runtime execution surfaces are consistently `runtime-only`. The authoritative entry
points, interpreter branches, and capability-admission helpers all make sense without a reasoner
present. No `interaction-layer` or `split` concern was found in the runtime execution path itself.

The main residual tension is implementation completeness, not runtime/reasoner overlap. Several
branches still short-circuit with TODO-level behavior or explicit errors, but those are runtime
gaps and do not create a reasoner-facing contract in this phase.

## Findings

### 1. Engine entry points are `runtime-only` and aligned

- [crates/ash-engine/src/lib.rs:200-228](/home/dikini/Projects/ash/crates/ash-engine/src/lib.rs#L200-L228)

Classification: `runtime-only`

Status: `Aligned`

Reasoning: `Engine::execute`, `Engine::run`, and `Engine::run_file` form a pure
parse-check-execute chain. They own runtime authority over workflow execution, but they do not
introduce any reasoner-specific semantics, projection, or advisory acceptance model.

### 2. `interpret` is a `runtime-only` convenience boundary

- [crates/ash-interp/src/lib.rs:91-93](/home/dikini/Projects/ash/crates/ash-interp/src/lib.rs#L91-L93)
- [crates/ash-interp/src/execute.rs:640-645](/home/dikini/Projects/ash/crates/ash-interp/src/execute.rs#L640-L645)

Classification: `runtime-only`

Status: `Aligned`

Reasoning: `interpret` delegates to `execute_simple`, which constructs default runtime contexts and
invokes the interpreter. This is a runtime convenience entry point, not an interaction-layer
bridge.

### 3. `execute_workflow_with_behaviour` is the main runtime execution boundary

- [crates/ash-interp/src/execute.rs:73-80](/home/dikini/Projects/ash/crates/ash-interp/src/execute.rs#L73-L80)
- [crates/ash-interp/src/execute.rs:81-570](/home/dikini/Projects/ash/crates/ash-interp/src/execute.rs#L81-L570)

Classification: `runtime-only`

Status: `Aligned`

Reasoning: This function owns the central runtime branch structure for workflows. It handles
observation, orientation, proposal, policy decision, set/send admission, and control-link stubs
entirely within runtime semantics.

### 4. Observation and set/send helpers are explicit runtime admission points

- [crates/ash-interp/src/execute_observe.rs:14-89](/home/dikini/Projects/ash/crates/ash-interp/src/execute_observe.rs#L14-L89)
- [crates/ash-interp/src/execute_set.rs:12-66](/home/dikini/Projects/ash/crates/ash-interp/src/execute_set.rs#L12-L66)
- [crates/ash-interp/src/exec_send.rs:14-72](/home/dikini/Projects/ash/crates/ash-interp/src/exec_send.rs#L14-L72)

Classification: `runtime-only`

Status: `Aligned`

Reasoning: `execute_observe` samples behavior and extends the runtime context, `execute_set`
validates before mutating a writable provider, and `execute_send` performs capability lookup,
backpressure handling, and effectful send. These are all runtime admission/commitment boundaries.

### 5. Policy decision and action branches are `runtime-only`, with runtime implementation gaps

- [crates/ash-interp/src/execute.rs:230-306](/home/dikini/Projects/ash/crates/ash-interp/src/execute.rs#L230-L306)
- [crates/ash-interp/src/execute.rs:418-569](/home/dikini/Projects/ash/crates/ash-interp/src/execute.rs#L418-L569)

Classification: `runtime-only`

Status: `Tension`

Reasoning: `Workflow::Decide` enforces runtime policy gating with an explicit `Permit` / `Deny`
split. `Workflow::Act` checks a guard but currently returns `ActionFailed`, and the control-link
branches (`Check`, `Kill`, `Pause`, `Resume`, `CheckHealth`) still short-circuit with continuation
or TODO behavior. That is a runtime completeness gap, not a reasoner boundary problem.

## Cross-Cutting Observation

No reviewed area was `interaction-layer` or `split`. That is the correct result for this phase:
runtime execution boundaries should stay authoritative and self-contained, while projection,
advisory reasoning, and acceptance-from-reasoner remain separate concerns.

The only substantive follow-up pressure is on runtime completeness around action execution and
control-link handling. Those are the right subjects for later runtime-boundary implementation
planning, but they do not require a reasoner model.

## Conclusion

No tensions were found between the runtime execution path and the runtime/reasoner separation
rules. The runtime boundary is intact, the acceptance/rejection/commitment points are in the
runtime layer, and the residual work is ordinary runtime implementation hardening.
