# TASK-170: Implement End-to-End Receive Execution

## Status: 📝 Planned

## Description

Implement canonical end-to-end `receive` behavior from parsed form through interpreter/runtime execution.

This task activates the frozen `receive` contract using the aligned core, type, and runtime
verification layers.

## Specification Reference

- SPEC-004: Operational Semantics
- SPEC-013: Streams and Event Processing
- SPEC-017: Capability Integration

## Reference Contract

- `docs/reference/type-to-runtime-contract.md`
- `docs/reference/runtime-observable-behavior-contract.md`

## Requirements

### Functional Requirements

1. Execute canonical non-blocking, blocking, and timed `receive` forms end-to-end
2. Respect control-mailbox semantics
3. Integrate runtime verification and declaration rules
4. Add integration tests proving canonical behavior from parsed form through execution

## Files

- Modify: `crates/ash-interp/src/execute_stream.rs`
- Modify: `crates/ash-interp/src/eval.rs`
- Modify: `crates/ash-interp/src/stream.rs`
- Test: `crates/ash-interp/tests/receive_execution.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add integration tests proving canonical `receive` behavior from parsed form through execution.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-interp receive_execution -- --nocapture
```

Expected: fail because `receive` is not fully implemented end-to-end.

### Step 3: Implement the minimal fix (Green)

Implement canonical runtime behavior for `receive` using the frozen core and verification contracts.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-interp receive_execution -- --nocapture
```

Expected: pass.

### Step 5: Verify broader GREEN

Run:

```bash
cargo test -p ash-interp
```

Expected: pass.

### Step 6: Commit

```bash
git add crates/ash-interp/src/execute_stream.rs crates/ash-interp/src/eval.rs crates/ash-interp/src/stream.rs crates/ash-interp/tests/receive_execution.rs CHANGELOG.md
git commit -m "feat: implement end-to-end receive execution"
```

## Completion Checklist

- [ ] failing receive-execution tests added
- [ ] failure verified
- [ ] canonical end-to-end `receive` execution implemented
- [ ] focused and broader verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No REPL exposure yet
- No fairness/source-scheduling modifier expansion beyond current defaults

## Dependencies

- Depends on: TASK-168, TASK-169
- Blocks: TASK-171
