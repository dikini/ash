# TASK-1671: Add Core mode end-to-end fixtures

**Status:** Planned
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Add parse -> validate -> type-check -> lower -> run fixtures and golden examples for lazy and memo modes.

## Specification Reference

- [SPEC-101 §13](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#13-acceptance-criteria)

## Dependencies

- [TASK-1670](TASK-1670-core-thunk-capture-authority.md)
- [TASK-1672](TASK-1672-core-mode-tracing-observability.md)

## Requirements

1. Add `.core` fixtures for lazy re-run, memo single-run, cached failure/trap, re-entrant rejection, captured handler, and mode mismatch.
2. Add CPS golden or structural assertions for lowered thunk forms.
3. Add integration tests that run the full Core pipeline.
4. Keep fixture text canonical through serializer round-trip where possible.
5. `memo_reentrant_trap.core` should use `LetRec` only if the Core syntax can express the
   self-force shape without broadening the phase. Otherwise, keep the re-entrant behavior covered
   by the direct CPS runtime test from TASK-1664 and document the Core fixture as deferred.

## Required Fixture Shapes

- `lazy_reruns.core`: force the same lazy binding twice and assert the effect/action preceding the
  returned value is observed twice.
- `memo_runs_once.core`: force the same memo binding twice and assert the effect/action preceding
  the returned value is observed once.
- `memo_caches_failure.core`: force a memo thunk that traps or reaches the lowered recoverable
  failure representation, then force it again and assert the preceding effect/action is not
  repeated.
- `memo_reentrant_trap.core`: prefer a Core `LetRec` self-force shape; if that is not expressible,
  reference the direct CPS test that inserts the thunk into its own captured environment.
- `force_captured_handler.core`: construct the thunk under one handler/provider chain, force it
  under another, and assert creation-time capture wins.
- `mode_mismatch_invalid.core`: use `(let-mode x lazy : (memo Int {}) 1 x)`.

## Observability Mechanism

Use trace counts as the required assertion mechanism:

- `lazy_reruns.core`: after two forces, assert two
  `ThunkBodyEvaluationStarted { mode: "lazy" }` events.
- `memo_runs_once.core`: after two forces, assert one
  `ThunkBodyEvaluationStarted { mode: "memo" }`, one `MemoCacheFilled`, and one `MemoCacheHit`.
- `memo_caches_failure.core`: assert one body-evaluation event, one failure/trap cache fill, and
  one `MemoReplayFailure` on the second force.
- `force_captured_handler.core`: assert the thunk body operation is handled by the creation-time
  chain, and use the body-evaluation trace to prove the body actually ran under force.

If a fixture reaches direct CPS runtime before the semantic execution-record trace path is wired,
use the `CpsRuntime` trace sink required by TASK-1663/TASK-1664 rather than ad hoc counters.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1671_core_mode_end_to_end.rs`.
2. Add required fixtures under `crates/ash-core/tests/fixtures/core/`.
3. Run the focused test and confirm missing behavior.
4. Fill implementation gaps only when fixture tests prove them.
5. Re-run `task_1671`, `task_1629`, and `task_1650`.

## Completion Checklist

- [ ] Required examples from PLAN-163 exist.
- [ ] Lowered thunk forms are checked.
- [ ] Runtime behavior is exercised where available.
- [ ] Lazy/memo execution counts are asserted through trace events, not informal comments.
