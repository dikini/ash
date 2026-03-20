# Trace Export and Presentation Planning Review

Date: 2026-03-20
Task: TASK-203

## Scope

Reviewed the trace export and presentation surfaces against:

- [Runtime Observable Behavior Contract](/home/dikini/Projects/ash/docs/reference/runtime-observable-behavior-contract.md)
- [Surface Guidance Boundary](/home/dikini/Projects/ash/docs/reference/surface-guidance-boundary.md)
- [Runtime-to-Reasoner Interaction Contract](/home/dikini/Projects/ash/docs/reference/runtime-to-reasoner-interaction-contract.md)
- [Runtime-Reasoner Separation Rules](/home/dikini/Projects/ash/docs/reference/runtime-reasoner-separation-rules.md)
- [ash-cli trace command](/home/dikini/Projects/ash/crates/ash-cli/src/commands/trace.rs)
- [ash-provenance export and trace surfaces](/home/dikini/Projects/ash/crates/ash-provenance/src/export.rs)
- [ash-provenance trace recording](/home/dikini/Projects/ash/crates/ash-provenance/src/trace.rs)

Review protocol:

- keep runtime-observable behavior distinct from explanatory surface guidance
- ask whether each surface still makes sense without a reasoner present
- report only; do not change normative specs

## Summary

The reviewed trace export and presentation surfaces are consistently `runtime-only`. The trace
command, provenance recorders, export helpers, and output formatting all make complete sense in a
reasoner-free execution model. No `interaction-layer` or `split` concern was found in the trace
export and presentation semantics themselves.

The remaining follow-up pressure is presentation-level rather than semantic: later tooling planning
may want to decide how to describe accepted versus rejected progression, or whether trace output
should use advisory/gated/committed wording for human readers. Those decisions belong in tooling
and surface guidance, not in trace ownership or export semantics.

## Findings

### 1. Trace command execution and export helpers are runtime-only and aligned

Classification: `runtime-only`

Status: `Aligned`

Reasoning: `crates/ash-cli/src/commands/trace.rs` parses workflow source, executes it through the
runtime, records provenance events, and exports the captured trace in JSON, NDJSON, or CSV. These
behaviors are runtime presentation and export concerns, not reasoner projection or advisory
interaction semantics.

### 2. Provenance recorder and trace-event surfaces are runtime-only and aligned

Classification: `runtime-only`

Status: `Aligned`

Reasoning: `TraceRecorder`, `TraceEvent`, and `ExportFormat` in `ash-provenance` define
authoritative runtime audit structures. They still make complete sense without any reasoner
present and remain the semantic source of trace history and export formatting.

### 3. Trace output does not overlap with reasoner projection

Classification: `runtime-only`

Status: `Aligned`

Reasoning: The trace output surfaces report runtime execution, provenance, and value capture. They
do not inject context to a reasoner, do not encode advisory derivation, and do not alter runtime
authority.

### 4. Stage-aware wording is a tooling and documentation concern, not a runtime semantic concern

Classification: `runtime-only`

Status: `Silent`

Reasoning: If later planning wants trace output or exported reports to describe advisory, gated,
or committed progression, that is a presentation decision for tooling or documentation. The runtime
contracts should stay focused on authoritative capture, export, and trace integrity.

## Cross-Cutting Observation

This audit does not introduce any blocking dependency for later tooling planning. It confirms that
trace export and presentation already belong on the runtime side of the boundary and that any
future stage-label wording must be layered on top as explanatory guidance.

The presentation surface may need to be careful not to use trace exports as a proxy for hidden
reasoner state. The correct reading remains: trace data is runtime-owned history, not reasoner
projection.

## Conclusion

No blocking contradiction was found in the trace export and presentation surfaces.
The semantics stay runtime-only, and the only remaining steering question is presentation-level:
how should later tooling or docs describe accepted versus rejected progression without making trace
output look like injected reasoner context?
