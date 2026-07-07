# TASK-1945: Process/Channel Convenience Library

**Status:** Complete
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

- [x] Helpers parse/check through stdlib imports.
- [x] Sendability and ownership failures remain fail-closed.
- [x] Cancellation and child failure propagation are preserved.
- [x] Trace evidence remains structured and redacted.

## Evidence

- Added pure `std::process` helper records for spawn/join plans, bounded worker pools,
  channel-loop plans, cancellation cleanup plans, sendability guards, channel diagnostic
  expectations, and process trace expectations.
- Helpers are metadata constructors only: they do not spawn processes, create channels, acquire
  provider authority, or bypass Phase 195 sendability, ownership, cancellation, failure, or trace
  semantics.
- Added `examples/11-process-channel-helpers/process_channel_helpers.ash`, a current-syntax
  fixture importing the helper surface through the real stdlib path.
- Focused verification:
  `cargo test -p ash-cli --test phase199_process_channel_helpers -- --nocapture`,
  `cargo test -p ash-cli --test phase199_current_syntax_audit -- --nocapture`, and
  `cargo test -p ash-cli --test example_corpus_check --test stdlib_corpus_check -- --nocapture`.
