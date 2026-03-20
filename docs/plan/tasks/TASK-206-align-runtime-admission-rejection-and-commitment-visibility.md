# TASK-206: Align Runtime Admission, Rejection, and Commitment Visibility

## Status: ✅ Complete

## Description

Align engine/interpreter entry points and runtime helper boundaries so admission, rejection,
commitment, and runtime-owned failure surfaces are explicit and consistent.

## Specification Reference

- SPEC-004: Operational Semantics
- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix
- SPEC-021: Runtime Observable Behavior

## Reference Contract

- `docs/plan/2026-03-20-runtime-boundary-steering-brief.md`
- `docs/reference/runtime-observable-behavior-contract.md`
- `docs/reference/runtime-reasoner-separation-rules.md`

## Requirements

### Functional Requirements

1. Make runtime-owned admission, rejection, and commitment boundaries explicit across engine and
   interpreter entry points
2. Align observe/set/send/receive boundary behavior with the canonical runtime outcome model
3. Add focused tests covering visible runtime boundary outcomes and rejection classes
4. Keep the work runtime-first and separate from CLI/REPL presentation concerns
5. Replace the transitional process-global control-link registry from TASK-205 with explicit
   runtime-owned lifecycle state, and make the current retention policy explicit for terminated
   instances

## Files

- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-interp/src/lib.rs`
- Add: `crates/ash-interp/src/runtime_state.rs`
- Modify: `crates/ash-interp/src/execute_observe.rs`
- Modify: `crates/ash-interp/src/execute_set.rs`
- Modify: `crates/ash-interp/src/exec_send.rs`
- Modify: `crates/ash-interp/src/execute.rs`
- Modify: `crates/ash-interp/src/execute_stream.rs`
- Test: `crates/ash-engine/tests/runtime_boundary_visibility.rs`
- Test: `crates/ash-interp/tests/runtime_boundary_visibility.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add focused tests for:
- explicit runtime rejection at boundary failures,
- consistent observable outcomes across observe/set/send/receive,
- preserved runtime-owned commitment behavior at engine/interpreter entry points,
- cross-execution control-link behavior backed by runtime-owned state rather than per-call or
  process-global fallback storage.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-engine --test runtime_boundary_visibility -- --nocapture
cargo test -p ash-interp --test runtime_boundary_visibility -- --nocapture
```

Expected: fail for current mismatches or missing explicit boundary behavior.

### Step 3: Implement the minimal fix (Green)

Align runtime boundary behavior with the canonical runtime contract.

### Step 4: Verify focused GREEN

Run the same commands again.

Expected: pass.

### Step 5: Verify broader GREEN

Run:

```bash
cargo test -p ash-engine
cargo test -p ash-interp
```

Expected: pass.

### Step 6: Commit

```bash
git add crates/ash-engine/src/lib.rs crates/ash-engine/tests/runtime_boundary_visibility.rs crates/ash-interp/src/lib.rs crates/ash-interp/src/runtime_state.rs crates/ash-interp/src/execute.rs crates/ash-interp/src/execute_stream.rs crates/ash-interp/tests/runtime_boundary_visibility.rs CHANGELOG.md
git commit -m "fix: align runtime boundary visibility"
```

## Completion Checklist

- [x] failing runtime-boundary visibility tests added
- [x] failure verified
- [x] admission/rejection/commitment boundaries aligned
- [x] focused and broader verification passing
- [x] `CHANGELOG.md` updated

## Resolution Note

`TASK-206` adopts explicit tombstone retention for terminated control targets as the current
runtime behavior. Killed instances remain observable as terminated across later executions that
share the same `RuntimeState`. The long-term bounded-retention and cleanup design is deferred to
[TASK-212](TASK-212-design-control-link-retention-policy.md).

## Non-goals

- No CLI or REPL output redesign
- No provenance presentation wording
- No new interaction-layer transport or projection behavior

## Task Note

`TASK-205` intentionally uses a shared process-global `ControlLinkRegistry` as a transitional fix
so transferred control links remain valid across top-level executions. `TASK-206` replaces that
fallback with explicit runtime-owned state and freezes tombstone retention as the current runtime
behavior. Long-term cleanup policy is tracked separately by
[TASK-212](TASK-212-design-control-link-retention-policy.md).

## Dependencies

- Depends on: TASK-205
- Blocks: TASK-207
