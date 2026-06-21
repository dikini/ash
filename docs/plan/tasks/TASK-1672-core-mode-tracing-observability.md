# TASK-1672: Add thunk tracing and observability

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Add trace/observability events for thunk construction, forcing, memo cache behavior, replay, and re-entrant rejection.

## Specification Reference

- [SPEC-101 §12](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#12-tracing-and-observability)

## Dependencies

- [TASK-1664](TASK-1664-cps-force-runtime.md)
- [TASK-1669](TASK-1669-core-mode-lowering.md)

## Requirements

1. Add thunk trace event variants to `ash_core::TraceEvent` in
   `crates/ash-core/src/provenance.rs`.
2. Add emission helpers in `crates/ash-interp/src/execution_record.rs` when the semantic
   execution-record pipeline is used.
3. If the CPS evaluator remains separate from execution records, add a small CPS runtime trace
   sink alongside `crates/ash-interp/src/cps/mod.rs` for direct CPS tests.
4. Trace thunk construction.
5. Trace lazy/memo force start and end.
6. Trace memo cache fill, hit, and terminal outcome replay.
7. Trace re-entrant-force rejection.
8. Do not expose raw memo-cell storage addresses.
9. Use the exact public trace event variant names and payload shapes listed below.
10. Every new public trace event timestamp field must use `DateTime<Utc>`, matching existing
    `TraceEvent` variants.
11. Update `trace_event_timestamp` in `crates/ash-interp/src/execution_record.rs` to include all
    new variants.
12. The CPS runtime trace sink is `Vec<TraceEvent>`, not a separate internal event type.
13. Use the exact stable outcome strings listed below.

## Existing Code Touchpoints

- `crates/ash-core/src/provenance.rs`: add `TraceEvent` variants with `DateTime<Utc>` timestamps.
- `crates/ash-interp/src/execution_record.rs`: update `trace_event_timestamp` and emission helpers.
- `crates/ash-interp/src/cps/mod.rs`: add or connect the CPS runtime trace sink if direct CPS
  evaluation remains separate from execution records.

## Observation Path

Prefer public trace observation through `ExecutionRecord::trace()` and
`SemanticWorkflowOutcome::trace()`. Direct CPS-only tests may observe the dedicated CPS trace sink
only when the force runtime is not yet connected to the semantic execution-record pipeline.

## Required Trace Events

Add these public `ash_core::TraceEvent` variants. Use string payloads for mode/outcome summaries
to keep the public trace independent of interpreter-private memo storage:

```rust
TraceEvent::ThunkConstructed { mode: String, row: Vec<String>, timestamp: DateTime<Utc> }
TraceEvent::ThunkForceStarted { mode: String, timestamp: DateTime<Utc> }
TraceEvent::ThunkBodyEvaluationStarted { mode: String, timestamp: DateTime<Utc> }
TraceEvent::ThunkBodyEvaluationCompleted { mode: String, outcome: String, timestamp: DateTime<Utc> }
TraceEvent::ThunkForceCompleted { mode: String, outcome: String, timestamp: DateTime<Utc> }
TraceEvent::MemoCacheFilled { outcome: String, timestamp: DateTime<Utc> }
TraceEvent::MemoCacheHit { outcome: String, timestamp: DateTime<Utc> }
TraceEvent::MemoReplayFailure { reason: String, timestamp: DateTime<Utc> }
TraceEvent::MemoReentrantRejected { timestamp: DateTime<Utc> }
```

No event may include `MemoCellId`, raw pointers, or process-local storage addresses.

## Outcome Strings

Use these exact strings in `outcome` and `reason` payloads:

| Runtime outcome | Trace string |
|-----------------|--------------|
| `Ok(_)` | `"success"` |
| `Err(CpsError::Trap(_))` | `"trap"` |
| `Err(CpsError::UnhandledEffect(_))` | `"unhandled-effect"` |
| any other `Err(CpsError::...)` | `"runtime-error"` |

`MemoReplayFailure.reason` uses the same string mapping. Do not introduce task-local synonyms
such as `"failed"`, `"error"`, or `"panic"` in Phase 163 tests.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1672_core_mode_tracing_docs_consistency.rs`
   and runtime trace tests at the observation path above.
2. Run focused tests and confirm missing trace events.
3. Add trace event variants/metadata.
4. Re-run focused tests plus affected runtime tests.

## Completion Checklist

- [x] All SPEC-101 trace event families are represented.
- [x] Memo cell internals are not exposed.
- [x] Trace docs name the events.
- [x] `trace_event_timestamp` handles every new thunk/memo variant.
- [x] `CpsRuntime.trace` stores public `TraceEvent` values directly.
- [x] Trace outcome/reason payloads use only the stable strings listed above.
- [x] Fixture tests can count body-evaluation/cache events to distinguish lazy rerun from memo hit.
