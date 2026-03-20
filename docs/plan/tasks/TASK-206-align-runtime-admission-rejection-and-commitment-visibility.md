# TASK-206: Align Runtime Admission, Rejection, and Commitment Visibility

## Status: 📝 Planned

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
   runtime-owned lifecycle state, including a documented cleanup versus tombstone policy for
   terminated instances

## Files

- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-interp/src/execute_observe.rs`
- Modify: `crates/ash-interp/src/execute_set.rs`
- Modify: `crates/ash-interp/src/exec_send.rs`
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
cargo test -p ash-engine runtime_boundary_visibility -- --nocapture
cargo test -p ash-interp runtime_boundary_visibility -- --nocapture
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
git add crates/ash-engine/src/lib.rs crates/ash-interp/src/execute_observe.rs crates/ash-interp/src/execute_set.rs crates/ash-interp/src/exec_send.rs crates/ash-interp/src/execute_stream.rs crates/ash-engine/tests/runtime_boundary_visibility.rs crates/ash-interp/tests/runtime_boundary_visibility.rs CHANGELOG.md
git commit -m "fix: align runtime boundary visibility"
```

## Completion Checklist

- [ ] failing runtime-boundary visibility tests added
- [ ] failure verified
- [ ] admission/rejection/commitment boundaries aligned
- [ ] focused and broader verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No CLI or REPL output redesign
- No provenance presentation wording
- No new interaction-layer transport or projection behavior

## Task Note

`TASK-205` intentionally uses a shared process-global `ControlLinkRegistry` as a transitional fix
so transferred control links remain valid across top-level executions. `TASK-206` must replace that
fallback with explicit runtime-owned state and define whether terminated instances remain observable
as tombstones or are eagerly removed after terminal control.

## Dependencies

- Depends on: TASK-205
- Blocks: TASK-207
