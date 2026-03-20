# Runtime Trace and Provenance Planning Review

Date: 2026-03-20
Task: TASK-200

## Scope

Reviewed the runtime trace, provenance, export, and workflow-wrapper surfaces against:

- [Runtime-Reasoner Implementation-Planning Surface](/home/dikini/Projects/ash/docs/reference/runtime-reasoner-implementation-planning-surface.md)
- [Runtime-to-Reasoner Interaction Contract](/home/dikini/Projects/ash/docs/reference/runtime-to-reasoner-interaction-contract.md)
- [Runtime-Reasoner Separation Rules](/home/dikini/Projects/ash/docs/reference/runtime-reasoner-separation-rules.md)
- [SPEC-004: Operational Semantics](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](/home/dikini/Projects/ash/docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)

Review protocol:

- keep the runtime-only versus interaction-layer split intact
- ask whether each surface still makes sense without a reasoner present
- report only; do not change normative specs

## Summary

The reviewed trace, provenance, and workflow-wrapper surfaces are consistently `runtime-only`.
They define authoritative execution reporting, trace capture, export, and wrapper framing without
depending on a reasoner being present. No `interaction-layer` or `split` concern was found in the
surface semantics themselves.

The remaining follow-up pressure is planning-level rather than semantic: later implementation
planning may need to decide where runtime traces present stage-like wording or how tooling renders
accepted versus rejected progression. Those decisions belong in tooling/surface planning, not in
trace/provenance ownership.

## Findings

### 1. Trace recorder and trace event surfaces are runtime-only and aligned

Classification: `runtime-only`

Status: `Aligned`

Reasoning: The trace recorder, trace event, and export helpers are authoritative runtime
mechanisms. They still make complete sense in a reasoner-free execution model and remain
responsible for runtime history, not projected reasoner context.

### 2. Workflow wrapper enter/exit hooks are runtime-only and aligned

Classification: `runtime-only`

Status: `Aligned`

Reasoning: The workflow wrapper boundaries frame execution entry and exit for runtime bookkeeping,
provenance capture, and trace emission. They are useful regardless of whether a reasoner ever
participates.

### 3. Trace/provenance surfaces do not overlap with reasoner projection

Classification: `runtime-only`

Status: `Aligned`

Reasoning: The reviewed surfaces describe runtime observation and auditability, not injected context
or advisory reasoning. Projection remains a separate runtime-to-reasoner concern and should not be
derived from trace capture or export semantics.

### 4. Stage-aware wording is a tooling concern, not a runtime semantic concern

Classification: `runtime-only`

Status: `Silent`

Reasoning: If later planning wants traces or exports to show advisory, gated, or committed wording,
that is a presentation decision for CLI/REPL or docs-facing surfaces. The runtime contract should
stay focused on authoritative capture and export.

## Cross-Cutting Observation

The audit does not introduce any new blocking dependency for runtime-boundary planning. It instead
confirms that trace/provenance are already the right place to preserve runtime-owned observability
and that later interaction-aware work should only decide how to surface those facts, not how to
reclassify them.

## Conclusion

No blocking contradiction was found in the runtime trace and provenance surfaces.
The semantics stay runtime-only, and the only remaining steering question is presentation-level:
how should later tooling or docs describe accepted versus rejected progression without making trace
capture look like reasoner projection?
