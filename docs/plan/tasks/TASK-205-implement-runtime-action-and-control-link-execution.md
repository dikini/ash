# TASK-205: Implement Runtime Action and Control-Link Execution

## Status: ✅ Complete

## Description

Implement the remaining runtime execution branches that are currently incomplete or stubbed,
specifically the canonical `Act` path and control-link operations.

## Specification Reference

- SPEC-004: Operational Semantics
- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix

## Reference Contract

- `docs/plan/2026-03-20-runtime-boundary-steering-brief.md`
- `docs/reference/runtime-reasoner-separation-rules.md`
- `docs/reference/runtime-observable-behavior-contract.md`

## Requirements

### Functional Requirements

1. Implement canonical runtime `Act` execution instead of stub failure behavior
2. Implement or converge the control-link runtime branches (`Check`, `Kill`, `Pause`, `Resume`,
   `CheckHealth`) to the hardened runtime contract
3. Add focused interpreter tests proving the runtime outcomes for action and control-link execution
4. Keep the implementation runtime-only with no CLI/REPL or projection semantics

## Files

- Modify: `crates/ash-interp/src/execute.rs`
- Modify: `crates/ash-interp/src/control_link.rs`
- Modify: `crates/ash-interp/src/execute_stream.rs`
- Test: `crates/ash-interp/tests/runtime_action_control.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add focused tests for:
- successful `Act` execution,
- guarded action rejection,
- canonical control-link outcomes,
- absence of placeholder `ActionFailed` / continuation-only fallback in the targeted branches.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-interp --test runtime_action_control -- --nocapture
```

Expected: fail because the targeted runtime branches are incomplete today.

### Step 3: Implement the minimal fix (Green)

Implement the targeted runtime branches to satisfy the hardened runtime contract.

### Step 4: Verify focused GREEN

Run:

```bash
cargo test -p ash-interp --test runtime_action_control -- --nocapture
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
git add crates/ash-interp/src/execute.rs crates/ash-interp/src/control_link.rs crates/ash-interp/src/execute_stream.rs crates/ash-interp/tests/runtime_action_control.rs CHANGELOG.md
git commit -m "feat: complete runtime action and control-link execution"
```

## Completion Checklist

- [x] failing runtime action/control tests added
- [x] failure verified
- [x] runtime `Act` and control-link branches implemented
- [x] focused and broader verification passing
- [x] `CHANGELOG.md` updated

## Non-goals

- No CLI or REPL changes
- No new projection or interaction-layer behavior
- No monitor or `exposes` reinterpretation

## Dependencies

- Depends on: TASK-170, TASK-171, TASK-211
- Blocks: TASK-206
