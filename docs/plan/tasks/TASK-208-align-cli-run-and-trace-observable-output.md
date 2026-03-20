# TASK-208: Align CLI Run and Trace Observable Output

## Status: ✅ Complete

## Description

Align `ash run` and `ash trace` user-visible output with the frozen runtime-observable behavior
contract.

## Specification Reference

- SPEC-005: CLI
- SPEC-011: REPL
- SPEC-021: Runtime Observable Behavior

## Reference Contract

- `docs/plan/2026-03-20-tooling-surface-steering-brief.md`
- `docs/reference/runtime-observable-behavior-contract.md`
- `docs/reference/surface-guidance-boundary.md`

## Requirements

### Functional Requirements

1. Align `ash run` result and trace-summary messaging with the canonical observable contract
2. Align `ash trace` export confirmations, integrity acknowledgements, and observable error output
3. Add focused CLI tests proving the canonical user-visible output categories
4. Keep the work presentation-level and runtime-observable, not semantic redesign

## Files

- Modify: `crates/ash-cli/src/commands/run.rs`
- Modify: `crates/ash-cli/src/commands/trace.rs`
- Test: `crates/ash-cli/tests/run_output.rs`
- Test: `crates/ash-cli/tests/trace_output.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add focused tests for:
- `ash run` result and trace-summary output,
- `ash trace` export and integrity output,
- distinct observable error/output categories required by the canonical contract.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-cli run_output -- --nocapture
cargo test -p ash-cli trace_output -- --nocapture
```

Expected: fail because the current CLI output still drifts from the frozen observable contract.

### Step 3: Implement the minimal fix (Green)

Align `ash run` and `ash trace` output with the canonical observable contract.

### Step 4: Verify focused GREEN

Run the same commands again.

Expected: pass.

### Step 5: Verify broader GREEN

Run:

```bash
cargo test -p ash-cli
```

Expected: pass.

### Step 6: Commit

```bash
git add crates/ash-cli/src/commands/run.rs crates/ash-cli/src/commands/trace.rs crates/ash-cli/tests/run_output.rs crates/ash-cli/tests/trace_output.rs CHANGELOG.md
git commit -m "fix: align cli observable output"
```

## Completion Checklist

- [x] failing CLI observable-output tests added
- [x] failure verified
- [x] `run` and `trace` output aligned
- [x] focused and broader verification passing
- [x] `CHANGELOG.md` updated

## Non-goals

- No new syntax
- No stage-guidance overlay work
- No runtime semantic redesign

## Dependencies

- Depends on: TASK-173
- Blocks: TASK-176
