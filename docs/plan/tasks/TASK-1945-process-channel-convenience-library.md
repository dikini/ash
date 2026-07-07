# TASK-1945: Process/Channel Convenience Library

**Status:** Planned
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Add process/channel convenience helpers over Phase 195 semantics without weakening sendability,
ownership, cancellation, failure propagation, or trace evidence.

## Requirements

- Add helpers for spawn/join/await patterns, bounded worker pools, channel send/receive loops, and
  cancellation-aware cleanup where current syntax supports them.
- Preserve sendability and ownership validation.
- Preserve channel close/empty/full diagnostics and process failure classification.
- Emit process/channel trace evidence through existing runtime facts.

## TDD Steps

1. Add failing process/channel helper fixtures.
2. Implement minimal helper modules.
3. Add negative sendability/ownership/cancellation tests.
4. Run focused process/channel tests and Rust quality gates.

## Completion Checklist

- [ ] Helpers parse/check through stdlib imports.
- [ ] Sendability and ownership failures remain fail-closed.
- [ ] Cancellation and child failure propagation are preserved.
- [ ] Trace evidence remains structured and redacted.
