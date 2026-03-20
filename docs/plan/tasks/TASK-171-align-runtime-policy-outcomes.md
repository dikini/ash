# TASK-171: Align Runtime Policy Outcomes

## Status: 📝 Planned

## Description

Align runtime policy outcomes with the canonical deny, warning, transform, and approval contract.

This task ensures runtime verification and interpreter execution apply policy outcomes in the
same way across observe/set/send boundaries.

## Specification Reference

- SPEC-017: Capability Integration
- SPEC-018: Capability Runtime Verification Matrix

## Reference Contract

- `docs/reference/type-to-runtime-contract.md`
- `docs/reference/runtime-observable-behavior-contract.md`

## Requirements

### Functional Requirements

1. Align runtime deny, warning, transform, and approval behavior with the canonical contract
2. Apply the same policy-outcome story across verification and interpreter boundaries
3. Add tests covering the canonical runtime outcomes

## Files

- Modify: `crates/ash-typeck/src/runtime_verification.rs`
- Modify: `crates/ash-interp/src/execute_observe.rs`
- Modify: `crates/ash-interp/src/execute_set.rs`
- Modify: `crates/ash-interp/src/exec_send.rs`
- Test: `crates/ash-typeck/tests/policy_runtime_outcomes.rs`
- Test: `crates/ash-interp/tests/policy_runtime_outcomes.rs`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write the failing tests (Red)

Add tests for deny, warning, transform, and approval behavior at the runtime boundary.

### Step 2: Verify RED

Run:

```bash
cargo test -p ash-typeck policy_runtime_outcomes -- --nocapture
cargo test -p ash-interp policy_runtime_outcomes -- --nocapture
```

Expected: fail for current mismatches.

### Step 3: Implement the minimal fix (Green)

Update runtime verification and interpreter integration to match the canonical policy-outcome contract.

### Step 4: Verify focused GREEN

Run the same commands again.

Expected: pass.

### Step 5: Commit

```bash
git add crates/ash-typeck/src/runtime_verification.rs crates/ash-interp/src/execute_observe.rs crates/ash-interp/src/execute_set.rs crates/ash-interp/src/exec_send.rs crates/ash-typeck/tests/policy_runtime_outcomes.rs crates/ash-interp/tests/policy_runtime_outcomes.rs CHANGELOG.md
git commit -m "fix: align runtime policy outcomes"
```

## Completion Checklist

- [ ] failing runtime policy-outcome tests added
- [ ] failure verified
- [ ] runtime policy outcomes aligned
- [ ] focused verification passing
- [ ] `CHANGELOG.md` updated

## Non-goals

- No provider-effect redesign
- No new policy features

## Dependencies

- Depends on: TASK-166, TASK-168, TASK-169, TASK-170
- Blocks: TASK-176, TASK-205
